use libtest2::Case;
use libtest2::FnCase;
use libtest2::RunError;
use libtest2::RunResult;

fn main() -> std::io::Result<()> {
    let harness = ::libtest2_harness::Harness::new();
    let harness = harness.with_env()?;
    let harness = match harness.parse() {
        Ok(harness) => harness,
        Err(err) => {
            eprintln!("{err}");
            ::std::process::exit(1);
        }
    };
    let tests = collect_tests()?;
    let harness = harness.discover(tests)?;
    if !harness.run()? {
        std::process::exit(libtest2_harness::ERROR_EXIT_CODE)
    }

    Ok(())
}

/// Creates one test for each `.rs` file in the current directory or
/// sub-directories of the current directory.
fn collect_tests() -> std::io::Result<Vec<Box<dyn Case>>> {
    fn visit_dir(path: &std::path::Path, tests: &mut Vec<Box<dyn Case>>) -> std::io::Result<()> {
        let current_dir = std::env::current_dir()?;
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let file_type = entry.file_type()?;

            // Handle files
            let path = entry.path();
            if file_type.is_file() {
                if path.extension() == Some(std::ffi::OsStr::new("rs")) {
                    let name = match path.strip_prefix(&current_dir) {
                        Ok(path) => path,
                        Err(_) => {
                            continue;
                        }
                    }
                    .as_os_str()
                    .to_string_lossy()
                    .into_owned();

                    let test = FnCase::test(name, move |_| check_file(&path));
                    tests.push(Box::new(test));
                }
            } else if file_type.is_dir() {
                // Handle directories
                visit_dir(&path, tests)?;
            }
        }

        Ok(())
    }

    // We recursively look for `.rs` files, starting from the current
    // directory.
    let mut tests = Vec::new();
    let current_dir = std::env::current_dir()?;
    visit_dir(&current_dir, &mut tests)?;

    Ok(tests)
}

/// Performs a couple of tidy tests.
fn check_file(path: &std::path::Path) -> RunResult {
    let content =
        std::fs::read(path).map_err(|e| RunError::fail(format_args!("Cannot read file: {e}")))?;

    // Check that the file is valid UTF-8
    let content = String::from_utf8(content)
        .map_err(|_| RunError::fail("The file's contents are not a valid UTF-8 string!"))?;

    // Check for `\r`: we only want `\n` line breaks!
    if content.contains('\r') {
        return Err(RunError::fail(
            "Contains '\\r' chars. Please use ' \\n' line breaks only!",
        ));
    }

    // Check for tab characters `\t`
    if content.contains('\t') {
        return Err(RunError::fail(
            "Contains tab characters ('\\t'). Indent with four spaces!",
        ));
    }

    // Check for too long lines
    if content.lines().any(|line| line.chars().count() > 100) {
        return Err(RunError::fail("Contains lines longer than 100 codepoints!"));
    }

    Ok(())
}
