//! Minimal, API stable CLI parser
//!
//! Inspired by [lexopt](https://crates.io/crates/lexopt), `lexarg` simplifies the formula down
//! further so it can be used for CLI plugin systems.
//!
//! ## Example
//! ```no_run
//! struct Args {
//!     thing: String,
//!     number: u32,
//!     shout: bool,
//! }
//!
//! fn parse_args() -> Result<Args, &'static str> {
//!     use lexarg::Arg::*;
//!
//!     let mut thing = None;
//!     let mut number = 1;
//!     let mut shout = false;
//!     let mut raw = std::env::args_os().collect::<Vec<_>>();
//!     let mut parser = lexarg::Parser::new(&raw);
//!     parser.bin();
//!     while let Some(arg) = parser.next() {
//!         match arg {
//!             Short('n') | Long("number") => {
//!                 number = parser
//!                     .flag_value().ok_or("`--number` requires a value")?
//!                     .to_str().ok_or("invalid number")?
//!                     .parse().map_err(|_e| "invalid number")?;
//!             }
//!             Long("shout") => {
//!                 shout = true;
//!             }
//!             Value(val) if thing.is_none() => {
//!                 thing = Some(val.to_str().ok_or("invalid number")?);
//!             }
//!             Long("help") => {
//!                 println!("Usage: hello [-n|--number=NUM] [--shout] THING");
//!                 std::process::exit(0);
//!             }
//!             _ => {
//!                 return Err("unexpected argument");
//!             }
//!         }
//!     }
//!
//!     Ok(Args {
//!         thing: thing.ok_or("missing argument THING")?.to_owned(),
//!         number,
//!         shout,
//!     })
//! }
//!
//! fn main() -> Result<(), String> {
//!     let args = parse_args()?;
//!     let mut message = format!("Hello {}", args.thing);
//!     if args.shout {
//!         message = message.to_uppercase();
//!     }
//!     for _ in 0..args.number {
//!         println!("{}", message);
//!     }
//!     Ok(())
//! }
//! ```

#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![warn(missing_docs, missing_debug_implementations, elided_lifetimes_in_paths)]

mod ext;

use std::ffi::OsStr;

use ext::OsStrExt as _;

/// A parser for command line arguments.
#[derive(Debug, Clone)]
pub struct Parser<'a> {
    raw: &'a dyn RawArgs,
    current: usize,
    state: Option<State<'a>>,
    was_attached: bool,
}

impl<'a> Parser<'a> {
    /// Create a parser from an iterator. This is useful for testing among other things.
    ///
    /// The first item from the iterator **must** be the binary name, as from [`std::env::args_os`].
    ///
    /// The iterator is consumed immediately.
    ///
    /// # Example
    /// ```
    /// let args = ["myapp", "-n", "10", "./foo.bar"];
    /// let mut parser = lexarg::Parser::new(&&args[1..]);
    /// ```
    pub fn new(raw: &'a dyn RawArgs) -> Self {
        Parser {
            raw,
            current: 0,
            state: None,
            was_attached: false,
        }
    }

