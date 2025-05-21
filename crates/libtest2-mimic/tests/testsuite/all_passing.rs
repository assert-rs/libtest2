fn test_cmd() -> snapbox::cmd::Command {
    static BIN: once_cell::sync::OnceCell<(std::path::PathBuf, std::path::PathBuf)> =
        once_cell::sync::OnceCell::new();
    let (bin, current_dir) = BIN.get_or_init(|| {
        let package_root = crate::util::new_test(
            r#"
fn main() {
    use libtest2_mimic::Trial;
    libtest2_mimic::Harness::with_env()
        .cases(vec![
            Trial::test("foo", |_| Ok(())),
            Trial::test("bar", |_| Ok(())),
            Trial::test("barro", |_| Ok(())),
        ])
        .main();
}
"#,
            false,
        );
        let bin = crate::util::compile_test(&package_root);
        (bin, package_root)
    });
    snapbox::cmd::Command::new(bin).current_dir(current_dir)
}

fn check(args: &[&str], single: &str, parallel: &str) {
    test_cmd()
        .args(args)
        .args(["--test-threads", "1"])
        .assert()
        .success()
        .stdout_matches(single);
    test_cmd()
        .args(args)
        .assert()
        .success()
        .stdout_matches(parallel);
}

#[test]
fn normal() {
    check(
        &[],
        r#"
running 3 tests
test bar   ... ok
test barro ... ok
test foo   ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 filtered out; finished in [..]s

"#,
        r#"
running 3 tests
...

test result: ok. 3 passed; 0 failed; 0 ignored; 0 filtered out; finished in [..]s

"#,
    );
}

#[test]
fn filter_one() {
    check(
        &["foo"],
        r#"
running 1 test
test foo ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 2 filtered out; finished in [..]s

"#,
        r#"
running 1 test
test foo ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 2 filtered out; finished in [..]s

"#,
    );
}

#[test]
fn filter_two() {
    check(
        &["bar"],
        r#"
running 2 tests
test bar   ... ok
test barro ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 1 filtered out; finished in [..]s

"#,
        r#"
running 2 tests
...

test result: ok. 2 passed; 0 failed; 0 ignored; 1 filtered out; finished in [..]s

"#,
    );
}

#[test]
fn filter_exact() {
    check(
        &["bar", "--exact"],
        r#"
running 1 test
test bar ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 2 filtered out; finished in [..]s

"#,
        r#"
running 1 test
test bar ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 2 filtered out; finished in [..]s

"#,
    );
}

#[test]
fn filter_two_and_skip() {
    check(
        &["--skip", "barro", "bar"],
        r#"
running 1 test
test bar ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 2 filtered out; finished in [..]s

"#,
        r#"
running 1 test
test bar ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 2 filtered out; finished in [..]s

"#,
    );
}

#[test]
fn skip_nothing() {
    check(
        &["--skip", "peter"],
        r#"
running 3 tests
test bar   ... ok
test barro ... ok
test foo   ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 filtered out; finished in [..]s

"#,
        r#"
running 3 tests
...

test result: ok. 3 passed; 0 failed; 0 ignored; 0 filtered out; finished in [..]s

"#,
    );
}

#[test]
fn skip_two() {
    check(
        &["--skip", "bar"],
        r#"
running 1 test
test foo ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 2 filtered out; finished in [..]s

"#,
        r#"
running 1 test
test foo ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 2 filtered out; finished in [..]s

"#,
    );
}

#[test]
fn skip_exact() {
    check(
        &["--exact", "--skip", "bar"],
        r#"
running 2 tests
test barro ... ok
test foo   ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 1 filtered out; finished in [..]s

"#,
        r#"
running 2 tests
...

test result: ok. 2 passed; 0 failed; 0 ignored; 1 filtered out; finished in [..]s

"#,
    );
}

#[test]
fn terse_output() {
    check(
        &["--quiet", "--skip", "foo"],
        r#"
running 2 tests
..
test result: ok. 2 passed; 0 failed; 0 ignored; 1 filtered out; finished in [..]s

"#,
        r#"
running 2 tests
..
test result: ok. 2 passed; 0 failed; 0 ignored; 1 filtered out; finished in [..]s

"#,
    );
}
