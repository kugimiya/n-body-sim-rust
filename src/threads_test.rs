use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

fn main() {
    let mut slave_threads = Vec::new();
    let mut slave_senders: Vec<Sender<(i32, i32)>> = Vec::new();

    let slave_threads_count = 4;
    let (master_sender, master_receiver): (Sender<(i32, i32)>, Receiver<(i32, i32)>) = mpsc::channel();

    for thread_index in 0..slave_threads_count {
        let _thread_index = thread_index;
        let _master_sender = master_sender.clone();
        let (slave_sender, slave_receiver): (Sender<(i32, i32)>, Receiver<(i32, i32)>) = mpsc::channel();

        let slave_thread = thread::spawn(move || {
            thread::sleep(Duration::from_millis(1000));
            loop {
                let received = slave_receiver.recv();
                if received.is_ok() {
                    let (_, task_value) = received.unwrap();
                    _master_sender.send((_thread_index, task_value)).unwrap();
                }
            }
        });

        slave_senders.push(slave_sender.clone());
        slave_threads.push(slave_thread);
    }

    for slave in slave_senders {
        slave.send((0, 2)).unwrap();
    }

    for received in master_receiver {
        println!("Received: {:?}", received);
    }    
}
