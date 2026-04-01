# Shared Memory SPSC Ring Buffer IPC demo

Companion code for the blog post on [bangnbx.github.io](https://bangnbx.github.io).

Lock-free single-producer single-consumer ring buffer over shared memory (`/dev/shm`).
Measures end-to-end latency using `quanta` (RDTSC-based timestamps).

The design supports SPMC broadcast — multiple consumers can read from the same ring buffer independently by tracking their own tail. See the blog post for details.

## Structure

```
src/
  lib.rs              # SharedRing, Message, Consumer, shm helpers
  bin/
    producer.rs       # Writes messages with embedded timestamps
    consumer.rs       # Reads messages, measures latency
```

## Build

```bash
cargo build --release
```

## Run

Terminal 1:
```bash
./target/release/producer
```

Terminal 2:
```bash
./target/release/consumer
```

## What It Measures

The producer stores a `quanta` raw timestamp in each message right before writing it to the buffer.
The consumer reads that timestamp and compares it to its own `quanta` clock to compute one-way latency.
Both clocks read the CPU's TSC register — no syscall, nanosecond resolution.

## Configuration

- `IPC_CAPACITY`: 4096 slots (in `lib.rs`)
- `SHM_PATH`: `/dev/shm/ipc_demo_ring`
- Producer rate: ~1000 msg/sec (1ms sleep)

## Cleanup

```bash
rm /dev/shm/ipc_demo_ring
```
