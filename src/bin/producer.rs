use shared_mem_demo::*;
use std::thread;
use std::time::Duration;

fn main() {
    // core_affinity::set_for_current(core_affinity::CoreId { id: 10 });
    let size = std::mem::size_of::<SharedRing>();
    println!("SharedRing size: {} bytes ({:.1} MB)", size, size as f64 / 1024.0 / 1024.0);
    println!("Message size: {} bytes", std::mem::size_of::<Message>());
    println!("Capacity: {} slots", IPC_CAPACITY);
    println!("SHM path: {}", SHM_PATH);

    // Create and initialize shared memory
    let ptr = create_shm(SHM_PATH, size);
    let ring = ptr as *mut SharedRing;
    SharedRing::init(ring);
    let ring_ref = unsafe { &*ring };

    println!("Ring buffer initialized. Producing messages...\n");

    let clock = quanta::Clock::new();
    let mut rng = rand::rng();
    let mut count: u64 = 0;

    loop {
        let msg = Message::new(&mut rng);
        ring_ref.push(msg, &clock);

        count += 1;
        if count % 10_000 == 0 {
            println!("Produced: {} messages", count);
        }

        // ~1000 msg/sec
        thread::sleep(Duration::from_millis(1));
    }
}
