use future_ground::{quiz, quiz_2, quiz_3, tasks};
use futures::FutureExt;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::task::JoinHandle;

#[tokio::main]
async fn main() {
    // let output = return_first(Duration::from_secs(10), Duration::from_secs(8)).await;
    // let output = quiz::return_first(1, "input2".to_string());
    // let output = quiz_2::return_first().await;

    let rt = quiz_3::MyRuntime::new();
    rt.spawn_task(|| {
        tasks::task_1(1);
    });

    rt.spawn_task(|| {
        tasks::task_2("hello".to_string());
    });

    rt.spawn_task(|| {
        tasks::heavy_loop();
    });

    rt.select_one();

    // println!("output: {:?}", output);
}

pub struct MyStruct {
    task_1: Option<JoinHandle<String>>,
    task_2: Option<JoinHandle<String>>,
}

impl Future for MyStruct {
    type Output = String;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        if let Some(mut task_1) = this.task_1.take() {
            match task_1.poll_unpin(cx) {
                Poll::Pending => {}
                Poll::Ready(value) => match value {
                    Ok(value) => {
                        this.task_2.take().unwrap().abort();
                        return Poll::Ready(value);
                    }
                    Err(error) => return Poll::Ready(error.to_string()),
                },
            }

            this.task_1.replace(task_1);
        }

        if let Some(mut task_2) = this.task_2.take() {
            match task_2.poll_unpin(cx) {
                Poll::Pending => {}
                Poll::Ready(value) => match value {
                    Ok(value) => {
                        this.task_1.take().unwrap().abort();
                        return Poll::Ready(value);
                    }
                    Err(error) => return Poll::Ready(error.to_string()),
                },
            }

            this.task_2.replace(task_2);
        }

        Poll::Pending
    }
}

pub fn return_first(t1: Duration, t2: Duration) -> MyStruct {
    let task_1 = tokio::spawn(async move {
        tokio::time::sleep(t1).await;
        String::from("task 1 finished..")
    });

    let task_2 = tokio::spawn(async move {
        tokio::time::sleep(t2).await;
        String::from("task 2 finished..")
    });

    MyStruct {
        task_1: Some(task_1),
        task_2: Some(task_2),
    }
}
