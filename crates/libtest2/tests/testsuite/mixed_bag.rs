use snapbox::prelude::*;
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
fn passed(_context: &libtest2::TestContext) {
}

#[libtest2::test]
fn failed(_context: &libtest2::TestContext) -> libtest2::RunResult {
    Err(libtest2::RunError::fail("was not a good boy"))
}

#[libtest2::test]
fn custom_error(context: &libtest2::TestContext) -> std::io::Result<()> {
    Err(std::io::Error::new(std::io::ErrorKind::Other, "I failed"))
}

#[libtest2::test]
fn later_passed(_context: &libtest2::TestContext) {
}

#[libtest2::test]
fn ignore_failed(context: &libtest2::TestContext) -> libtest2::RunResult {
    context.ignore_for("fails")?;
    Err(libtest2::RunError::fail("jumped too high"))
}

#[libtest2::test]
fn ignore_passed(context: &libtest2::TestContext) -> libtest2::RunResult {
    context.ignore_for("slow")?;
    Ok(())
}

#[libtest2::test]
fn later_ignore_failed(context: &libtest2::TestContext) -> libtest2::RunResult {
    context.ignore_for("fails")?;
    Err(libtest2::RunError::fail("broke neck"))
}

#[libtest2::test]
fn later_ignore_passed(context: &libtest2::TestContext) -> libtest2::RunResult {
    context.ignore_for("fails")?;
    Ok(())
}

#[libtest2::test]
#[ignore]
fn ignore_attribute(context: &libtest2::TestContext) -> libtest2::RunResult {
    Err(libtest2::RunError::fail("got lost blindly following the flock"))
}

#[libtest2::test]
#[ignore = "fails"]
fn ignore_attribute_reason(context: &libtest2::TestContext) -> libtest2::RunResult {
    Err(libtest2::RunError::fail("no honey"))
}

#[libtest2::test]
#[cfg(all())]
fn cfg_in(context: &libtest2::TestContext) {
}

#[libtest2::test]
#[cfg(any())]
fn cfged_out(context: &libtest2::TestContext) {
    panic!("I don't exist");
}
"#,
            false,
        );
        let bin = crate::util::compile_test(&package_root);
        (bin, package_root)
    });
    snapbox::cmd::Command::new(bin).current_dir(current_dir)
}

fn check(args: &[&str], code: i32, single: impl IntoData, parallel: impl IntoData) {
    test_cmd()
        .args(args)
        .args(["--test-threads", "1"])
        .assert()
        .code(code)
        .stdout_eq(single);
    test_cmd()
        .args(args)
        .assert()
        .code(code)
        .stdout_eq(parallel);
}

