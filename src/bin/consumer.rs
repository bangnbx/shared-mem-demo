use shared_mem_demo::*;
use quanta::Clock;

const BATCH_SIZE: usize = 10_000;

fn main() {
    // core_affinity::set_for_current(core_affinity::CoreId { id: 22 });
    let size = std::mem::size_of::<SharedRing>();
    let ptr = open_shm_readonly(SHM_PATH, size);
    let ring = ptr as *const SharedRing;
    let mut consumer = Consumer::new(ring);

    println!("Consumer started. Waiting for messages...\n");

    let clock = Clock::new();
    let mut latencies = vec![0u64; BATCH_SIZE];
    let mut count: usize = 0;
    let mut total_lost: usize = 0;
    let mut round: u64 = 0;

    consumer.skip_to_head();

    loop {
        match consumer.pop() {
            Some((msg, lost)) => {
                let now = clock.raw();
                latencies[count] = clock.delta(msg.ts, now).as_nanos() as u64;
                count += 1;
                total_lost += lost;

                if count == BATCH_SIZE {
                    round += 1;
                    latencies.sort_unstable();

                    let avg = latencies.iter().sum::<u64>() / BATCH_SIZE as u64;
                    let min = latencies[0];
                    let p50 = latencies[BATCH_SIZE / 2];
                    let p90 = latencies[BATCH_SIZE * 90 / 100];
                    let p99 = latencies[BATCH_SIZE * 99 / 100];

                    println!(
                        "[round {}] avg: {} ns | min: {} ns | p50: {} ns | p90: {} ns | p99: {} ns | lost: {}",
                        round, avg, min, p50, p90, p99, total_lost
                    );

                    count = 0;
                    total_lost = 0;
                    consumer.skip_to_head();
                }
            }
            None => {
                std::hint::spin_loop();
                // thread::sleep(Duration::from_micros(100));
            }
        }
    }
}
