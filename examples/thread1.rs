use anyhow::Result;
use std::{sync::mpsc, thread, time::Duration};

const NUM_PRODUCERS: usize = 4;

#[derive(Debug)]
struct Msg {
    idx: usize,
    value: usize,
}
impl Msg {
    fn new(idx: usize, value: usize) -> Self {
        Self { idx, value }
    }
}

fn main() -> Result<()> {
    let (tx, rx) = mpsc::channel();

    // 创建 producer 线程
    for i in 0..NUM_PRODUCERS {
        let tx = tx.clone();
        thread::spawn(move || producer(i, tx));
    }
    drop(tx); //释放tx, 否则rx无法结束

    // 创建 consumer 线程
    let consumer = thread::spawn(move || consumer(rx));

    let secret = consumer.join().unwrap()?;
    println!("consumer: secret:{}", secret);
    // thread::sleep(Duration::from_secs(5));
    Ok(())
}

fn producer(idx: usize, tx: mpsc::Sender<Msg>) -> Result<()> {
    loop {
        let value = rand::random::<usize>();
        tx.send(Msg::new(idx, value))?;
        let sleep_time = rand::random::<u64>() % 1000;
        thread::sleep(Duration::from_millis(sleep_time));
        //random exit the producer
        if rand::random::<u8>() % 10 == 0 {
            println!("producer: idx:{}, exit", idx);
            break;
        }
    }
    Ok(())
}

fn consumer(rx: mpsc::Receiver<Msg>) -> Result<usize> {
    for msg in rx {
        println!("consumer: idx:{}, value:{}", msg.idx, msg.value);
    }
    println!("consumer: exit");
    Ok(42)
}
