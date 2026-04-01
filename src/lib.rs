use std::sync::atomic::{AtomicUsize, Ordering};

pub const IPC_CAPACITY: usize = 4096;
pub const IPC_MASK: usize = IPC_CAPACITY - 1;
pub const SHM_PATH: &str = "/dev/shm/ipc_demo_ring";

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Message {
    pub key: u64,
    pub val: u64,
    pub ts: u64,
}

impl Message {
    pub fn new(rng: &mut impl rand::Rng) -> Self {
        let key: u64 = rng.random();
        let val: u64 = rng.random();
        Message { key, val, ts: 0 }
    }
}

#[repr(C, align(64))]
pub struct SharedRing {
    head: AtomicUsize,
    _pad1: [u8; 56],
    buffer: [Message; IPC_CAPACITY],
}

impl SharedRing {
    pub fn init(ptr: *mut SharedRing) {
        unsafe {
            (*ptr).head = AtomicUsize::new(0);
            std::ptr::write_bytes((*ptr).buffer.as_mut_ptr(), 0, IPC_CAPACITY);
        }
    }

    #[inline]
    pub fn push(&self, mut msg: Message, clock: &quanta::Clock) {
        let head = self.head.load(Ordering::Relaxed);
        msg.ts = clock.raw();
        unsafe {
            let ptr = self as *const SharedRing as *mut SharedRing;
            (*ptr).buffer[head & IPC_MASK] = msg;
        }
        self.head.store(head.wrapping_add(1), Ordering::Release);
    }

    #[inline]
    pub fn head(&self) -> usize {
        self.head.load(Ordering::Acquire)
    }
}

pub struct Consumer {
    ring: *const SharedRing,
    tail: usize,
}

impl Consumer {
    pub fn new(ring: *const SharedRing) -> Self {
        let head = unsafe { &*ring }.head.load(Ordering::Acquire);
        Self { ring, tail: head }
    }

    /// Pop the next message, returning (message, lost_count).
    /// lost_count > 0 means the producer lapped the consumer.
    #[inline]
    pub fn pop(&mut self) -> Option<(Message, usize)> {
        let head = unsafe { &*self.ring }.head(  );

        if head == self.tail {
            return None;
        }

        let diff = head.wrapping_sub(self.tail);
        let lost = if diff > IPC_CAPACITY {
            let skip = diff - IPC_CAPACITY;
            self.tail = head.wrapping_sub(IPC_CAPACITY);
            skip
        } else {
            0
        };

        let data = unsafe { (*self.ring).buffer[self.tail & IPC_MASK] };
        self.tail = self.tail.wrapping_add(1);
        Some((data, lost))
    }

    pub fn skip_to_head(&mut self) {
        let head = unsafe { &*self.ring }.head();
        self.tail = head;
    }
}

// Shared Memory helpers
pub fn create_shm(path: &str, size: usize) -> *mut u8 {
    use std::fs::OpenOptions;
    use std::os::unix::fs::OpenOptionsExt;

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)
        .expect("Failed to create shm file");

    file.set_len(size as u64).expect("Failed to set shm size");

    unsafe {
        let ptr = libc::mmap(
            std::ptr::null_mut(),
            size,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            std::os::unix::io::AsRawFd::as_raw_fd(&file),
            0,
        );
        if ptr == libc::MAP_FAILED {
            panic!("mmap failed");
        }
        // Lock pages in RAM to avoid page faults on hot path
        libc::mlock(ptr, size);
        ptr as *mut u8
    }
}

pub fn open_shm_readonly(path: &str, size: usize) -> *const u8 {
    use std::fs::OpenOptions;

    let file = OpenOptions::new()
        .read(true)
        .open(path)
        .expect("Failed to open shm file (is producer running?)");

    unsafe {
        let ptr = libc::mmap(
            std::ptr::null_mut(),
            size,
            libc::PROT_READ,
            libc::MAP_SHARED,
            std::os::unix::io::AsRawFd::as_raw_fd(&file),
            0,
        );
        if ptr == libc::MAP_FAILED {
            panic!("mmap failed");
        }
        libc::mlock(ptr, size);
        ptr as *const u8
    }
}
