#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(clippy::print_stderr)]
#![warn(clippy::print_stdout)]

use std::iter::FromIterator;

use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

#[allow(missing_docs)]
#[proc_macro_attribute]
pub fn test(attribute: TokenStream, item: TokenStream) -> TokenStream {
    generate("test", "libtest2", attribute, item)
}

#[allow(missing_docs)]
#[proc_macro_attribute]
pub fn main(attribute: TokenStream, item: TokenStream) -> TokenStream {
    generate("main", "libtest2", attribute, item)
}

/// Expands to `(file!(), line!(), column!(), <name length in chars>)` for a test's name
#[doc(hidden)]
#[proc_macro]
pub fn _source(input: TokenStream) -> TokenStream {
    let mut input = input.into_iter();
    let name = match (input.next(), input.next()) {
        (Some(TokenTree::Ident(name)), None) => name,
        _ => panic!("expected the test function's name"),
    };

    // `line!()`/`column!()` report where they are invoked, so every token takes the name's span
    // to point at the `fn` name instead of the `#[test]` attribute
    let span = name.span();
    let punct = |ch, spacing| {
        let mut punct = Punct::new(ch, spacing);
        punct.set_span(span);
        TokenTree::Punct(punct)
    };

    let mut fields = TokenStream::new();
    for builtin in ["file", "line", "column"] {
        let mut args = Group::new(Delimiter::Parenthesis, TokenStream::new());
        args.set_span(span);
        fields.extend([
            punct(':', Spacing::Joint),
            punct(':', Spacing::Alone),
            TokenTree::Ident(Ident::new("core", span)),
            punct(':', Spacing::Joint),
            punct(':', Spacing::Alone),
            TokenTree::Ident(Ident::new(builtin, span)),
            punct('!', Spacing::Alone),
            TokenTree::Group(args),
            punct(',', Spacing::Alone),
        ]);
    }
    let mut len = Literal::usize_suffixed(name.to_string().chars().count());
    len.set_span(span);
    fields.extend([TokenTree::Literal(len)]);

    let mut tuple = Group::new(Delimiter::Parenthesis, fields);
    tuple.set_span(span);
    TokenStream::from_iter([TokenTree::Group(tuple)])
}

/// Generates the equivalent of this Rust code as a `TokenStream`:
///
/// ```nocompile
/// ::libtest2::__support::test_parse!(#[ctor] fn foo() { ... });
/// ```
fn generate(
    macro_type: &str,
    macro_crate: &str,
    attribute: TokenStream,
    item: TokenStream,
) -> TokenStream {
    let mut inner = TokenStream::new();

    // Search for crate_path in attributes
    let mut crate_path = None;
    let mut tokens = attribute.clone().into_iter().peekable();

    while let Some(token) = tokens.next() {
        if let TokenTree::Ident(ident) = &token {
            if ident.to_string() == "crate_path" {
                // Look for =
                if let Some(TokenTree::Punct(punct)) = tokens.next() {
                    if punct.as_char() == '=' {
                        // Collect tokens until comma or end
                        let mut path = TokenStream::new();
                        while let Some(token) = tokens.peek() {
                            match token {
                                TokenTree::Punct(p) if p.as_char() == ',' => {
                                    tokens.next();
                                    break;
                                }
                                _ => {
                                    path.extend(std::iter::once(tokens.next().unwrap()));
                                }
                            }
                        }
                        crate_path = Some(path);
                        break;
                    }
                }
            }
        }
    }

    if attribute.is_empty() {
        // #[test]
        inner.extend([
            TokenTree::Punct(Punct::new('#', Spacing::Alone)),
            TokenTree::Group(Group::new(
                Delimiter::Bracket,
                TokenStream::from_iter([TokenTree::Ident(Ident::new(
                    macro_type,
                    Span::call_site(),
                ))]),
            )),
        ]);
    } else {
        inner.extend([
            TokenTree::Punct(Punct::new('#', Spacing::Alone)),
            TokenTree::Group(Group::new(
                Delimiter::Bracket,
                TokenStream::from_iter([
                    TokenTree::Ident(Ident::new(macro_type, Span::call_site())),
                    TokenTree::Group(Group::new(Delimiter::Parenthesis, attribute)),
                ]),
            )),
        ]);
    }

    inner.extend(item);

    let mut invoke = crate_path.unwrap_or_else(|| {
        TokenStream::from_iter([
            TokenTree::Punct(Punct::new(':', Spacing::Joint)),
            TokenTree::Punct(Punct::new(':', Spacing::Alone)),
            TokenTree::Ident(Ident::new(macro_crate, Span::call_site())),
        ])
    });

    invoke.extend([
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Ident(Ident::new("_private", Span::call_site())),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Ident(Ident::new(
            &format!("{macro_type}_parse"),
            Span::call_site(),
        )),
        TokenTree::Punct(Punct::new('!', Spacing::Alone)),
        TokenTree::Group(Group::new(Delimiter::Parenthesis, inner)),
        TokenTree::Punct(Punct::new(';', Spacing::Alone)),
    ]);

    invoke
}
