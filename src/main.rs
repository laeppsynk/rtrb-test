use std::hint;

use rtrb::Producer;
use thread::channel;

mod thread;

type DataType = i32;
type Sender<D> = thread::channel::Sender<D, DataType>;
type Receiver<O> = thread::channel::Receiver<O, DataType>;

pub(crate) type Result<T> = anyhow::Result<T>;

struct Thread {}

struct Main {}

impl crate::thread::channel::Origin for Main {}
impl crate::thread::channel::Origin for Thread {}
impl crate::thread::channel::Destination for Main {}
impl crate::thread::channel::Destination for Thread {}

fn main() -> Result<()> {
    let (mut to_thread, mut from_main): (Sender<Thread>, Receiver<Main>) =
        channel::bounded_default();
    let (mut to_main, mut from_thread): (Sender<Main>, Receiver<Thread>) =
        channel::bounded_default();

    println!("to_thread capacity: {}", to_thread.buffer().capacity());
    println!("from_main capacity: {}", from_main.buffer().capacity());
    println!("to_main capacity: {}", to_main.buffer().capacity());
    println!("from_thread capacity: {}", from_thread.buffer().capacity());

    std::thread::spawn(move || {
        thread(from_main, to_main).unwrap();
    });

    let mut i = 0;
    loop {
        while to_thread.is_full() {
            hint::spin_loop();
        }
        to_thread.push(i)?;
        println!("Pushed: {}", i);
        i += 1;
    }

    Ok(())
}

fn thread(mut from_main: Receiver<Main>, mut to_main: Sender<Main>) -> Result<()> {
    loop {
        while from_main.is_empty() {
            hint::spin_loop();
        }
        let data = from_main.pop()?;
        println!("Received: {}", data);
    }
}
