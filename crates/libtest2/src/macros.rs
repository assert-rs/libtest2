#[macro_export]
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
macro_rules! _parse_ignore {
    (ignore) => {
        ::std::option::Option::<&'static str>::None
    };
    (ignore = $reason:expr) => {
        ::std::option::Option::<&'static str>::Some($reason)
    };
    ($($attr:tt)*) => {
        compile_error!(concat!("unknown attribute '", stringify!($($attr)*), "'"));
    };
}

#[macro_export]
#[allow(clippy::crate_in_macro_def)] // accessing item defined by `_main_parse`
macro_rules! _test_parse {
    (#[test] $(#[$($attr:tt)*])* fn $name:ident $($item:tt)*) => {
        #[allow(non_camel_case_types)]
        struct $name;

        impl $crate::Case for $name {
            fn name(&self) -> &str {
                $crate::_private::push!(crate::TESTS, _: $crate::_private::DynCase = $crate::_private::DynCase(&$name));

                stringify!($name)
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
                fn run $($item)*

                $(
                    match $crate::_private::parse_ignore!($($attr)*) {
                        ::std::option::Option::None => context.ignore()?,
                        ::std::option::Option::Some(reason) => context.ignore_for(reason)?,
                    }
                )*

                use $crate::IntoRunResult;
                let result = run(context);
                IntoRunResult::into_run_result(result)
            }
        }
    };
}