    /// Extract the binary name before parsing [`Arg`]s
    ///
    /// # Panic
    ///
    /// Will panic if `next` has been called
    pub fn bin(&mut self) -> Option<&'a OsStr> {
        assert_eq!(self.current, 0);
        self.next_raw()
    }

    /// Get the next option or positional argument.
    ///
    /// A return value of `Ok(None)` means the command line has been exhausted.
    ///
    /// Options that are not valid unicode are transformed with replacement
    /// characters as by [`String::from_utf8_lossy`].
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<Arg<'a>> {
        // Always reset
        self.was_attached = false;

        match self.state {
            Some(State::PendingValue(attached)) => {
                // Last time we got `--long=value`, and `value` hasn't been used.
                self.state = None;
                self.current += 1;
                Some(Arg::Unexpected(attached))
            }
            Some(State::PendingShorts(valid, invalid, index)) => {
                // We're somewhere inside a -abc chain. Because we're in .next(), not .flag_value(), we
                // can assume that the next character is another option.
                let unparsed = &valid[index..];
                let mut char_indices = unparsed.char_indices();
                if let Some((0, short)) = char_indices.next() {
                    if matches!(short, '=' | '-') {
                        let arg = self
                            .raw
                            .get(self.current)
                            .expect("`current` is valid if state is `Shorts`");
                        // SAFETY: everything preceding `index` were a short flags, making them valid UTF-8
                        let unexpected_index = if index == 1 {
                            0
                        } else if short == '=' {
                            index + 1
                        } else {
                            index
                        };
                        let unexpected = unsafe { ext::split_at(arg, unexpected_index) }.1;

                        self.state = None;
                        self.current += 1;
                        Some(Arg::Unexpected(unexpected))
                    } else {
                        if let Some((offset, _)) = char_indices.next() {
                            let next_index = index + offset;
                            // Might have more flags
                            self.state = Some(State::PendingShorts(valid, invalid, next_index));
                        } else {
                            // No more flags
                            if invalid.is_empty() {
                                self.state = None;
                                self.current += 1;
                            } else {
                                self.state = Some(State::PendingValue(invalid));
                            }
                        }
                        Some(Arg::Short(short))
                    }
                } else {
                    debug_assert_ne!(invalid, "");
                    if index == 1 {
                        let arg = self
                            .raw
                            .get(self.current)
                            .expect("`current` is valid if state is `Shorts`");
                        self.state = None;
                        self.current += 1;
                        Some(Arg::Unexpected(arg))
                    } else {
                        self.state = None;
                        self.current += 1;
                        Some(Arg::Unexpected(invalid))
                    }
                }
            }
            Some(State::Escaped) => {
                self.state = Some(State::Escaped);
                self.next_raw().map(Arg::Value)
            }
            None => {
                let arg = self.raw.get(self.current)?;
                if arg == "--" {
                    self.state = Some(State::Escaped);
                    self.current += 1;
                    Some(Arg::Escape)
                } else if arg == "-" {
                    self.state = None;
                    self.current += 1;
                    Some(Arg::Value(arg))
                } else if let Some(long) = arg.strip_prefix("--") {
                    let (name, value) = long
                        .split_once("=")
                        .map(|(n, v)| (n, Some(v)))
                        .unwrap_or((long, None));
                    if name.is_empty() {
                        self.state = None;
                        self.current += 1;
                        Some(Arg::Unexpected(arg))
                    } else if let Ok(name) = name.try_str() {
                        if let Some(value) = value {
                            self.state = Some(State::PendingValue(value));
                        } else {
                            self.state = None;
                            self.current += 1;
                        }
                        Some(Arg::Long(name))
                    } else {
                        self.state = None;
                        self.current += 1;
                        Some(Arg::Unexpected(arg))
                    }
                } else if arg.starts_with("-") {
                    let (valid, invalid) = split_nonutf8_once(arg);
                    let invalid = invalid.unwrap_or_default();
                    self.state = Some(State::PendingShorts(valid, invalid, 1));
                    self.next()
                } else {
                    self.state = None;
                    self.current += 1;
                    Some(Arg::Value(arg))
                }
            }
        }
    }

    /// Get a flag's value
    ///
    /// This function should normally be called right after seeing a flag that expects a value;
    /// positional arguments should be collected with [`Parser::next()`].
    ///
    /// A value is collected even if it looks like an option (i.e., starts with `-`).
    ///
    /// `None` is returned if there is not another applicable flag value, including:
    /// - No more arguments are present
    /// - `--` was encountered, meaning all remaining arguments are positional
    /// - Being called again when the first value was attached (e.g. `--hello=world`)
    pub fn flag_value(&mut self) -> Option<&'a OsStr> {
        if let Some(value) = self.next_attached_value() {
            self.was_attached = true;
            return Some(value);
        }

        if !self.was_attached {
            return self.next_value();
        }

        None
    }

    fn next_attached_value(&mut self) -> Option<&'a OsStr> {
        match self.state? {
            State::PendingValue(attached) => {
                self.state = None;
                self.current += 1;
                Some(attached)
            }
            State::PendingShorts(_, _, index) => {
                let arg = self
                    .raw
                    .get(self.current)
                    .expect("`current` is valid if state is `Shorts`");
                self.state = None;
                self.current += 1;
                if index == arg.len() {
                    None
                } else {
                    // SAFETY: everything preceding `index` were a short flags, making them valid UTF-8
                    let remainder = unsafe { ext::split_at(arg, index) }.1;
                    let remainder = remainder.split_once("=").map(|s| s.1).unwrap_or(remainder);
                    Some(remainder)
                }
            }
            State::Escaped => {
                self.state = Some(State::Escaped);
                None
            }
        }
    }

    fn next_value(&mut self) -> Option<&'a OsStr> {
        if self.state == Some(State::Escaped) {
            // Escaped values are positional-only
            return None;
        }

        let next = self.next_raw()?;

        if next == "--" {
            self.state = Some(State::Escaped);
            None
        } else {
            Some(next)
        }
    }

    fn next_raw(&mut self) -> Option<&'a OsStr> {
        let next = self.raw.get(self.current)?;
        self.current += 1;
        Some(next)
    }

    #[cfg(test)]
    fn has_pending(&self) -> bool {
        self.state.as_ref().map(State::has_pending).unwrap_or(false)
    }
}

