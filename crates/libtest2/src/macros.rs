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
        const _: () = {
            $crate::_private::push!(
                crate::TESTS,
                _: $crate::_private::DynCase = $crate::_private::DynCase(&$crate::FnCase::test(
                    ::std::borrow::Cow::Borrowed(stringify!($name)),
                    |context| {
                        $(
                            match $crate::_private::parse_ignore!($($attr)*) {
                                ::std::option::Option::None => context.ignore()?,
                                ::std::option::Option::Some(reason) => context.ignore_for(reason)?,
                            }
                        )*

                        use $crate::IntoRunResult;
                        let result = $name(context);
                        IntoRunResult::into_run_result(result)
                    }
                ))
            );
        };

        fn $name $($item)*
    };
}
