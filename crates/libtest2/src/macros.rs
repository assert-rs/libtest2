#[macro_export]
macro_rules! _main_parse {
    (#[main] fn main $($item:tt)*) => {
        fn main() {
            fn inner $($item)*

            inner();
            $crate::main($crate::get_tests());
        }
    };
}

#[macro_export]
#[allow(clippy::crate_in_macro_def)] // accessing item defined by `_main_parse`
macro_rules! _test_parse {
    (#[test] fn $name:ident $($item:tt)*) => {
        #[allow(non_camel_case_types)]
        struct $name;

        impl $crate::_private::Case for $name {
            fn name(&self) -> &str {
                $crate::_private::push!($crate::_private::TESTS, _: $crate::_private::DynCase = $crate::_private::DynCase(&$name));

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

                run(context)
            }
        }
    };
}
