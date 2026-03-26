# Design decisions

Overall goals:
1. Very fast build times (our competitor, libtest, has 0s build times)
2. Flexible enough that we can have just one test harness to cover nearly everyone's needs
  - `trybuild`, `trycmd`, `tryfn`, `toml-test-harness`, etc can all be extensions
3. Minimal breaking changes after 1.0
4. Provide building blocks for other other custom test harnesses

## json format

Goals:
- Allow a runner, like cargo-test, to take over UX concerns, making the experience richer and removing burdens from custom test harness writers
- Allow a runner, like cargo-test, to run binaries in parallel
- Evolve with users to handle their custom test harnesses

Care abouts
- Minimize the burden on custom test harness authors
- Recognize we can't see the future and allow adaptation

See also [eRFC 3558](https://github.com/rust-lang/rfcs/pull/3558)

Current proposal:
- [serde definition](https://github.com/assert-rs/libtest2/blob/main/crates/libtest-json/src/event.rs)
- [JSONSchema](https://github.com/assert-rs/libtest2/blob/main/crates/libtest-json/event.schema.json)
- [example output](https://github.com/assert-rs/libtest2/blob/main/crates/libtest2-mimic/tests/testsuite/mixed_bag.rs)

Decisions
- Always report discovery
  - Allows callers to provide a progress indicator
  - Replaces the need for harnesses to provide statistics
- Each event carries an `elapsed_s` as an offset from process start
  - Makes units explicit
  - Can track duration of any operation
- Defer to cargo to provide test-binary differentiating information, like it does for rustc
- No equivalent of rustc including a rendered diagnostic
  - Terse and pretty progress indicators are too nebulous to render (see their notifiers)
  - There is likely not enough value add in the failure message
  - This puts more of a burden on custom test harnesses for their implementation than is strictly needed
- Report failures separate from test-complete so we can have multiple
- `DiscoverCase` order is unspecified so we can report them as found rather than waiting for a sort phase so users can identify slow discovery

### Prior Art

#### libtest's existing format

[libtest's existing format](https://github.com/rust-lang/rust/blob/master/library/test/src/formatters/json.rs) (as ndjson):
```json
[
    {
        "type": "suite",
        "event": "discovery"
    },
    {
        "type": "<test|bench>",
        "event": "discovered",
        "name": "",
        "ignore": false,
        "ignore_message": "",
        "source_path": "",
        "start_line": 0,
        "start_col": 0,
        "end_line": 0,
        "end_col": 0
    },
    {
        "type": "suite",
        "event": "completed",
        "tests": 0,
        "benches": 0,
        "total": 0,
        "ignored": 0
    },
    {
        "type": "suite",
        "event": "started",
        "test_count": 0,
        "shuffle_seed": 0  # or not-present (unstable)
    },
    {
        "type": "test",
        "event": "started",
        "name": "",
    },
    {
        "type": "test",
        "event": "<ok|failed|ignored>",
        "name": "",
        "exec_time": 0.0,  # or not-present (unstable)
        "stdout": "",  # or not-present
        "message": "", # present only for `failed`, `ignored`
        "reason": "time limit exceeded",  # present only for `failed`
    },
    {
        "type": "bench",  # (unstable)
        "name": "",
        "median": 0,
        "deviation": 0,
        "mib_per_second": 0  # or not-present
    },
    {
        "type": "test",
        "event": "timeout",
        "name": ""
    },
    {
        "type": "suite",
        "event": "<ok|failed>",
        "passed": 0,
        "failed": 0,
        "ignored": 0,
        "measured": 0,
        "filtered_out": 0,
        "exec_time": 0  # (unstable)
    }
]
```
- The event type is split between `event` and `type`
  - This becomes even more complicated when `event` is also used to convey "status"
- Ambiguous when multiple streams of these get merged, like if we had `cargo test --message-format=json` support for this
- Carries presentation-layer concerns like count
- Line/column is a presentation-layer way of tracking location within a file (vs byte offsets)
- Does not support runtime ignoring (can't report a test is ignored outside of discovery)
- Does not directly support runtime test case (`name` is assumed to be same between discovery and running)
- bench (unstable)
  - Doesn't even have `event`
  - No "started" event
  - `mib_per_second` is too application-specific
  - Does not convey units
  - No extension point for special reporters

#### TAP

[TAP](https://testanything.org/tap-version-14-specification.html)

TAP uses a custom syntax.
Within the Rust toolchain, json output is commonly used and the output from a test harness may be mixed with the output from rustc.
While Cargo could translate TAP to json messages for these cases, we then duplicate effort.

TAP uses indices for tests.
**TODO** we could report indices during discovery and then use those from then on for a lighter weight way of tracking cases.

#### pytest-json-report

[pytest-json-report](https://github.com/numirias/pytest-json-report)
- Tracks the test environment

[example](https://github.com/numirias/pytest-json-report/blob/master/sample_report.json)

#### pytest-reportlog

[pytest-reportlog](https://github.com/pytest-dev/pytest-reportlog)
- Uses `$report_type` for each jsonline message
- Full output is delimited by "session start" and "session end"

Endorsed in [pytest's docs](https://docs.pytest.org/en/latest/deprecations.html#result-log-result-log)

#### subunit

[subunit](https://github.com/testing-cabal/subunit) ([rust impl](https://github.com/mtreinish/subunit-rust))

#### JUnit XML

[testmoapp/junitxml](https://github.com/testmoapp/junitxml)
- Non-streaming format
- More work to generate properly, possibly impacting compile times of custom test harnesses
- No specified format; requires experimenting with supported consumers
- Lacks per-case timestamps

Example:
```xml
<?xml version="1.0" encoding="UTF-8"?>
<testsuites time="15.682687">
    <testsuite name="Tests.Registration" time="6.605871">
        <testcase name="testCase1" classname="Tests.Registration" time="2.113871" />
        <testcase name="testCase2" classname="Tests.Registration" time="1.051" />
        <testcase name="testCase3" classname="Tests.Registration" time="3.441" />
    </testsuite>
    <testsuite name="Tests.Authentication" time="9.076816">
        <testsuite name="Tests.Authentication.Login" time="4.356">
            <testcase name="testCase4" classname="Tests.Authentication.Login" time="2.244" />
            <testcase name="testCase5" classname="Tests.Authentication.Login" time="0.781" />
            <testcase name="testCase6" classname="Tests.Authentication.Login" time="1.331" />
        </testsuite>
        <testcase name="testCase7" classname="Tests.Authentication" time="2.508" />
        <testcase name="testCase8" classname="Tests.Authentication" time="1.230816" />
        <testcase name="testCase9" classname="Tests.Authentication" time="0.982">
            <failure message="Assertion error message" type="AssertionError">
                <!-- Call stack printed here -->
            </failure>            
        </testcase>
    </testsuite>
</testsuites>
```

## lexarg

Goal: provide an API-stable CLI parser for inclusion in APIs for plugin-specific CLI args

### Decision: level of abstraction

Potential design directions
- High-level argument definitions that get aggregated
  - e.g. like https://crates.io/crates/gflags
- Low-level, cooperative parsing
  - e.g. like https://crates.io/crates/lexopt

`lexopt`-like API was selected as it was assumed to have the most potential for
meeting future needs because parsing control is handed to the plugin.

This comes at the cost of:
- Requires every plugin to cooperate
- More manual help construction

### Decision: iteration model

Potential design directions
- `lexopt` exposes a single iterator type that walks over both longs and shorts.
- `clap_lex` exposes an iterator type that walks over each argument with an inner iterator when walking over short flags

`lexopt`-like API was selected.  While `clap_lex` is the more powerful API,
this makes delegating to plugins in a cooperative way more challenging.

### Decision: reuse lexopt vs build something new

In reviewing lexopt's API:
- Error handling is included in the API in a way that might make evolution difficult
- Escapes aren't explicitly communicated which makes communal parsing more difficult
- lexopt builds in specific option-value semantics

And in general we will be putting the parser in the libtest-next's API and it will be a fundamental point of extension.
Having complete control helps ensure the full experience is cohesive.

### Decision: `Short(&str)`

`lexopt` and `clap` / `clap_lex` treat shorts as a `char` which gives a level of type safety to parsing.
However, with a minimal API, providing `&str` provides span information "for free".

If someone were to make an API for pluggable lexers,
support for multi-character shorts is something people may want to opt-in to (it has been requested of clap).

Performance isn't the top priority, so remoing `&str` -> `char` conversions isn't necessarily viewed as a benefit.
This also makes `match` need to work off of `&str` instead of `char`.
Unsure which of those would be slower and how the different characteristics match up.

## Harness

### Decision: report and run tests in filter order

Rather than build into every harness shuffle, sharding, and any other specific logic like that,
we can instead give the user direct control over the test order by the order they are specified on the command line.

### Decision: argfile support

Similar to filters changing the order of tests,
argfile support allows for passing a large list of arguments to a test binary.

The syntax and semantics match rustc:
- Expanded before parsing, independent of any other syntax
- Arguments are delimited by newlines; no shell escaping
  - rustc has unstable support for `@shell:<path>`
- Lines are read literal, empty lines are empty arguments and no comments
- Non-recursive

## `json-write`

### Decision: custom json writer

The goal is to minimize build times.  Switching from `serde_json` dropped out build times by an order of magnitude.

Other libraries exist in this space but generally take on too much, e.g.
- https://crates.io/crates/write-json: json-safe API
- https://crates.io/crates/json-writer: also supports a more json-safe API
- https://crates.io/crates/escape8259: only strings, also parses