/// Accessor for unparsed arguments
pub trait RawArgs: std::fmt::Debug + private::Sealed {
    /// Returns a reference to an element or subslice depending on the type of index.
    ///
    /// - If given a position, returns a reference to the element at that position or None if out
    ///   of bounds.
    /// - If given a range, returns the subslice corresponding to that range, or None if out
    ///   of bounds.
    fn get(&self, index: usize) -> Option<&OsStr>;

    /// Returns the number of elements in the slice.
    fn len(&self) -> usize;

    /// Returns `true` if the slice has a length of 0.
    fn is_empty(&self) -> bool;
}

impl<const C: usize, S> RawArgs for [S; C]
where
    S: AsRef<OsStr> + std::fmt::Debug,
{
    #[inline]
    fn get(&self, index: usize) -> Option<&OsStr> {
        self.as_slice().get(index).map(|s| s.as_ref())
    }

    #[inline]
    fn len(&self) -> usize {
        C
    }

    #[inline]
    fn is_empty(&self) -> bool {
        C != 0
    }
}

impl<S> RawArgs for &'_ [S]
where
    S: AsRef<OsStr> + std::fmt::Debug,
{
    #[inline]
    fn get(&self, index: usize) -> Option<&OsStr> {
        (*self).get(index).map(|s| s.as_ref())
    }

    #[inline]
    fn len(&self) -> usize {
        (*self).len()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        (*self).is_empty()
    }
}

