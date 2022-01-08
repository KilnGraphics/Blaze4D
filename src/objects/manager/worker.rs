use std::sync::Arc;
use std::thread::JoinHandle;

use concurrent_queue::ConcurrentQueue;

struct WorkerData {
    queue: ConcurrentQueue<()>,
}

impl WorkerData {
    fn new() -> Self {
        Self {
            queue: ConcurrentQueue::unbounded(),
        }
    }

    /// After this function is called no new requests may be pushed into the worker
    pub fn close(&self) {
        self.queue.close();
    }
}

struct Worker {
    thread: Option<JoinHandle<()>>,
    data: Arc<WorkerData>,
}

impl Worker {
    pub fn spawn() -> Self {
        let data = Arc::new(WorkerData::new());
        let cloned_data = data.clone();
        Self {
            thread: Some(std::thread::spawn(move|| Self::run(cloned_data))),
            data,
        }
    }

    fn run(data: Arc<WorkerData>) {

    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.data.close();
        self.thread.take().unwrap().join().unwrap();
    }
}