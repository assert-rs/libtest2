#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(clippy::print_stderr)]
#![warn(clippy::print_stdout)]

mod list;

pub use list::DistributedList;

#[doc(hidden)]
pub mod _private {
    pub use ctor::declarative::ctor;
}

/// Push `$value` into `$root`
///
/// # Example
///
/// ```rust
/// # use distributed_list::DistributedList;
/// # use distributed_list::push;
///
/// static REGISTRY: DistributedList<usize> = DistributedList::root();
///
/// push!(REGISTRY, ONE: usize = 1);
/// push!(REGISTRY, TWO: usize = 2);
/// push!(REGISTRY, THREE: usize = 3);
///
/// for entry in REGISTRY.iter() {
///     println!("{entry}");
/// }
/// ```
///
/// ```rust
/// # use distributed_list::DistributedList;
/// # use distributed_list::push;
///
/// static REGISTRY: DistributedList<usize> = DistributedList::root();
///
/// fn foo() {
///     push!(REGISTRY, _: usize = 1);
/// }
/// fn bar() {
///     push!(REGISTRY, _: usize = 2);
/// }
/// fn baz() {
///     push!(REGISTRY, _: usize = 3);
/// }
///
/// for entry in REGISTRY.iter() {
///     println!("{entry}");
/// }
/// ```
#[macro_export]
macro_rules! push {
    ($root:path, _ : $ty:ty = $value:expr) => {
        $crate::_private::ctor! {
            #[ctor]
            unsafe fn anonymous_pushes_require_unique_scope() {
                static ITEM: $ty = $value;

                // Report type errors
                let _: &$crate::DistributedList<$ty> = &$root;

                static ENTRY: $crate::DistributedList<$ty> = $crate::DistributedList::new(&ITEM);

                $root.push(&ENTRY);
            }
        }
    };
    ($root:path, $name:ident : $ty:ty = $value:expr) => {
        static $name: $ty = {
            {
                $crate::_private::ctor! {
                    #[ctor]
                    unsafe fn push() {
                        // Report type errors
                        let _: &$crate::DistributedList<$ty> = &$root;

                        static ENTRY: $crate::DistributedList<$ty> = $crate::DistributedList::new(&$name);

                        $root.push(&ENTRY);
                    }
                }
            }

            $value
        };
    };
}

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;
