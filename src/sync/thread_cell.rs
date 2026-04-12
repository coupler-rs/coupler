use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};
use std::thread::{self, ThreadId};

pub struct ThreadCell<T> {
    pub thread: ThreadId,
    data: ManuallyDrop<T>,
}

impl<T> ThreadCell<T> {
    pub fn new(data: T) -> ThreadCell<T> {
        ThreadCell {
            thread: thread::current().id(),
            data: ManuallyDrop::new(data),
        }
    }
}

unsafe impl<T> Send for ThreadCell<T> {}
unsafe impl<T> Sync for ThreadCell<T> {}

impl<T> Deref for ThreadCell<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        assert!(thread::current().id() == self.thread);

        &*self.data
    }
}

impl<T> DerefMut for ThreadCell<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        assert!(thread::current().id() == self.thread);

        &mut *self.data
    }
}

impl<T> Drop for ThreadCell<T> {
    fn drop(&mut self) {
        assert!(thread::current().id() == self.thread);

        unsafe { ManuallyDrop::drop(&mut self.data) };
    }
}

// #[derive(Debug)]
// pub struct ThreadError;

// impl fmt::Display for ThreadError {
//     fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
//         "ThreadCell was created on a different thread".fmt(fmt)
//     }
// }

// impl Error for ThreadError {}
