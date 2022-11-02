use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread::{self, JoinHandle};

struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            // recv() will block until a job becomes available
            let job = receiver.lock().unwrap().recv().unwrap();

            println!("Worker {id} got a job, executing now.");

            job();
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

impl ThreadPool {
    // TODO Write a build function that returns an Error instead of calling panic
    // pub fn build(size: usize) -> Result<ThreadPool, PoolCreationError>

    /// Create a new ThreadPool.
    ///
    /// The n_threads is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if n_threads is 0.
    pub fn new(n_threads: usize) -> ThreadPool {
        assert!(n_threads > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(n_threads);

        for id in 0..n_threads {
            // thread::spawn() expects code to execute, so use Worker instead

            // clone() bumbs the reference count so the workers can share
            // ownership of the receiver
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    // Send-trait to transfer the closure from one thread to another
    // 'static because no idea how long this thread executing the closure will live.
    // FnOnce to execute once
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
