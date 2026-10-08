use snapbox::str;

fn test_cmd() -> snapbox::cmd::Command {
    static BIN: once_cell_polyfill::sync::OnceLock<(std::path::PathBuf, std::path::PathBuf)> =
        once_cell_polyfill::sync::OnceLock::new();
    let (bin, current_dir) = BIN.get_or_init(|| {
        let package_root = crate::util::new_test(
            r#"
#[libtest2::main]
fn main() {}

#[libtest2::test]
fn foo(_context: &libtest2::TestContext) {}

mod some_module {
    #[libtest2::test]
    fn foo(_context: &libtest2::TestContext) {}

    #[libtest2::test]
    fn can_call_foo(context: &libtest2::TestContext) {
        foo(context);
    }
    
    #[libtest2::test]
    fn can_call_super_foo(context: &libtest2::TestContext) {
        super::foo(context);
    }
}

#[libtest2::test]
fn takes_context(_context: &libtest2::TestContext) {}

#[libtest2::test]
fn no_parameters() {}

#[libtest2::test]
fn takes_context_return_result(_context: &libtest2::TestContext) -> libtest2::RunResult {
    Ok(())
}

#[libtest2::test]
fn no_parameters_return_result() -> libtest2::RunResult {
    Ok(())
}

#[libtest2::test]
fn ignored_context(_: &libtest2::TestContext) {}

#[libtest2::test]
fn context_as_pattern(libtest2::TestContext { .. }: &libtest2::TestContext) {}
"#,
            false,
        );
        let bin = crate::util::compile_test(&package_root);
        (bin, package_root)
    });
    snapbox::cmd::Command::new(bin).current_dir(current_dir)
}

#[test]
fn check() {
    let data = str![[r#"

running 10 tests
test context_as_pattern              ... ok
test foo                             ... ok
test ignored_context                 ... ok
test no_parameters                   ... ok
test no_parameters_return_result     ... ok
test some_module::can_call_foo       ... ok
test some_module::can_call_super_foo ... ok
test some_module::foo                ... ok
test takes_context                   ... ok
test takes_context_return_result     ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 filtered out; finished in [..]s


"#]];

    test_cmd()
        .args(["--test-threads", "1"])
        .assert()
        .success()
        .stdout_eq(data);
}