impl<S> RawArgs for Vec<S>
where
    S: AsRef<OsStr> + std::fmt::Debug,
{
    #[inline]
    fn get(&self, index: usize) -> Option<&OsStr> {
        self.as_slice().get(index).map(|s| s.as_ref())
    }

    #[inline]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum State<'a> {
    /// We have a value left over from --option=value.
    PendingValue(&'a OsStr),
    /// We're in the middle of -abc.
    ///
    /// On Windows and other non-UTF8-OsString platforms this Vec should
    /// only ever contain valid UTF-8 (and could instead be a String).
    PendingShorts(&'a str, &'a OsStr, usize),
    /// We saw -- and know no more options are coming.
    Escaped,
}

impl<'a> State<'a> {
    #[cfg(test)]
    fn has_pending(&self) -> bool {
        match self {
            Self::PendingValue(_) | Self::PendingShorts(_, _, _) => true,
            Self::Escaped => false,
        }
    }
}

/// A command line argument found by [`Parser`], either an option or a positional argument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Arg<'a> {
    /// A short option, e.g. `Short('q')` for `-q`.
    Short(char),
    /// A long option, e.g. `Long("verbose")` for `--verbose`. (The dashes are not included.)
    Long(&'a str),
    /// A positional argument, e.g. `/dev/null`.
    Value(&'a OsStr),
    /// Marks the following values have been escaped with `--`
    Escape,
    /// User passed something in that doesn't work
    Unexpected(&'a OsStr),
}

fn split_nonutf8_once(b: &OsStr) -> (&str, Option<&OsStr>) {
    match b.try_str() {
        Ok(s) => (s, None),
        Err(err) => {
            // SAFETY: `char_indices` ensures `index` is at a valid UTF-8 boundary
            let (valid, after_valid) = unsafe { ext::split_at(b, err.valid_up_to()) };
            let valid = valid.try_str().unwrap();
            (valid, Some(after_valid))
        }
    }
}

mod private {
    use super::*;

    pub trait Sealed {}
    impl<const C: usize, S> Sealed for [S; C] where S: AsRef<OsStr> + std::fmt::Debug {}
    impl<S> Sealed for &'_ [S] where S: AsRef<OsStr> + std::fmt::Debug {}
    impl<S> Sealed for Vec<S> where S: AsRef<OsStr> + std::fmt::Debug {}
}

#[cfg(test)]
mod tests {
    use super::Arg::*;
    use super::*;

    #[test]
    fn test_basic() {
        let mut p = Parser::new(&["-n", "10", "foo", "-", "--", "baz", "-qux"]);
        assert_eq!(p.next().unwrap(), Short('n'));
        assert_eq!(p.flag_value().unwrap(), "10");
        assert_eq!(p.next().unwrap(), Value(OsStr::new("foo")));
        assert_eq!(p.next().unwrap(), Value(OsStr::new("-")));
        assert_eq!(p.next().unwrap(), Escape);
        assert_eq!(p.next().unwrap(), Value(OsStr::new("baz")));
        assert_eq!(p.next().unwrap(), Value(OsStr::new("-qux")));
        assert_eq!(p.next(), None);
        assert_eq!(p.next(), None);
        assert_eq!(p.next(), None);
    }

    #[test]
    fn test_combined() {
        let mut p = Parser::new(&["-abc", "-fvalue", "-xfvalue"]);
        assert_eq!(p.next().unwrap(), Short('a'));
        assert_eq!(p.next().unwrap(), Short('b'));
        assert_eq!(p.next().unwrap(), Short('c'));
        assert_eq!(p.next().unwrap(), Short('f'));
        assert_eq!(p.flag_value().unwrap(), "value");
        assert_eq!(p.next().unwrap(), Short('x'));
        assert_eq!(p.next().unwrap(), Short('f'));
        assert_eq!(p.flag_value().unwrap(), "value");
        assert_eq!(p.next(), None);
    }

    #[test]
    fn test_long() {
        let mut p = Parser::new(&["--foo", "--bar=qux", "--foobar=qux=baz"]);
        assert_eq!(p.next().unwrap(), Long("foo"));
        assert_eq!(p.next().unwrap(), Long("bar"));
        assert_eq!(p.flag_value().unwrap(), "qux");
        assert_eq!(p.flag_value(), None);
        assert_eq!(p.next().unwrap(), Long("foobar"));
        assert_eq!(p.next().unwrap(), Unexpected(OsStr::new("qux=baz")));
        assert_eq!(p.next(), None);
    }

    #[test]
    fn test_dash_args() {
        // "--" should indicate the end of the options
        let mut p = Parser::new(&["-x", "--", "-y"]);
        assert_eq!(p.next().unwrap(), Short('x'));
        assert_eq!(p.next().unwrap(), Escape);
        assert_eq!(p.next().unwrap(), Value(OsStr::new("-y")));
        assert_eq!(p.next(), None);

        // ...even if it's an argument of an option
        let mut p = Parser::new(&["-x", "--", "-y"]);
        assert_eq!(p.next().unwrap(), Short('x'));
        assert_eq!(p.flag_value(), None);
        assert_eq!(p.next().unwrap(), Value(OsStr::new("-y")));
        assert_eq!(p.next(), None);

        // "-" is a valid value that should not be treated as an option
        let mut p = Parser::new(&["-x", "-", "-y"]);
        assert_eq!(p.next().unwrap(), Short('x'));
        assert_eq!(p.next().unwrap(), Value(OsStr::new("-")));
        assert_eq!(p.next().unwrap(), Short('y'));
        assert_eq!(p.next(), None);

        // '-' is a silly and hard to use short option, but other parsers treat
        // it like an option in this position
        let mut p = Parser::new(&["-x-y"]);
        assert_eq!(p.next().unwrap(), Short('x'));
        assert_eq!(p.next().unwrap(), Unexpected(OsStr::new("-y")));
        assert_eq!(p.next(), None);
    }

    #[test]
    fn test_missing_value() {
        let mut p = Parser::new(&["-o"]);
        assert_eq!(p.next().unwrap(), Short('o'));
        assert_eq!(p.flag_value(), None);

        let mut q = Parser::new(&["--out"]);
        assert_eq!(q.next().unwrap(), Long("out"));
        assert_eq!(q.flag_value(), None);

        let args: [&OsStr; 0] = [];
        let mut r = Parser::new(&args);
        assert_eq!(r.flag_value(), None);
    }

    #[test]
    fn test_weird_args() {
        let mut p = Parser::new(&[
            "--=", "--=3", "-", "-x", "--", "-", "-x", "--", "", "-", "-x",
        ]);
        assert_eq!(p.next().unwrap(), Unexpected(OsStr::new("--=")));
        assert_eq!(p.next().unwrap(), Unexpected(OsStr::new("--=3")));
        assert_eq!(p.next().unwrap(), Value(OsStr::new("-")));
        assert_eq!(p.next().unwrap(), Short('x'));
        assert_eq!(p.next().unwrap(), Escape);
        assert_eq!(p.next().unwrap(), Value(OsStr::new("-")));
        assert_eq!(p.next().unwrap(), Value(OsStr::new("-x")));
        assert_eq!(p.next().unwrap(), Value(OsStr::new("--")));
        assert_eq!(p.next().unwrap(), Value(OsStr::new("")));
        assert_eq!(p.next().unwrap(), Value(OsStr::new("-")));
        assert_eq!(p.next().unwrap(), Value(OsStr::new("-x")));
        assert_eq!(p.next(), None);

        let bad = bad_string("--=@");
        let args = [&bad];
        let mut q = Parser::new(&args);
        assert_eq!(q.next().unwrap(), Unexpected(OsStr::new(&bad)));

        let mut r = Parser::new(&[""]);
        assert_eq!(r.next().unwrap(), Value(OsStr::new("")));
    }

    #[test]
    fn test_unicode() {
        let mut p = Parser::new(&["-aµ", "--µ=10", "µ", "--foo=µ"]);
        assert_eq!(p.next().unwrap(), Short('a'));
        assert_eq!(p.next().unwrap(), Short('µ'));
        assert_eq!(p.next().unwrap(), Long("µ"));
        assert_eq!(p.flag_value().unwrap(), "10");
        assert_eq!(p.next().unwrap(), Value(OsStr::new("µ")));
        assert_eq!(p.next().unwrap(), Long("foo"));
        assert_eq!(p.flag_value().unwrap(), "µ");
    }

    #[cfg(any(unix, target_os = "wasi", windows))]
    #[test]
    fn test_mixed_invalid() {
        let args = [bad_string("--foo=@@@")];
        let mut p = Parser::new(&args);
        assert_eq!(p.next().unwrap(), Long("foo"));
        assert_eq!(p.flag_value().unwrap(), bad_string("@@@"));

        let args = [bad_string("-💣@@@")];
        let mut q = Parser::new(&args);
        assert_eq!(q.next().unwrap(), Short('💣'));
        assert_eq!(q.flag_value().unwrap(), bad_string("@@@"));

        let args = [bad_string("-f@@@")];
        let mut r = Parser::new(&args);
        assert_eq!(r.next().unwrap(), Short('f'));
        assert_eq!(r.next().unwrap(), Unexpected(&bad_string("@@@")));
        assert_eq!(r.next(), None);

        let args = [bad_string("--foo=bar=@@@")];
        let mut s = Parser::new(&args);
        assert_eq!(s.next().unwrap(), Long("foo"));
        assert_eq!(s.flag_value().unwrap(), bad_string("bar=@@@"));
    }

    #[cfg(any(unix, target_os = "wasi", windows))]
    #[test]
    fn test_separate_invalid() {
        let args = [bad_string("--foo"), bad_string("@@@")];
        let mut p = Parser::new(&args);
        assert_eq!(p.next().unwrap(), Long("foo"));
        assert_eq!(p.flag_value().unwrap(), bad_string("@@@"));
    }

    #[cfg(any(unix, target_os = "wasi", windows))]
    #[test]
    fn test_invalid_long_option() {
        let args = [bad_string("--@=10")];
        let mut p = Parser::new(&args);
        assert_eq!(p.next().unwrap(), Unexpected(&args[0]));
        assert_eq!(p.next(), None);

        let args = [bad_string("--@")];
        let mut p = Parser::new(&args);
        assert_eq!(p.next().unwrap(), Unexpected(&args[0]));
        assert_eq!(p.next(), None);
    }

    #[cfg(any(unix, target_os = "wasi", windows))]
    #[test]
    fn test_invalid_short_option() {
        let args = [bad_string("-@")];
        let mut p = Parser::new(&args);
        assert_eq!(p.next().unwrap(), Unexpected(&args[0]));
        assert_eq!(p.next(), None);
    }

    #[test]
    fn short_opt_equals_sign() {
        let mut p = Parser::new(&["-a=b"]);
        assert_eq!(p.next().unwrap(), Short('a'));
        assert_eq!(p.flag_value().unwrap(), OsStr::new("b"));
        assert_eq!(p.next(), None);

        let mut p = Parser::new(&["-a=b", "c"]);
        assert_eq!(p.next().unwrap(), Short('a'));
        assert_eq!(p.flag_value().unwrap(), OsStr::new("b"));
        assert_eq!(p.flag_value(), None);
        assert_eq!(p.next().unwrap(), Value(OsStr::new("c")));
        assert_eq!(p.next(), None);

        let mut p = Parser::new(&["-a=b"]);
        assert_eq!(p.next().unwrap(), Short('a'));
        assert_eq!(p.next().unwrap(), Unexpected(OsStr::new("b")));
        assert_eq!(p.next(), None);

        let mut p = Parser::new(&["-a="]);
        assert_eq!(p.next().unwrap(), Short('a'));
        assert_eq!(p.flag_value().unwrap(), OsStr::new(""));
        assert_eq!(p.next(), None);

        let mut p = Parser::new(&["-a="]);
        assert_eq!(p.next().unwrap(), Short('a'));
        assert_eq!(p.next().unwrap(), Unexpected(OsStr::new("")));
        assert_eq!(p.next(), None);

        let mut p = Parser::new(&["-="]);
        assert_eq!(p.next().unwrap(), Unexpected(OsStr::new("-=")));
        assert_eq!(p.next(), None);

        let mut p = Parser::new(&["-=a"]);
        assert_eq!(p.next().unwrap(), Unexpected(OsStr::new("-=a")));
        assert_eq!(p.next(), None);
    }

    #[cfg(any(unix, target_os = "wasi", windows))]
    #[test]
    fn short_opt_equals_sign_invalid() {
        let bad = bad_string("@");
        let args = [bad_string("-a=@")];
        let mut p = Parser::new(&args);
        assert_eq!(p.next().unwrap(), Short('a'));
        assert_eq!(p.flag_value().unwrap(), bad_string("@"));
        assert_eq!(p.next(), None);

        let mut p = Parser::new(&args);
        assert_eq!(p.next().unwrap(), Short('a'));
        assert_eq!(p.next().unwrap(), Unexpected(&bad));
        assert_eq!(p.next(), None);
    }

    /// Transform @ characters into invalid unicode.
    fn bad_string(text: &str) -> std::ffi::OsString {
        #[cfg(any(unix, target_os = "wasi"))]
        {
            #[cfg(unix)]
            use std::os::unix::ffi::OsStringExt;
            #[cfg(target_os = "wasi")]
            use std::os::wasi::ffi::OsStringExt;
            let mut text = text.as_bytes().to_vec();
            for ch in &mut text {
                if *ch == b'@' {
                    *ch = b'\xFF';
                }
            }
            std::ffi::OsString::from_vec(text)
        }
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStringExt;
            let mut out = Vec::new();
            for ch in text.chars() {
                if ch == '@' {
                    out.push(0xD800);
                } else {
                    let mut buf = [0; 2];
                    out.extend(&*ch.encode_utf16(&mut buf));
                }
            }
            std::ffi::OsString::from_wide(&out)
        }
        #[cfg(not(any(unix, target_os = "wasi", windows)))]
        {
            if text.contains('@') {
                unimplemented!("Don't know how to create invalid OsStrings on this platform");
            }
            text.into()
        }
    }

    /// Basic exhaustive testing of short combinations of "interesting"
    /// arguments. They should not panic, not hang, and pass some checks.
    ///
    /// The advantage compared to full fuzzing is that it runs on all platforms
    /// and together with the other tests. cargo-fuzz doesn't work on Windows
    /// and requires a special incantation.
    ///
    /// A disadvantage is that it's still limited by arguments I could think of
    /// and only does very short sequences. Another is that it's bad at
    /// reporting failure, though the println!() helps.
    ///
    /// This test takes a while to run.
    #[test]
    fn basic_fuzz() {
        #[cfg(any(windows, unix, target_os = "wasi"))]
        const VOCABULARY: &[&str] = &[
            "", "-", "--", "---", "a", "-a", "-aa", "@", "-@", "-a@", "-@a", "--a", "--@", "--a=a",
            "--a=", "--a=@", "--@=a", "--=", "--=@", "--=a", "-@@", "-a=a", "-a=", "-=", "-a-",
        ];
        #[cfg(not(any(windows, unix, target_os = "wasi")))]
        const VOCABULARY: &[&str] = &[
            "", "-", "--", "---", "a", "-a", "-aa", "--a", "--a=a", "--a=", "--=", "--=a", "-a=a",
            "-a=", "-=", "-a-",
        ];
        let args: [&OsStr; 0] = [];
        exhaust(Parser::new(&args), vec![]);
        let vocabulary: Vec<std::ffi::OsString> =
            VOCABULARY.iter().map(|&s| bad_string(s)).collect();
        let mut permutations = vec![vec![]];
        for _ in 0..3 {
            let mut new = Vec::new();
            for old in permutations {
                for word in &vocabulary {
                    let mut extended = old.clone();
                    extended.push(word);
                    new.push(extended);
                }
            }
            permutations = new;
            for permutation in &permutations {
                println!("Starting {:?}", permutation);
                let p = Parser::new(permutation);
                exhaust(p, vec![]);
            }
        }
    }

    /// Run many sequences of methods on a Parser.
    fn exhaust(parser: Parser<'_>, path: Vec<String>) {
        if path.len() > 100 {
            panic!("Stuck in loop: {:?}", path);
        }

        if parser.has_pending() {
            {
                let mut parser = parser.clone();
                let next = parser.next();
                assert!(
                    matches!(next, Some(Unexpected(_)) | Some(Short(_))),
                    "{next:?} via {path:?}",
                );
                let mut path = path.clone();
                path.push(format!("pending-next-{:?}", next));
                exhaust(parser, path);
            }

            {
                let mut parser = parser.clone();
                let next = parser.flag_value();
                assert!(next.is_some(), "{next:?} via {path:?}",);
                let mut path = path;
                path.push(format!("pending-value-{:?}", next));
                exhaust(parser, path);
            }
        } else {
            {
                let mut parser = parser.clone();
                let next = parser.next();
                match &next {
                    None => {
                        assert!(
                            matches!(parser.state, None | Some(State::Escaped)),
                            "{next:?} via {path:?}",
                        );
                        assert_eq!(parser.current, parser.raw.len(), "{next:?} via {path:?}",);
                    }
                    _ => {
                        let mut path = path.clone();
                        path.push(format!("next-{:?}", next));
                        exhaust(parser, path)
                    }
                }
            }

            {
                let mut parser = parser.clone();
                let next = parser.flag_value();
                match &next {
                    None => {
                        assert!(
                            matches!(parser.state, None | Some(State::Escaped)),
                            "{next:?} via {path:?}",
                        );
                        if parser.state.is_none() && !parser.was_attached {
                            assert_eq!(parser.current, parser.raw.len(), "{next:?} via {path:?}",);
                        }
                    }
                    Some(_) => {
                        assert!(
                            matches!(parser.state, None | Some(State::Escaped)),
                            "{next:?} via {path:?}",
                        );
                        let mut path = path;
                        path.push(format!("value-{:?}", next));
                        exhaust(parser, path);
                    }
                }
            }
        }
    }
}
