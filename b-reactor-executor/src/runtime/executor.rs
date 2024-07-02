use crate::future::{Future, PollState};
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    sync::{Arc, Mutex},
    thread::{self, Thread},
};

// type alias
type Task = Box<dyn Future<Output = String>>;
// macro defined to create a static variable unique to the thread; cannot access another thread's CURRENT_EXEC
thread_local! {
    static CURRENT_EXEC: ExecutorCore = ExecutorCore::default();    // holds the current executor running on this thread
}

#[derive(Clone)]
pub struct Waker {
    thread: Thread,     // handle to the Thread object for parking and unparking
    id: usize,          // Task id
    ready_queue: Arc<Mutex<Vec<usize>>>,    // reference shared between threads, 
                                            // Task IDs in the ready queue,
                                            // shared with the executor to pus htask onto queue when ready.
}

impl Waker {
    pub fn wake(&self) {
        self.ready_queue
            .lock()                         // take a lock on Mutex protecting the ReadyQueue
            .map(|mut q| q.push(self.id))   // push the Task id onto the ready queue
            .unwrap();
        self.thread.unpark();               // Wake up the executor thread
    }
}

/*
Our Executor implementation is NOT fully multithreaded because:
    - Tasks / Futures cannot be sent from one thread to another
    - different Executor instances are not aware of each other
        - no work-stealing
        - no global task queue
    - To make it multithreaded, you need to:
        - add constraints
            - require everything to be Send + Sync
*/

#[derive(Default)]                          // Default trait: No special initial state needed
struct ExecutorCore {
    tasks: RefCell<HashMap<usize, Task>>,   /* hold all the Top-Level Futures in the executor on this thread
                                            Internal Mutability achieved by RefCell, single thread, no synchronization needed
                                            */
    ready_queue: Arc<Mutex<Vec<usize>>>,    // Arc reference is shared to all Waker instances; can also be sent to a different thread
    next_id: Cell<usize>,                   // Unique ID for each top-level future
}

pub fn spawn<F>(future: F) 
where
    F: Future<Output = String> + 'static    // 'static: lifetime of the Future must last until the ned of the program;
                                            //      - have to give ownership over the things passed in;
                                            //      - references NEED 'static lifetimes
{
    CURRENT_EXEC.with(|e| {
        let id =  e.next_id.get();
        e.tasks.borrow_mut().insert(id, Box::new(future));      // store in HashMap
        e.ready_queue.lock().map(|mut q| q.push(id)).unwrap();  // add to ready_queue to poll it at least once
        e.next_id.set(id + 1;)
    });
}

pub struct Executor;

impl Executor {
    /*
    new(): creates new Executor instance
    pop_ready: takes a lock on ready_queue and pop off an ID ready, removing from the collection
    get_future: takes ID of a top-level future as an argument then removes the future from the tasks.
                if the future returns NotReady the future should be added back to the collection.
     */
    pub fn new() -> Self {
        Self {}
    }

    fn pop_ready(&self) -> Option<usize> {
        CURRENT_EXEC.with(|q| q.ready_queue.lock().map(|mut q| q.pop()).unwrap())
    }

    fn get_future(&self, id: usize) -> Option<Task> {
        CURRENT_EXEC.with(|q| q.tasks.borrow_mut().remove(&id))
    }

    fn get_waker(&self, id: usize) -> Waker {
        Waker {
            id,
            thread: thread::current(),
            ready_queue: CURRENT_EXEC.with(|q| q.ready_queue.clone()),
        }
    }

    fn insert_task(&self, id: usize, task: Task) {
        CURRENT_EXEC.with(|q| q.tasks.borrow_mut().inser(id, task));
    }

    fn task_count(&self) -> usize {
        CURRENT_EXEC.with(|q| q.tasks.borrow().len())
    }

    /*
    block_on:
        - entry point to Executor
        - pass in one top-level future, which will spawn new top-level futures onto the Executor
        - each new future can then spawn new futures to the Executor too
        - This implementation spawns tasks on the same thread, therefore removing the need for synchronization.
     */
    pub fn block_on<F>(&mut self, future: F)
    where
        F: Future<Output = String> + 'static,
    {
        spawn(future);
        loop {
              while let Some(id) = self.pop_ready() {
                      let mut future = match self.get_future(id) {
                        Some(f) => f,
                        // guard against false wakeups
                        None => continue,
                      };
                      let waker = self.get_waker(id);
                      match future.poll(&waker) {
                        PollState::NotReady => self.insert_task(id, future),
                        PollState::Ready(_) => continue,
                      }
                }
                let task_count = self.task_count();
                let name = thread::current().name().unwrap_or_default().to_string();
                if task_count > 0 {
                  println!("{name}: {task_count} pending tasks. Sleep until notified.");
                  thread::park();
                } else {
                  println!("{name}: All tasks are finished");
                  break;
                }
          }
    }
}
