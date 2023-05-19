use rand::RngCore;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

static mut RESOURCES: Vec<Mutex<()>> = Vec::new();
static mut LOCKS: Vec<Option<MutexGuard<()>>> = Vec::new();
static mut EATING: Vec<bool> = Vec::new();

fn phil(i: usize, s: Sender<usize>, r: Receiver<()>, o_m: Arc<Mutex<()>>) {
    let mut rng = rand::thread_rng();
    loop {
        // Sleep
        let sleep_dur = Duration::from_millis(1000 + rng.next_u32() as u64 % 1000);
        {
            let _ = o_m.lock();
            println!("Philosopher {i} is thinking for {sleep_dur:?}");
        }
        std::thread::sleep(sleep_dur);

        // Acquire
        {
            let _ = o_m.lock();
            println!("Philosopher {i} requests to take forks");
        }
        s.send(i).unwrap();
        r.recv().unwrap();

        println!("Philosopher {i} has taken forks");

        // Eat
        let sleep_dur = Duration::from_millis(1000 + rng.next_u32() as u64 % 1000);
        {
            let _ = o_m.lock();
            println!("Philosopher {i} is eating for {sleep_dur:?}");
        }
        std::thread::sleep(sleep_dur);

        // Release
        s.send(i).unwrap();
        r.recv().unwrap();
        {
            let _ = o_m.lock();
            println!("Philosopher {i} put forks back");
        }
    }
}

fn oracle(r: Receiver<usize>, s: Vec<Sender<()>>, _o_m: Arc<Mutex<()>>) {
    unsafe {
        for _ in 0..5 {
            RESOURCES.push(Mutex::new(()));
            LOCKS.push(None);
            EATING.push(false);
        }
    }

    loop {
        let i = r.recv().unwrap();
        let sender = s[i].clone();
        std::thread::spawn(move || unsafe {
            if EATING[i] {
                LOCKS[i] = None;
                LOCKS[(i + 1) % 5] = None;
            } else {
                LOCKS[i] = Some(RESOURCES[i].lock().unwrap());
                LOCKS[(i + 1) % 5] = Some(RESOURCES[(i + 1) % 5].lock().unwrap());
            }
            EATING[i] = !EATING[i];
            sender.send(()).unwrap();
        });
    }
}

fn main() {
    let output_mutex = Arc::new(Mutex::new(()));
    let om1 = output_mutex.clone();
    let om2 = output_mutex.clone();
    let om3 = output_mutex.clone();
    let om4 = output_mutex.clone();
    let om5 = output_mutex.clone();

    let (sender, receiver) = channel();
    let se1 = sender;
    let se2 = se1.clone();
    let se3 = se1.clone();
    let se4 = se1.clone();
    let se5 = se1.clone();
    let (s1, r1) = channel();
    let (s2, r2) = channel();
    let (s3, r3) = channel();
    let (s4, r4) = channel();
    let (s5, r5) = channel();

    let oracle_handle = std::thread::spawn(move || {
        oracle(receiver, vec![s1, s2, s3, s4, s5], output_mutex.clone())
    });
    let p1_handle = std::thread::spawn(move || phil(0, se1, r1, om1));
    let p2_handle = std::thread::spawn(move || phil(1, se2, r2, om2));
    let p3_handle = std::thread::spawn(move || phil(2, se3, r3, om3));
    let p4_handle = std::thread::spawn(move || phil(3, se4, r4, om4));
    let p5_handle = std::thread::spawn(move || phil(4, se5, r5, om5));

    let _ = oracle_handle.join();
    let _ = p1_handle.join();
    let _ = p2_handle.join();
    let _ = p3_handle.join();
    let _ = p4_handle.join();
    let _ = p5_handle.join();
}
