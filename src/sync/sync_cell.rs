use std::cell::UnsafeCell;
use std::error::Error;
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicBool, Ordering};

pub struct SyncCell<T> {
    borrowed: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for SyncCell<T> {}
unsafe impl<T: Send> Sync for SyncCell<T> {}

impl<T> SyncCell<T> {
    pub fn new(data: T) -> SyncCell<T> {
        SyncCell {
            borrowed: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    pub fn try_borrow_mut(&self) -> Result<Guard<'_, T>, BorrowMutError> {
        match self.borrowed.swap(true, Ordering::Acquire) {
            false => Ok(Guard { cell: self }),
            true => Err(BorrowMutError {}),
        }
    }

    pub fn borrow_mut(&self) -> Guard<'_, T> {
        match self.try_borrow_mut() {
            Ok(b) => b,
            Err(_) => panic!("SyncCell already borrowed"),
        }
    }
}

pub struct Guard<'a, T> {
    cell: &'a SyncCell<T>,
}

impl<'a, T> Deref for Guard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.cell.data.get() }
    }
}

impl<'a, T> DerefMut for Guard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.cell.data.get() }
    }
}

impl<'a, T> Drop for Guard<'a, T> {
    fn drop(&mut self) {
        self.cell.borrowed.store(false, Ordering::Release);
    }
}

#[derive(Debug)]
pub struct BorrowMutError;

impl fmt::Display for BorrowMutError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        "SyncCell already borrowed".fmt(fmt)
    }
}

impl Error for BorrowMutError {}
