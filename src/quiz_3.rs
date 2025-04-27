use libc::pthread_cancel;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{
    collections::VecDeque,
    fmt::Debug,
    os::unix::thread::JoinHandleExt,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

pub enum PollResult<T> {
    Pending,
    Ready(T),
    Cancelled,
}

pub trait Pollable {
    type Output;
    fn poll(&mut self) -> PollResult<Self::Output>;
    fn cancel(&mut self);
}

pub struct MyFuture<T: Send + 'static> {
    handle: Option<JoinHandle<()>>,
    result: Arc<Mutex<Option<T>>>,
    is_done: Arc<AtomicBool>,
}

impl<T: Send + 'static> MyFuture<T> {
    pub fn new<F>(task: F) -> Self
    where
        F: FnOnce() -> T + Send + 'static,
    {
        let result = Arc::new(Mutex::new(None));
        let is_done = Arc::new(AtomicBool::new(false));

        let result_clone = Arc::clone(&result);
        let is_done_clone = Arc::clone(&is_done);

        let builder = thread::Builder::new();

        let handle = builder
            .spawn(move || {
                let value = task();
                *result_clone.lock().unwrap() = Some(value);
                is_done_clone.store(true, Ordering::SeqCst);
            })
            .unwrap();

        Self {
            handle: Some(handle),
            result,
            is_done,
        }
    }
}

impl<T: Send + 'static> Pollable for MyFuture<T> {
    type Output = T;

    fn poll(&mut self) -> PollResult<Self::Output> {
        if self.is_done.load(Ordering::Relaxed) {
            let mut result = self.result.lock().unwrap();
            if let Some(val) = result.take() {
                return PollResult::Ready(val);
            }
        }
        PollResult::Pending
    }

    fn cancel(&mut self) {
        if let Some(handle) = &self.handle {
            unsafe {
                let pthread_id = handle.as_pthread_t();
                pthread_cancel(pthread_id);
                // pthread_cancel(self.pthread_id);
            }
        }
        self.handle = None;
    }
}

pub struct MyRuntime<T: Send + 'static + Debug> {
    queue: Arc<Mutex<VecDeque<MyFuture<T>>>>,
}

impl<T: Send + 'static + Debug> MyRuntime<T> {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn spawn_task<F>(&self, f: F)
    where
        F: FnOnce() -> T + Send + 'static,
    {
        let future = MyFuture::new(f);
        self.queue.lock().unwrap().push_back(future);
    }

    pub fn select_one(&self) -> Option<T> {
        loop {
            let mut queue = self.queue.lock().unwrap();

            for i in 0..queue.len() {
                let fut = &mut queue[i];
                match fut.poll() {
                    PollResult::Ready(val) => {
                        for (j, other) in queue.iter_mut().enumerate() {
                            if j != i {
                                other.cancel();
                            }
                        }
                        queue.clear();
                        return Some(val);
                    }
                    PollResult::Cancelled => continue, // Add this to handle cancellation or timeout
                    PollResult::Pending => continue,
                }
            }
        }
    }
}
