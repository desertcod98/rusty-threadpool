use std::{
    sync::{mpsc, Arc, Mutex},
    thread::{self, JoinHandle},
};

type Task = Box<dyn FnOnce() + Send>;

pub struct ThreadPool {
    threads: Vec<Option<JoinHandle<()>>>,
    sender: Option<mpsc::Sender<Task>>,
}

impl ThreadPool {
    pub fn new(n_threads: usize) -> Self {
        let mut threads = Vec::with_capacity(n_threads);

        let (sender, reciever) = mpsc::channel::<Task>();

        let reciever = Arc::new(Mutex::new(reciever));

        for _ in 0..n_threads {
            let reciever = Arc::clone(&reciever);
            let thread = thread::spawn(move || loop {
                let guard = reciever.lock().unwrap();
                match guard.recv() {
                    Ok(task) => {
                        std::mem::drop(guard); //unlocks resource for other threads
                        task()
                    }
                    Err(_) => break,
                }
            });
            threads.push(Some(thread));
        }

        ThreadPool {
            threads,
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let task = Box::new(f);
        self.sender.as_ref().unwrap().send(task).unwrap();
    }

    pub fn thread_count(&self) -> usize {
        self.threads.len()
    }
}
test
impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.sender.take().unwrap();

        for thread in &mut self.threads {
            thread.take().unwrap().join().unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let threadpool = crate::ThreadPool::new(10);
        assert_eq!(threadpool.thread_count(), 10);
    }
}
