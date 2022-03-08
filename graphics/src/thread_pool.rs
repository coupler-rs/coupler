use std::any::Any;
use std::marker::PhantomData;
use std::mem;
use std::panic::{self, AssertUnwindSafe};
use std::ptr;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle, Thread};

use crossbeam_channel::Sender;

type Task = Box<dyn FnOnce() + Send>;

pub struct ThreadPool {
    handles: Vec<JoinHandle<()>>,
    sender: Option<Sender<Task>>,
}

impl ThreadPool {
    pub fn with_threads(num_threads: usize) -> ThreadPool {
        assert!(num_threads != 0);

        let (sender, receiver) = crossbeam_channel::unbounded::<Task>();

        let mut handles = Vec::with_capacity(num_threads);
        for _ in 0..num_threads {
            let receiver = receiver.clone();

            let handle = thread::spawn(move || {
                while let Ok(task) = receiver.recv() {
                    task();
                }
            });

            handles.push(handle);
        }

        ThreadPool { handles, sender: Some(sender) }
    }

    pub fn scope<'s, F>(&self, f: F)
    where
        F: FnOnce(&Scope<'s>),
    {
        let scope = Scope {
            sender: self.sender.as_ref().unwrap().clone(),
            task_count: Mutex::new(0),
            zero_tasks: Condvar::new(),
            panic: Mutex::new(None),
            phantom: PhantomData,
        };

        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            f(&scope);
        }));

        {
            let mut task_count = scope.task_count.lock().unwrap();
            while *task_count != 0 {
                task_count = scope.zero_tasks.wait(task_count).unwrap();
            }
        }

        if let Some(err) = scope.panic.lock().unwrap().take() {
            panic::resume_unwind(err);
        }

        if let Err(err) = result {
            panic::resume_unwind(err);
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for handle in self.handles.drain(0..self.handles.len()) {
            handle.join().unwrap();
        }
    }
}

pub struct Scope<'s> {
    sender: Sender<Task>,
    task_count: Mutex<usize>,
    zero_tasks: Condvar,
    panic: Mutex<Option<Box<dyn Any + Send>>>,
    phantom: PhantomData<fn(&'s ())>,
}

impl<'s> Scope<'s> {
    pub fn spawn<F>(&self, task: F)
    where
        F: FnOnce(&Scope<'s>) + Send + 's,
    {
        let task: Box<dyn FnOnce() + Send> = Box::new(move || {
            let result = panic::catch_unwind(AssertUnwindSafe(move || {
                task(self);
            }));

            {
                let mut task_count = self.task_count.lock().unwrap();
                *task_count -= 1;
                if *task_count == 0 {
                    self.zero_tasks.notify_one();
                }
            }

            if let Err(err) = result {
                let mut panic = self.panic.lock().unwrap();
                if panic.is_none() {
                    *panic = Some(err);
                }
            }
        });
        let task: Box<dyn FnOnce() + Send + 'static> = unsafe { mem::transmute(task) };

        *self.task_count.lock().unwrap() += 1;
        self.sender.send(task).unwrap();
    }
}
