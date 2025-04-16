use futures::future::FutureExt;
use futures::select;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread;
use std::time::Duration;

enum State<T> {
    Init(Option<Box<dyn FnOnce() -> T + Send>>),
    Running,
    Done(Option<T>),
}

struct Inner<T> {
    state: State<T>,
    waker: Option<Waker>,
}

pub struct MyFuture<T> {
    shared: Arc<Mutex<Inner<T>>>,
}

impl<T: Send + 'static> MyFuture<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce() -> T + Send + 'static,
    {
        let shared = Arc::new(Mutex::new(Inner {
            state: State::Init(Some(Box::new(f))),
            waker: None,
        }));

        MyFuture { shared }
    }
}

impl<T: Send + 'static> Future for MyFuture<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        let mut inner = self.shared.lock().unwrap();

        match &mut inner.state {
            State::Init(f_opt) => {
                if let Some(f) = f_opt.take() {
                    let shared_clone = self.shared.clone();
                    thread::spawn(move || {
                        let result = f();
                        let mut inner = shared_clone.lock().unwrap();
                        inner.state = State::Done(Some(result));
                        if let Some(waker) = inner.waker.take() {
                            waker.wake();
                        }
                    });

                    inner.state = State::Running;
                }

                inner.waker = Some(cx.waker().clone());
                Poll::Pending
            }
            State::Running => {
                inner.waker = Some(cx.waker().clone());
                Poll::Pending
            }
            State::Done(result) => {
                let value = result.take().unwrap();
                Poll::Ready(value)
            }
        }
    }
}

fn task_1(input: usize) -> usize {
    thread::sleep(Duration::from_secs(1));
    input
}

fn task_2(input: String) -> String {
    thread::sleep(Duration::from_secs(2));
    input
}

fn task_3(input: String) -> String {
    input
}

pub async fn return_first() -> String {
    let fut1 = MyFuture::new(|| task_1(1).to_string()).fuse();
    let fut2 = MyFuture::new(|| task_2("hello".into())).fuse();
    let fut3 = MyFuture::new(|| task_3("world".into())).fuse();

    futures::pin_mut!(fut1, fut2, fut3);

    select! {
        result = fut1 => result,
        result = fut2 => result,
        result = fut3 => result,
    }
}
