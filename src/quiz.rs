use std::thread::{self, JoinHandle};
use std::time::Duration;

pub fn task_1(input: usize) -> JoinHandle<usize> {
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(1));
        input
    })
}

pub fn task_2(input: String) -> JoinHandle<String> {
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(5));
        input
    })
}

pub fn return_first(input: usize, input2: String) -> MyStruct {
    let task_1_handle = task_1(input);
    let task_2_handle = task_2(input2);

    loop {
        if task_1_handle.is_finished() {
            return MyStruct::Task1(task_1_handle.join().unwrap());
        }

        if task_2_handle.is_finished() {
            return MyStruct::Task2(task_2_handle.join().unwrap());
        }
    }
}

#[derive(Debug)]
pub enum MyStruct {
    Task1(usize),
    Task2(String),
}
