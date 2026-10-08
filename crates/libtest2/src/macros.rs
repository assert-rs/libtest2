#[macro_export]
#[doc(hidden)]
macro_rules! _main_parse {
    (#[main] $(#[$meta:meta])* fn main $($item:tt)*) => {
        static TESTS: $crate::_private::DistributedList<$crate::_private::DynCase> = $crate::_private::DistributedList::root();

        $(#[$meta])*
        fn main() {
            fn inner $($item)*

            inner();
            $crate::main(TESTS.iter().copied());
        }
    };
}

#[macro_export]
#[doc(hidden)]
#[allow(clippy::crate_in_macro_def)] // accessing item defined by `_main_parse`/`_parse_ignore`/`_run_test`, and recursively calling the macro itself
macro_rules! _test_parse {
    // Entry point
    (#[test] $(#[$($attr:tt)+])* fn $name:ident $($item:tt)*) => {
        $crate::_private::test_parse!(continue:
            name=$name
            body=[$($item)*]
            attrs=[$(#[$($attr)+])*]
        );
    };

    // Recursively handle attributes:

    // Edge condition (no more attributes to parse)
    (continue: name=$name:ident body=[$($item:tt)*] attrs=[] $(ignore=$ignore:tt)? $(should_panic=$should_panic:tt)?) => {
        $crate::_private::test_parse!(break:
            name=$name
            body=[$($item)*]
            $(ignore=$ignore)?
            $(should_panic=$should_panic)?
        );
    };
    // Process `#[ignore]`/`#[ignore = ".."]` (NOTE: This will only match if an ignore macro has not already been parsed)
    (continue: name=$name:ident body=[$($item:tt)*] attrs=[#[ignore $(= $reason:literal)?] $(#[$($attr:tt)+])*] $(should_panic=$should_panic:tt)?) => {
        $crate::_private::test_parse!(continue:
            name=$name
            body=[$($item)*]
            attrs=[$(#[$($attr)*])*]
            ignore=[$($reason)?]
            $(should_panic=$should_panic)?
        );
    };
    // Ignore subsequent calls to `#[ignore]`/`#[ignore = ".."]`
    (continue: name=$name:ident body=[$($item:tt)*] attrs=[#[ignore $(= $reason:literal)?] $(#[$($attr:tt)+])*] ignore=$ignore:tt $(should_panic=$should_panic:tt)?) => {
        $crate::_private::test_parse!(continue:
            name=$name
            body=[$($item)*]
            attrs=[$(#[$($attr)*])*]
            ignore=$ignore
            $(should_panic=$should_panic)?
        );
    };
    // Process `#[should_panic]`/`#[should_panic = ".."]` (NOTE: This will only match if a should_panic macro has not already been parsed)
    (continue: name=$name:ident body=[$($item:tt)*] attrs=[#[should_panic $(= $expected:literal)?] $(#[$($attr:tt)+])*] $(ignore=$ignore:tt)?) => {
        $crate::_private::test_parse!(continue:
            name=$name
            body=[$($item)*]
            attrs=[$(#[$($attr)*])*]
            $(ignore=$ignore)?
            should_panic=[$($expected)?]
        );
    };
    // Process `#[should_panic(expected = "..")]` (NOTE: Same as branch above)
    (continue: name=$name:ident body=[$($item:tt)*] attrs=[#[should_panic(expected = $expected:literal)] $(#[$($attr:tt)+])*] $(ignore=$ignore:tt)?) => {
        $crate::_private::test_parse!(continue:
            name=$name
            body=[$($item)*]
            attrs=[$(#[$($attr)*])*]
            $(ignore=$ignore)?
            should_panic=[$expected]
        );
    };
    // Emit an error for subsequent calls to `#[should_panic]`/`#[should_panic = ".."]`/`#[should_panic(expected = "..")]` (but continue parsing)
    (continue: name=$name:ident body=[$($item:tt)*] attrs=[#[should_panic $($unused:tt)*] $(#[$($attr:tt)+])*] $(ignore=$ignore:tt)? should_panic=$should_panic:tt) => {
        compile_error!("annotating a test with multiple 'should_panic' attributes is not allowed");
        $crate::_private::test_parse!(continue:
            name=$name
            body=[$($item)*]
            attrs=[$(#[$($attr)*])*]
            $(ignore=$ignore)?
            should_panic=$should_panic
        );
    };
    // Emit error on unknown attributes (but continue parsing)
    (continue: name=$name:ident body=[$($item:tt)*] attrs=[#[$($unknown_attr:tt)+] $(#[$($attr:tt)+])*] $(ignore=$ignore:tt)? $(should_panic=$should_panic:tt)?) => {
        compile_error!(concat!("unknown attribute '", stringify!($($unknown_attr)+), "'"));
        $crate::_private::test_parse!(continue:
            name=$name
            body=[$($item)*]
            attrs=[$(#[$($attr)*])*]
            $(ignore=$ignore)?
            $(should_panic=$should_panic)?
        );
    };

    // End result
    (break: name=$name:ident body=[($($params:tt)*) $($item:tt)*] $(ignore=$ignore:tt)? $(should_panic=$should_panic:tt)?) => {
        fn $name($($params)*) $($item)*

        const _: () = {
            struct Test;

            impl $crate::Case for Test {
                fn name(&self) -> &str {
                    $crate::_private::push!(crate::TESTS, _: $crate::_private::DynCase = $crate::_private::DynCase(&Test));

                    const FULL_PATH: &str = concat!(std::module_path!(), "::", stringify!($name));
                    let i = FULL_PATH.find("::").expect("we have inserted this in the line above so it must be there");
                    &FULL_PATH[(i+2)..]
                }
                fn kind(&self) -> $crate::_private::TestKind {
                    Default::default()
                }
                fn source(&self) -> Option<&$crate::_private::Source> {
                    None
                }
                fn exclusive(&self, _: &$crate::TestContext) -> bool {
                    false
                }

                fn run(&self, context: &$crate::TestContext) -> $crate::RunResult {
                    $crate::_private::parse_ignore!(context, $($ignore)?);

                    use $crate::IntoRunResult;
                    let result = $crate::_private::run_test!($crate::_private::test_expr!($name, context, [$($params)*]), $($should_panic)?);
                    IntoRunResult::into_run_result(result)
                }
            }
        };
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! _parse_ignore {
    ($context:expr, [$reason:literal] $(,)?) => {
        $context.ignore_for($reason)?
    };
    ($context:expr, [] $(,)?) => {
        $context.ignore()?
    };
    ($context:expr $(,)?) => {};
}

#[macro_export]
#[doc(hidden)]
macro_rules! _test_expr {
    ($name:ident, $context:expr, []) => {
        $name()
    };
    ($name:ident, $context:expr, [$($params:tt)+]) => {
        $name($context)
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! _run_test {
    ($test:expr, [$expected:literal]) => {
        $crate::panic::assert_panic_contains(|| $test, $expected)
    };
    ($test:expr, []) => {
        $crate::panic::assert_panic(|| $test)
    };
    ($test:expr $(,)?) => {{
        $test
    }};
}
