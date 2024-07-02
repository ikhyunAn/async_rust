use crate::future::{Future, PollState};
use mio::{Events, Poll, Registry};
use std::sync::OnceLock;

// REGISTRY: register interest in events with `Poll` instance
// register interest in TCP stream when making the actual HTTP GET request
// Have Http::get accept a Registry struct to be stored for later use
// Can we have access to Registry inside HttpGetFuture without passing around the reference?

/*
std::sync::OnceLock
    - A synchronization primitive which can be written to only once.
    - thread-safe `OnceCell`, can be used in statics
    - initialize REGISTRY when the runtime starts
    - prevent from calling Http::get without having a Runtime instance running (otherwise panic)
*/
static REGISTRY: OnceLock<Registry>  = OnceLock::new();
pub fn registry() -> &'static Registry {
    REGISTRY.get().expect("Called outside a runtime context")
}

pub struct Runtime {
    poll: Poll,
}


impl Runtime {
    /*
    new()
        - initialize runtime
        - new Poll instance: get owned version of Registry by cloning
        - store registry in the REGISTRY global variable to access it from the http module
     */
    pub fn new() -> Self {
        let poll = Poll::new().unwrap();
        let registry = poll.registry().try_clone().unwrap();
        REGISTRY.set(registry).unwrap();
        Self { poll }
    }

    /*
    block_on()
        - takes generic argument which block on anything that implements the Future trait with String Output
     */
    pub fn block_on<F>(&mut self, future: F)
    where
        F: Future<Output = String>,
    {
        let mut future = future;
        
        // loop until the top-level future returns Ready
        loop {
            match future.poll() {
                PollState::NotReady => {
                    println!("Schedule other tasks\n");
                    /*
                    `Events` collection to `mio::Poll::poll` method
                        - Events: used to store information about I/O events since the last `poll`
                        - represent "Readiness" for I/O operations
                        - provides a way to react to I/O readiness instead of constant polling
                    `poll.poll()`
                        - ask `mio` to wait for I/O events and store in `Events` collection
                        - for each event occur, wake up corresponding futures using waker mechanism
                    NOTE, there is only one top level Future to run
                    */
                    let mut events = Events::with_capacity(100);    // FIXME: create events only once outside the loop
                    self.poll.poll(&mut events, None).unwrap();     // 

                    /*
                    for event in events.iter() {
                        let token = event.token();
                        // Use the token to identify which operation is ready
                        // Wake up the corresponding future
                        // This part would typically involve a task queue or waker mechanism
                    }
                     */
                }
                PollState::Ready(_) => break,
            }
        }
    }
}