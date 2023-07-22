use std::array;
use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

const UNREAD_MASK: u8 = 0b100;
const INDEX_MASK: u8 = 0b011;

#[derive(Copy, Clone)]
struct State {
    unread: bool,
    index: usize,
}

impl State {
    fn to_bits(self) -> u8 {
        let index = self.index as u8 & INDEX_MASK;
        if self.unread {
            UNREAD_MASK | index
        } else {
            index
        }
    }

    fn from_bits(bits: u8) -> State {
        State {
            unread: bits & UNREAD_MASK != 0,
            index: (bits & INDEX_MASK) as usize,
        }
    }
}

pub fn triple_buffer<T: Clone>(value: &T) -> (Writer<T>, Reader<T>) {
    let state = State::to_bits(State {
        unread: false,
        index: 2,
    });

    let inner = Arc::new(Inner {
        state: AtomicU8::new(state),
        data: array::from_fn(|_| UnsafeCell::new(value.clone())),
    });

    let writer = Writer {
        inner: inner.clone(),
        current: 0,
        next: 1,
    };

    let reader = Reader {
        inner: inner.clone(),
        index: 0,
    };

    (writer, reader)
}

struct Inner<T> {
    state: AtomicU8,
    data: [UnsafeCell<T>; 3],
}

pub struct Writer<T> {
    inner: Arc<Inner<T>>,
    current: usize,
    next: usize,
}

unsafe impl<T: Send + Sync> Send for Writer<T> {}

impl<T> Writer<T> {
    pub fn next(&self) -> &T {
        let value = &self.inner.data[self.next];
        unsafe { &*value.get() }
    }

    pub fn next_mut(&mut self) -> &mut T {
        let value = &self.inner.data[self.next];
        unsafe { &mut *value.get() }
    }

    pub fn current(&self) -> &T {
        let value = &self.inner.data[self.current];
        unsafe { &*value.get() }
    }

    pub fn split(&mut self) -> (&T, &mut T) {
        let current = &self.inner.data[self.current];
        let next = &self.inner.data[self.next];

        unsafe { (&*current.get(), &mut *next.get()) }
    }

    pub fn unread(&self) -> bool {
        State::from_bits(self.inner.state.load(Ordering::Relaxed)).unread
    }

    pub fn swap(&mut self) {
        let new_state = State::to_bits(State {
            unread: true,
            index: self.next,
        });
        let old_state = self.inner.state.swap(new_state, Ordering::AcqRel);

        self.current = self.next;
        self.next = State::from_bits(old_state).index;
    }
}

pub struct Reader<T> {
    inner: Arc<Inner<T>>,
    index: usize,
}

unsafe impl<T: Send + Sync> Send for Reader<T> {}

impl<T> Reader<T> {
    pub fn get(&self) -> &T {
        let value = &self.inner.data[self.index];
        unsafe { &*value.get() }
    }

    pub fn unread(&self) -> bool {
        State::from_bits(self.inner.state.load(Ordering::Relaxed)).unread
    }

    pub fn update(&mut self) -> bool {
        if !self.unread() {
            return false;
        }

        let new_state = State::to_bits(State {
            unread: false,
            index: self.index,
        });
        let old_state = self.inner.state.swap(new_state, Ordering::AcqRel);

        self.index = State::from_bits(old_state).index;

        true
    }
}
