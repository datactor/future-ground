use std::{thread, time::Duration};

pub fn task_1(input: usize) -> usize {
    thread::sleep(Duration::from_secs(1));
    println!("task_1 finished..");
    input
}

pub fn task_2(input: String) -> String {
    thread::sleep(Duration::from_secs(3));
    println!("task_2 finished..");
    input
}

pub fn task_3(input: String) -> String {
    input
}

pub fn heavy_loop_with_sleep() -> u64 {
    let mut x = 0;
    for i in 0..1_000_000_000 {
        x += i;
        if i % 1_000_000 == 0 {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
    println!("heavy_loop finished..");
    x
}

pub fn heavy_loop() -> u64 {
    let mut x: u64 = 0;
    for i in 0..1_000_000_00 {
        x += i;
    }
    println!("heavy_loop finished..");
    x
}
