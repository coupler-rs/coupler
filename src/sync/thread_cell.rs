use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};
use std::thread::{self, ThreadId};

/// A wrapper that makes it possible to share `!Send` values between threads.
///
/// `ThreadCell` implements `Send` and `Sync` for all `T` while ensuring that its contents can only
/// be accessed from the original thread on which it was created. Creating a `ThreadCell` on one
/// thread and then dereferencing or dropping it on another thread will result in a panic.
pub struct ThreadCell<T> {
    thread: ThreadId,
    data: ManuallyDrop<T>,
}

unsafe impl<T> Send for ThreadCell<T> {}
unsafe impl<T> Sync for ThreadCell<T> {}

impl<T> ThreadCell<T> {
    pub fn new(data: T) -> ThreadCell<T> {
        ThreadCell {
            thread: thread::current().id(),
            data: ManuallyDrop::new(data),
        }
    }

    fn assert_thread(&self) {
        if thread::current().id() != self.thread {
            panic!("ThreadCell was created on a different thread");
        }
    }
}

impl<T> Deref for ThreadCell<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.assert_thread();

        &*self.data
    }
}

impl<T> DerefMut for ThreadCell<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.assert_thread();

        &mut *self.data
    }
}

impl<T> Drop for ThreadCell<T> {
    fn drop(&mut self) {
        self.assert_thread();

        unsafe { ManuallyDrop::drop(&mut self.data) };
    }
}
