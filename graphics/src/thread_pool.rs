use std::marker::PhantomData;
use std::mem;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::thread::{JoinHandle, Thread};

use crossbeam_channel::Sender;

type Task = Box<dyn FnOnce() + Send>;

struct TaskCount {
    count: AtomicUsize,
    thread: Thread,
}

pub struct ThreadPool {
    handles: Vec<JoinHandle<()>>,
    sender: Option<Sender<Task>>,
    task_count: Arc<TaskCount>,
}

impl ThreadPool {
    pub fn with_threads(num_threads: usize) -> ThreadPool {
        assert!(num_threads != 0);

        let (sender, receiver) = crossbeam_channel::unbounded::<Task>();

        let task_count =
            Arc::new(TaskCount { count: AtomicUsize::new(0), thread: thread::current() });

        let mut handles = Vec::with_capacity(num_threads);
        for _ in 0..num_threads {
            let receiver = receiver.clone();
            let task_count = task_count.clone();

            let handle = thread::spawn(move || {
                while let Ok(task) = receiver.recv() {
                    task();

                    if task_count.count.fetch_sub(1, Ordering::Release) == 1 {
                        task_count.thread.unpark();
                    }
                }
            });

            handles.push(handle);
        }

        ThreadPool { handles, sender: Some(sender), task_count }
    }

    pub fn scope<F>(&mut self, f: F)
    where
        F: FnOnce(&Scope),
    {
        f(&Scope { pool: self, phantom: PhantomData });

        while self.task_count.count.load(Ordering::Acquire) != 0 {
            thread::park();
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

pub struct Scope<'p, 's> {
    pool: &'p ThreadPool,
    phantom: PhantomData<fn(&'s ())>,
}

impl<'p, 's> Scope<'p, 's> {
    pub fn spawn<F>(&self, task: F)
    where
        F: FnOnce() + Send + 's,
    {
        let task: Box<dyn FnOnce() + Send> = Box::new(task);
        let task: Box<dyn FnOnce() + Send + 'static> = unsafe { mem::transmute(task) };

        self.pool.task_count.count.fetch_add(1, Ordering::Relaxed);

        self.pool.sender.as_ref().unwrap().send(task).unwrap();
    }
}
