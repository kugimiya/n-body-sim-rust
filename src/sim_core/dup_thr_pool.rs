use std::{sync::mpsc::{self, Receiver, Sender}, thread::{self, JoinHandle}, time::Duration};
use super::{verlet_object::VerletObject, point::Point};

type SlaveMessage = (i32, Option<(i32, VerletObject, usize, Vec<VerletObject>)>);
type MasterMessage = (i32, (f64, f64), Vec<(f64, f64)>);

pub struct DuplexThreadPool {
    pub slave_threads: Vec<JoinHandle<()>>,
    pub slave_senders: Vec<Sender<SlaveMessage>>,
    pub master_sender: Sender<MasterMessage>,
    pub master_receiver: Receiver<MasterMessage>,
    pub threads_count: i32
}

impl DuplexThreadPool {
    pub fn new(threads_count: i32) -> DuplexThreadPool {
        let mut slave_threads = Vec::new();
        let mut slave_senders: Vec<Sender<SlaveMessage>> = Vec::new();
        let (master_sender, master_receiver): (Sender<MasterMessage>, Receiver<MasterMessage>) = mpsc::channel();

        for thread_index in 0..threads_count {
            let _thread_index = thread_index;
            let _master_sender = master_sender.clone();
            let (slave_sender, slave_receiver): (Sender<SlaveMessage>, Receiver<SlaveMessage>) = mpsc::channel();

            let slave_thread = thread::spawn(move || {
                let gravity_const = 6.67;

                loop {
                    // thread::sleep(Duration::from_millis(1));
                    let received = slave_receiver.recv();

                    if !received.is_ok() {
                        continue;
                    }

                    let (task_id, payload) = received.unwrap();

                    // println!("DEBUG: thread#{} got task#{}", _thread_index, task_id);

                    if task_id == 0 {
                        break;
                    }

                    if task_id == 1 && !payload.is_none() {
                        let mut results: Vec<(f64, f64)> = Vec::new();
                        let (thread_id, mut object1, i, objects) = payload.unwrap();

                        for j in 0 .. objects.len() {
                            if i == j {
                                continue;
                            }

                            let object2 = &mut objects.get(j).clone().unwrap();

                            let mut velocity = object1.position.minus(Point::new(object2.position.0, object2.position.1));
                            let velocity_squared = velocity.length_square();
                            let force = gravity_const * ((object1.mass * object2.mass) / velocity_squared);
                            let acceleration = force / f64::sqrt(velocity_squared);

                            let object1_acc = object2.position.clone().minus(Point::new(object1.position.0, object1.position.1)).multiply(acceleration);
                            let mut object2_acc = object1.position.minus(Point::new(object2.position.0, object2.position.1)).multiply(acceleration);

                            object1.accelerate(object1_acc);
                            results.push(object2_acc.as_tupl());
                        }

                        _master_sender.send((thread_id, object1.acceleration.as_tupl(), results)).unwrap();
                    }
                }
            });

            slave_senders.push(slave_sender.clone());
            slave_threads.push(slave_thread);
        }

        return DuplexThreadPool {
            master_receiver,
            master_sender,
            slave_senders,
            slave_threads,
            threads_count
        };
    }

    pub fn close_pool(&mut self) {
        for slave in self.slave_senders.iter() {
            slave.send((0, Option::None)).unwrap();
            std::mem::drop(slave);
        }
    }
}
