// Adapted from https://github.com/nvzqz/divan/blob/a773ab9974be97fd0d7d33d7e9fb2d4413dbc797/src/entry/list.rs

use std::{
    ptr,
    sync::atomic::{AtomicPtr, Ordering as AtomicOrdering},
};

/// Linked list of entries.
pub struct DistributedList<T: 'static> {
    entry: Option<&'static T>,
    // This is implemented in a thread-safe way despite the fact that constructors
    // are run single-threaded.
    next: AtomicPtr<Self>,
}

impl<T> DistributedList<T> {
    /// Dereferences the `next` pointer.
    #[inline]
    fn next(&self) -> Option<&Self> {
        // SAFETY: `next` is only assigned by `push`, which always receives a
        // 'static lifetime.
        unsafe { self.next.load(AtomicOrdering::Relaxed).as_ref() }
    }
}

// Externally used by macros or tests.
#[allow(missing_docs)]
impl<T> DistributedList<T> {
    /// Create an empty list
    #[inline]
    pub const fn root() -> Self {
        Self {
            entry: None,
            next: AtomicPtr::new(ptr::null_mut()),
        }
    }

    /// Create a new node
    ///
    /// See [`DistributedList::push`]
    #[inline]
    pub const fn new(entry: &'static T) -> Self {
        Self {
            entry: Some(entry),
            next: AtomicPtr::new(ptr::null_mut()),
        }
    }

    /// Iterate over entries starting at `self`
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        let mut list = Some(self);
        std::iter::from_fn(move || -> Option<Option<&T>> {
            let current = list?;
            list = current.next();
            Some(current.entry.as_ref().copied())
        })
        .flatten()
    }

    /// Inserts `other` to the front of the list.
    ///
    /// This function must be safe to call before `main`.
    #[inline]
    pub fn push(&'static self, other: &'static Self) {
        let mut old_next = self.next.load(AtomicOrdering::Relaxed);
        loop {
            // Each publicly-created instance has `list.next` be null, so we can
            // simply store `self.next` there.
            other.next.store(old_next, AtomicOrdering::Release);

            // NOTE: The content of `other` can already be seen, so we don't
            // need to strongly order reads into it.
            let other = other as *const Self as *mut Self;
            match self.next.compare_exchange_weak(
                old_next,
                other,
                AtomicOrdering::AcqRel,
                AtomicOrdering::Acquire,
            ) {
                // Successfully wrote our thread's value to the list.
                Ok(_) => return,

                // Lost the race, store winner's value in `other.next`.
                Err(new) => old_next = new,
            }
        }
    }
}