#[test]
fn normal() {
    check(
        &[],
        101,
        str![[r#"

running 11 tests
test cfg_in                  ... ok
test custom_error            ... FAILED
test failed                  ... FAILED
test ignore_attribute        ... ignored
test ignore_attribute_reason ... ignored
test ignore_failed           ... ignored
test ignore_passed           ... ignored
test later_ignore_failed     ... ignored
test later_ignore_passed     ... ignored
test later_passed            ... ok
test passed                  ... ok

failures:

---- custom_error ----
I failed

---- failed ----
was not a good boy


failures:
    custom_error
    failed

test result: FAILED. 3 passed; 2 failed; 6 ignored; 0 filtered out; finished in [..]s


"#]],
        str![[r#"

running 11 tests
...

failures:

---- custom_error ----
I failed

---- failed ----
was not a good boy


failures:
    custom_error
    failed

test result: FAILED. 3 passed; 2 failed; 6 ignored; 0 filtered out; finished in [..]s


"#]],
    );
}

#[test]
fn fail_fast() {
    check(
        &["--fail-fast"],
        101,
        str![[r#"

running 11 tests
test cfg_in                  ... ok
test custom_error            ... FAILED

failures:

---- custom_error ----
I failed


failures:
    custom_error

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 filtered out; finished in [..]s


"#]],
        str![[r#"
...

"#]],
    );
}

#[test]
fn test_mode() {
    check(
        &["--test"],
        101,
        str![[r#"

running 11 tests
test cfg_in                  ... ok
test custom_error            ... FAILED
test failed                  ... FAILED
test ignore_attribute        ... ignored
test ignore_attribute_reason ... ignored
test ignore_failed           ... ignored
test ignore_passed           ... ignored
test later_ignore_failed     ... ignored
test later_ignore_passed     ... ignored
test later_passed            ... ok
test passed                  ... ok

failures:

---- custom_error ----
I failed

---- failed ----
was not a good boy


failures:
    custom_error
    failed

test result: FAILED. 3 passed; 2 failed; 6 ignored; 0 filtered out; finished in [..]s


"#]],
        str![[r#"

running 11 tests
...

failures:

---- custom_error ----
I failed

---- failed ----
was not a good boy


failures:
    custom_error
    failed

test result: FAILED. 3 passed; 2 failed; 6 ignored; 0 filtered out; finished in [..]s


"#]],
    );
}

#[test]
fn bench_mode() {
    check(
        &["--bench"],
        101,
        str![[r#"

running 11 tests
test cfg_in                  ... ok
test custom_error            ... FAILED
test failed                  ... FAILED
test ignore_attribute        ... ignored
test ignore_attribute_reason ... ignored
test ignore_failed           ... ignored
test ignore_passed           ... ignored
test later_ignore_failed     ... ignored
test later_ignore_passed     ... ignored
test later_passed            ... ok
test passed                  ... ok

failures:

---- custom_error ----
I failed

---- failed ----
was not a good boy


failures:
    custom_error
    failed

test result: FAILED. 3 passed; 2 failed; 6 ignored; 0 filtered out; finished in [..]s


"#]],
        str![[r#"

running 11 tests
...

failures:

---- custom_error ----
I failed

---- failed ----
was not a good boy


failures:
    custom_error
    failed

test result: FAILED. 3 passed; 2 failed; 6 ignored; 0 filtered out; finished in [..]s


"#]],
    );
}

#[test]
fn list() {
    check(
        &["--list"],
        0,
        str![[r#"
cfg_in: test
custom_error: test
failed: test
ignore_attribute: test
ignore_attribute_reason: test
ignore_failed: test
ignore_passed: test
later_ignore_failed: test
later_ignore_passed: test
later_passed: test
passed: test

11 tests


"#]],
        str![[r#"
cfg_in: test
custom_error: test
failed: test
ignore_attribute: test
ignore_attribute_reason: test
ignore_failed: test
ignore_passed: test
later_ignore_failed: test
later_ignore_passed: test
later_passed: test
passed: test

11 tests


"#]],
    );
}

#[test]
fn list_ignored() {
    check(
        &["--list", "--ignored"],
        0,
        str![[r#"
cfg_in: test
custom_error: test
failed: test
ignore_attribute: test
ignore_attribute_reason: test
ignore_failed: test
ignore_passed: test
later_ignore_failed: test
later_ignore_passed: test
later_passed: test
passed: test

11 tests


"#]],
        str![[r#"
cfg_in: test
custom_error: test
failed: test
ignore_attribute: test
ignore_attribute_reason: test
ignore_failed: test
ignore_passed: test
later_ignore_failed: test
later_ignore_passed: test
later_passed: test
passed: test

11 tests


"#]],
    );
}

#[test]
fn list_with_filter() {
    check(
        &["--list", "a"],
        0,
        str![[r#"
failed: test
ignore_attribute: test
ignore_attribute_reason: test
ignore_failed: test
ignore_passed: test
later_ignore_failed: test
later_ignore_passed: test
later_passed: test
passed: test

9 tests


"#]],
        str![[r#"
failed: test
ignore_attribute: test
ignore_attribute_reason: test
ignore_failed: test
ignore_passed: test
later_ignore_failed: test
later_ignore_passed: test
later_passed: test
passed: test

9 tests


"#]],
    );
}

#[test]
fn list_with_specified_order() {
    check(
        &[
            "--list",
            "--exact",
            "later_passed",
            "failed",
            "passed",
            "ignore_passed",
        ],
        0,
        str![[r#"
failed: test
ignore_passed: test
later_passed: test
passed: test

4 tests


"#]],
        str![[r#"
failed: test
ignore_passed: test
later_passed: test
passed: test

4 tests


"#]],
    );
}

#[test]
fn include_ignored_normal() {
    check(
        &["--include-ignored"],
        101,
        str![[r#"

running 11 tests
test cfg_in                  ... ok
test custom_error            ... FAILED
test failed                  ... FAILED
test ignore_attribute        ... FAILED
test ignore_attribute_reason ... FAILED
test ignore_failed           ... FAILED
test ignore_passed           ... ok
test later_ignore_failed     ... FAILED
test later_ignore_passed     ... ok
test later_passed            ... ok
test passed                  ... ok

failures:

---- custom_error ----
I failed

---- failed ----
was not a good boy

---- ignore_attribute ----
got lost blindly following the flock

---- ignore_attribute_reason ----
no honey

---- ignore_failed ----
jumped too high

---- later_ignore_failed ----
broke neck


failures:
    custom_error
    failed
    ignore_attribute
    ignore_attribute_reason
    ignore_failed
    later_ignore_failed

test result: FAILED. 5 passed; 6 failed; 0 ignored; 0 filtered out; finished in [..]s


"#]],
        str![[r#"

running 11 tests
...

failures:

---- custom_error ----
I failed

---- failed ----
was not a good boy

---- ignore_attribute ----
got lost blindly following the flock

---- ignore_attribute_reason ----
no honey

---- ignore_failed ----
jumped too high

---- later_ignore_failed ----
broke neck


failures:
    custom_error
    failed
    ignore_attribute
    ignore_attribute_reason
    ignore_failed
    later_ignore_failed

test result: FAILED. 5 passed; 6 failed; 0 ignored; 0 filtered out; finished in [..]s


"#]],
    );
}

#[test]
fn include_ignored_test_filter() {
    check(
        &["--test", "--include-ignored", "a"],
        101,
        str![[r#"

running 9 tests
test failed                  ... FAILED
test ignore_attribute        ... FAILED
test ignore_attribute_reason ... FAILED
test ignore_failed           ... FAILED
test ignore_passed           ... ok
test later_ignore_failed     ... FAILED
test later_ignore_passed     ... ok
test later_passed            ... ok
test passed                  ... ok

failures:

---- failed ----
was not a good boy

---- ignore_attribute ----
got lost blindly following the flock

---- ignore_attribute_reason ----
no honey

---- ignore_failed ----
jumped too high

---- later_ignore_failed ----
broke neck


failures:
    failed
    ignore_attribute
    ignore_attribute_reason
    ignore_failed
    later_ignore_failed

test result: FAILED. 4 passed; 5 failed; 0 ignored; 2 filtered out; finished in [..]s


"#]],
        str![[r#"

running 9 tests
...

failures:

---- failed ----
was not a good boy

---- ignore_attribute ----
got lost blindly following the flock

---- ignore_attribute_reason ----
no honey

---- ignore_failed ----
jumped too high

---- later_ignore_failed ----
broke neck


failures:
    failed
    ignore_attribute
    ignore_attribute_reason
    ignore_failed
    later_ignore_failed

test result: FAILED. 4 passed; 5 failed; 0 ignored; 2 filtered out; finished in [..]s


"#]],
    );
}

#[test]
fn ignored_normal() {
    check(
        &["--ignored"],
        101,
        str![[r#"

running 11 tests
test cfg_in                  ... ok
test custom_error            ... FAILED
test failed                  ... FAILED
test ignore_attribute        ... FAILED
test ignore_attribute_reason ... FAILED
test ignore_failed           ... FAILED
test ignore_passed           ... ok
test later_ignore_failed     ... FAILED
test later_ignore_passed     ... ok
test later_passed            ... ok
test passed                  ... ok

failures:

---- custom_error ----
I failed

---- failed ----
was not a good boy

---- ignore_attribute ----
got lost blindly following the flock

---- ignore_attribute_reason ----
no honey

---- ignore_failed ----
jumped too high

---- later_ignore_failed ----
broke neck


failures:
    custom_error
    failed
    ignore_attribute
    ignore_attribute_reason
    ignore_failed
    later_ignore_failed

test result: FAILED. 5 passed; 6 failed; 0 ignored; 0 filtered out; finished in [..]s


"#]],
        str![[r#"

running 11 tests
...

failures:

---- custom_error ----
I failed

---- failed ----
was not a good boy

---- ignore_attribute ----
got lost blindly following the flock

---- ignore_attribute_reason ----
no honey

---- ignore_failed ----
jumped too high

---- later_ignore_failed ----
broke neck


failures:
    custom_error
    failed
    ignore_attribute
    ignore_attribute_reason
    ignore_failed
    later_ignore_failed

test result: FAILED. 5 passed; 6 failed; 0 ignored; 0 filtered out; finished in [..]s


"#]],
    );
}

#[test]
fn ignored_test_filter() {
    check(
        &["--test", "--ignored", "a"],
        101,
        str![[r#"

running 9 tests
test failed                  ... FAILED
test ignore_attribute        ... FAILED
test ignore_attribute_reason ... FAILED
test ignore_failed           ... FAILED
test ignore_passed           ... ok
test later_ignore_failed     ... FAILED
test later_ignore_passed     ... ok
test later_passed            ... ok
test passed                  ... ok

failures:

---- failed ----
was not a good boy

---- ignore_attribute ----
got lost blindly following the flock

---- ignore_attribute_reason ----
no honey

---- ignore_failed ----
jumped too high

---- later_ignore_failed ----
broke neck


failures:
    failed
    ignore_attribute
    ignore_attribute_reason
    ignore_failed
    later_ignore_failed

test result: FAILED. 4 passed; 5 failed; 0 ignored; 2 filtered out; finished in [..]s


"#]],
        str![[r#"

running 9 tests
...

failures:

---- failed ----
was not a good boy

---- ignore_attribute ----
got lost blindly following the flock

---- ignore_attribute_reason ----
no honey

---- ignore_failed ----
jumped too high

---- later_ignore_failed ----
broke neck


failures:
    failed
    ignore_attribute
    ignore_attribute_reason
    ignore_failed
    later_ignore_failed

test result: FAILED. 4 passed; 5 failed; 0 ignored; 2 filtered out; finished in [..]s


"#]],
    );
}

#[test]
fn lots_of_flags() {
    check(
        &["--ignored", "--skip", "g", "--test", "o"],
        101,
        str![[r#"

running 1 test
test custom_error ... FAILED

failures:

---- custom_error ----
I failed


failures:
    custom_error

test result: FAILED. 0 passed; 1 failed; 0 ignored; 10 filtered out; finished in [..]s


"#]],
        str![[r#"

running 1 test
...

failures:

---- custom_error ----
I failed


failures:
    custom_error

test result: FAILED. 0 passed; 1 failed; 0 ignored; 10 filtered out; finished in [..]s


"#]],
    );
}

#[test]
fn terse_output() {
    check(
        &["--quiet"],
        101,
        str![[r#"

running 11 tests
.FFiiiiii..
failures:

---- custom_error ----
I failed

---- failed ----
was not a good boy


failures:
    custom_error
    failed

test result: FAILED. 3 passed; 2 failed; 6 ignored; 0 filtered out; finished in [..]s


"#]],
        str![[r#"

running 11 tests
...
failures:

---- custom_error ----
I failed

---- failed ----
was not a good boy


failures:
    custom_error
    failed

test result: FAILED. 3 passed; 2 failed; 6 ignored; 0 filtered out; finished in [..]s


"#]],
    );
}

#[test]
fn json_list() {
    check(
        &["-Zunstable-options", "--format=json", "--list", "a"],
        0,
        str![[r#"
[
  {
    "elapsed_s": "[..]",
    "event": "discover_start"
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "cfg_in",
    "selected": false,
    "source": {
      "end_col": 10,
      "end_line": 61,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 61
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "custom_error",
    "selected": false,
    "source": {
      "end_col": 16,
      "end_line": 15,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 15
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "failed",
    "source": {
      "end_col": 10,
      "end_line": 10,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 10
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "ignore_attribute",
    "source": {
      "end_col": 20,
      "end_line": 49,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 49
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "ignore_attribute_reason",
    "source": {
      "end_col": 27,
      "end_line": 55,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 55
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "ignore_failed",
    "source": {
      "end_col": 17,
      "end_line": 24,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 24
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "ignore_passed",
    "source": {
      "end_col": 17,
      "end_line": 30,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 30
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "later_ignore_failed",
    "source": {
      "end_col": 23,
      "end_line": 36,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 36
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "later_ignore_passed",
    "source": {
      "end_col": 23,
      "end_line": 42,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 42
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "later_passed",
    "source": {
      "end_col": 16,
      "end_line": 20,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 20
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "passed",
    "source": {
      "end_col": 10,
      "end_line": 6,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 6
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_complete"
  }
]
"#]]
        .is_json()
        .against_jsonlines(),
        str![[r#"
[
  {
    "elapsed_s": "[..]",
    "event": "discover_start"
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "cfg_in",
    "selected": false,
    "source": {
      "end_col": 10,
      "end_line": 61,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 61
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "custom_error",
    "selected": false,
    "source": {
      "end_col": 16,
      "end_line": 15,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 15
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "failed",
    "source": {
      "end_col": 10,
      "end_line": 10,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 10
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "ignore_attribute",
    "source": {
      "end_col": 20,
      "end_line": 49,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 49
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "ignore_attribute_reason",
    "source": {
      "end_col": 27,
      "end_line": 55,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 55
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "ignore_failed",
    "source": {
      "end_col": 17,
      "end_line": 24,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 24
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "ignore_passed",
    "source": {
      "end_col": 17,
      "end_line": 30,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 30
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "later_ignore_failed",
    "source": {
      "end_col": 23,
      "end_line": 36,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 36
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "later_ignore_passed",
    "source": {
      "end_col": 23,
      "end_line": 42,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 42
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "later_passed",
    "source": {
      "end_col": 16,
      "end_line": 20,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 20
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "passed",
    "source": {
      "end_col": 10,
      "end_line": 6,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 6
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_complete"
  }
]
"#]]
        .is_json()
        .against_jsonlines(),
    );
}

#[test]
fn json_filter() {
    check(
        &["-Zunstable-options", "--format=json", "a"],
        101,
        str![[r#"
[
  {
    "elapsed_s": "[..]",
    "event": "discover_start"
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "cfg_in",
    "selected": false,
    "source": {
      "end_col": 10,
      "end_line": 61,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 61
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "custom_error",
    "selected": false,
    "source": {
      "end_col": 16,
      "end_line": 15,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 15
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "failed",
    "source": {
      "end_col": 10,
      "end_line": 10,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 10
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "ignore_attribute",
    "source": {
      "end_col": 20,
      "end_line": 49,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 49
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "ignore_attribute_reason",
    "source": {
      "end_col": 27,
      "end_line": 55,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 55
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "ignore_failed",
    "source": {
      "end_col": 17,
      "end_line": 24,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 24
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "ignore_passed",
    "source": {
      "end_col": 17,
      "end_line": 30,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 30
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "later_ignore_failed",
    "source": {
      "end_col": 23,
      "end_line": 36,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 36
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "later_ignore_passed",
    "source": {
      "end_col": 23,
      "end_line": 42,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 42
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "later_passed",
    "source": {
      "end_col": 16,
      "end_line": 20,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 20
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "passed",
    "source": {
      "end_col": 10,
      "end_line": 6,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 6
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_complete"
  },
  {
    "elapsed_s": "[..]",
    "event": "run_start"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_start",
    "name": "failed"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_message",
    "kind": "error",
    "message": "was not a good boy",
    "name": "failed"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_complete",
    "name": "failed"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_start",
    "name": "ignore_attribute"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_message",
    "kind": "ignored",
    "name": "ignore_attribute"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_complete",
    "name": "ignore_attribute"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_start",
    "name": "ignore_attribute_reason"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_message",
    "kind": "ignored",
    "message": "fails",
    "name": "ignore_attribute_reason"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_complete",
    "name": "ignore_attribute_reason"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_start",
    "name": "ignore_failed"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_message",
    "kind": "ignored",
    "message": "fails",
    "name": "ignore_failed"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_complete",
    "name": "ignore_failed"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_start",
    "name": "ignore_passed"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_message",
    "kind": "ignored",
    "message": "slow",
    "name": "ignore_passed"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_complete",
    "name": "ignore_passed"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_start",
    "name": "later_ignore_failed"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_message",
    "kind": "ignored",
    "message": "fails",
    "name": "later_ignore_failed"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_complete",
    "name": "later_ignore_failed"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_start",
    "name": "later_ignore_passed"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_message",
    "kind": "ignored",
    "message": "fails",
    "name": "later_ignore_passed"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_complete",
    "name": "later_ignore_passed"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_start",
    "name": "later_passed"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_complete",
    "name": "later_passed"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_start",
    "name": "passed"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_complete",
    "name": "passed"
  },
  {
    "elapsed_s": "[..]",
    "event": "run_complete"
  }
]
"#]]
        .is_json()
        .against_jsonlines(),
        str![[r#"
[
  {
    "elapsed_s": "[..]",
    "event": "discover_complete"
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_start"
  },
  {
    "elapsed_s": "[..]",
    "event": "run_start"
  },
  {
    "elapsed_s": "[..]",
    "event": "run_complete"
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "custom_error",
    "selected": false,
    "source": {
      "end_col": 16,
      "end_line": 15,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 15
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "cfg_in",
    "selected": false,
    "source": {
      "end_col": 10,
      "end_line": 61,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 61
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "failed",
    "source": {
      "end_col": 10,
      "end_line": 10,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 10
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "ignore_attribute",
    "source": {
      "end_col": 20,
      "end_line": 49,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 49
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "ignore_attribute_reason",
    "source": {
      "end_col": 27,
      "end_line": 55,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 55
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "ignore_failed",
    "source": {
      "end_col": 17,
      "end_line": 24,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 24
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "ignore_passed",
    "source": {
      "end_col": 17,
      "end_line": 30,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 30
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "later_ignore_failed",
    "source": {
      "end_col": 23,
      "end_line": 36,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 36
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "later_ignore_passed",
    "source": {
      "end_col": 23,
      "end_line": 42,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 42
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "later_passed",
    "source": {
      "end_col": 16,
      "end_line": 20,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 20
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "passed",
    "source": {
      "end_col": 10,
      "end_line": 6,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 6
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "case_start",
    "name": "failed"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_start",
    "name": "ignore_attribute"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_message",
    "kind": "ignored",
    "name": "ignore_attribute"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_complete",
    "name": "ignore_attribute"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_message",
    "kind": "error",
    "message": "was not a good boy",
    "name": "failed"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_complete",
    "name": "failed"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_start",
    "name": "ignore_failed"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_start",
    "name": "ignore_passed"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_message",
    "kind": "ignored",
    "message": "slow",
    "name": "ignore_passed"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_complete",
    "name": "ignore_passed"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_start",
    "name": "ignore_attribute_reason"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_message",
    "kind": "ignored",
    "message": "fails",
    "name": "ignore_attribute_reason"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_complete",
    "name": "ignore_attribute_reason"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_start",
    "name": "passed"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_complete",
    "name": "passed"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_start",
    "name": "later_passed"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_complete",
    "name": "later_passed"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_message",
    "kind": "ignored",
    "message": "fails",
    "name": "ignore_failed"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_complete",
    "name": "ignore_failed"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_start",
    "name": "later_ignore_passed"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_start",
    "name": "later_ignore_failed"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_message",
    "kind": "ignored",
    "message": "fails",
    "name": "later_ignore_passed"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_complete",
    "name": "later_ignore_passed"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_message",
    "kind": "ignored",
    "message": "fails",
    "name": "later_ignore_failed"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_complete",
    "name": "later_ignore_failed"
  }
]
"#]]
        .unordered()
        .is_json()
        .against_jsonlines(),
    );
}

#[test]
fn json_fail_fast() {
    check(
        &["-Zunstable-options", "--format=json", "--fail-fast"],
        101,
        str![[r#"
[
  {
    "elapsed_s": "[..]",
    "event": "discover_start"
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "cfg_in",
    "source": {
      "end_col": 10,
      "end_line": 61,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 61
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "custom_error",
    "source": {
      "end_col": 16,
      "end_line": 15,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 15
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "failed",
    "source": {
      "end_col": 10,
      "end_line": 10,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 10
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "ignore_attribute",
    "source": {
      "end_col": 20,
      "end_line": 49,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 49
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "ignore_attribute_reason",
    "source": {
      "end_col": 27,
      "end_line": 55,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 55
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "ignore_failed",
    "source": {
      "end_col": 17,
      "end_line": 24,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 24
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "ignore_passed",
    "source": {
      "end_col": 17,
      "end_line": 30,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 30
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "later_ignore_failed",
    "source": {
      "end_col": 23,
      "end_line": 36,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 36
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "later_ignore_passed",
    "source": {
      "end_col": 23,
      "end_line": 42,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 42
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "later_passed",
    "source": {
      "end_col": 16,
      "end_line": 20,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 20
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_case",
    "name": "passed",
    "source": {
      "end_col": 10,
      "end_line": 6,
      "path": "test.rs",
      "start_col": 4,
      "start_line": 6
    }
  },
  {
    "elapsed_s": "[..]",
    "event": "discover_complete"
  },
  {
    "elapsed_s": "[..]",
    "event": "run_start"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_start",
    "name": "cfg_in"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_complete",
    "name": "cfg_in"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_start",
    "name": "custom_error"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_message",
    "kind": "error",
    "message": "I failed",
    "name": "custom_error"
  },
  {
    "elapsed_s": "[..]",
    "event": "case_complete",
    "name": "custom_error"
  },
  {
    "elapsed_s": "[..]",
    "event": "run_complete"
  }
]
"#]]
        .is_json()
        .against_jsonlines(),
        str![[r#"
...

"#]],
    );
}
