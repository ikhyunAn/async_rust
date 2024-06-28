use std::time::Instant;
mod http;
mod future;
use crate::http::Http;
use future::{Future, PollState};
use std::thread;
use std::time::Duration;


/*
Code Structure principles:
    1. Imports and module declarations at the top
    2. Main function near the top for visibility
    3. Type definitions (enum and struct) before their use
    4. Implementation blocks after type definitions
    5. Helper functions (like async_main) at the end
*/

/*
loop in the main function drives the asynchronous operations to completion

similar to:
async fn async_main() {
    println!("Async Program starting");
    let txt = Http::get("/1000/HelloWorld").await;
    println!("{txt}");
    let txt2 = Http::("500/HelloWorld2").await;
    println!("{txt2}");
}
*/
fn main() {
    let mut future = async_main();
    loop {
        match future.poll() {
            PollState::NotReady => {
                println!("Schedule other tasks");
            },
            PollState::Ready(_) => break,
        }
        thread::sleep(Duration::from_millis(100));      // simulate "other tasks"
    }
}

/* 
coroutine can be in four states:
- Start: Coroutine is created, yet to be polled
- Wait1: HttpGetFuture is returned upon calling Http::get, which we store in the enum State.
            This returns control back to the calling function
- Wait2: 2nd call to Http::get
- Resolved: no more work to do. Future is resolved.
*/
enum State {
    Start,
    Wait1(Box<dyn Future<Output = String>>),
    Wait2(Box<dyn Future<Output = String>>),
    Resolved,
}

struct Coroutine {
    // more state could be added to the Coroutine struct; otherwise just assign it to enum
    state: State,
}

impl Coroutine {
    fn new() -> Self {
        Self {
            state: State::Start,
        }    
    }    
}    

impl Future for Coroutine {
    type Output = ();
    
    fn poll(&mut self) -> PollState<Self::Output> {
        // loop matches the self.state, drives the state machine forward until job is done
        loop {
            match self.state {
                State::Start => {
                    println!("Program Starting");
                    let fut = Box::new(Http::get("/600/HelloWorld"));  // Http::get returns a future
                    self.state = State::Wait1(fut);                    // store the future
                }
                // still in a loop without breaking, so immediately checks if State::Wait1
                State::Wait1(ref mut fut) => match fut.poll() {         // poll the Future
                    PollState::Ready(txt) => {
                        // more operations could be done with the returned data
                        println!("{txt}");
                        let fut2 = Box::new(Http::get("/400/HelloWorld2"));
                        self.state = State::Wait2(fut2);
                    }
                    PollState::NotReady => break PollState::NotReady,   // break and return `NotReady` to the caller, to be polled again
                }
                State::Wait2(ref mut fut2) => match fut2.poll() {
                    PollState::Ready(txt2) => {
                        println!("{txt2}");
                        self.state = State::Resolved;
                        break PollState::Ready(());                     // communnicate to the caller that Future is done
                    }
                    PollState::NotReady => break PollState::NotReady,
                }
                State::Resolved => panic!("Polled a resolved Future!"),
            }
        }
    }
}


fn async_main() -> impl Future<Output = ()> {
    Coroutine::new()            // create a new Coroutine
}
  
