use crate::runtime::Waker;
use mio::{net::TcpStream, Events, Interest, Poll, Registry, Token};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex, OnceLock,
    },
    thread,
};


/*
mio::Registry
    - regiser an event::Source with the `Poll` instance
    - registered `Poll` instance will monitor the event soruce for readiness state change
    pub fn register<S>(
        &self,
        source: &mut S,             // source of events that the Poll should monitor
        token: Token,               // picked by caller to associate with the socket. poll returns this with the event for the handle
        interests: Interest,        // specifies which operations `Poll` should monitor for readiness
    ) -> Result<()>

*/

// type alias for the Wakers collection
type Wakers = Arc<Mutex<HashMap<usize, Waker>>>;

// static variable, possible to access from idfferent threads
// OnceLock: allows defining a static variable that we can write to once initialized in the start of Reactor
static REACTOR: OnceLock<Reactor> = OnceLock::new();

// public function allows accessing REACTOR
pub fn reactor() -> &'static Reactor {
    REACTOR.get().expect("Called outside an runtime context")
}

// Reactor Struct
pub struct Reactor {
    wakers: Wakers,         // HashMap of Waker objects
    registry: Registry,     // Registry instance to interact with event queue in `mio`
    next_id: AtomicUsize,   
}

impl Reactor {
    // id is used to identify which event occurs
    pub fn register(&self, stream: &mut TcpStream, interest: Interest, id: usize) {
        self.registry.register(stream, Token(id), interest).unwrap();
    }

    // The most recent Waker should be stored; old Waker will be dropped
    pub fn set_waker(&self, waker: &Waker, id: usize) {
        let _ = self.wakers
                    .lock()
                    .map(|mut w| w.insert(id, waker.clone()).is_none())
                    .unwrap();
    }

    // Removes the Waker from wakers collection and deregsiter TcpStream from the Poll instance
    pub fn deregister(&self, stream: &mut TcpStream, id: usize) {
        self.wakers.lock().map(|mut w| w.remove(&id)).unwrap();
        self.registry.deregister(stream).unwrap();              // Poll instance no longer monitors for readiness state change; 
                                                                // interanal resources are cleared up
    }

    pub fn next_id(&self) -> usize {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }
}

// logic for event loop that waits and reacts to new events
fn event_loop(mut poll: Poll, wakers: Wakers) {
    // 
    let mut events = Events::with_capacity(100);

    // loop never ends; this could be fixed but not necessary for this runtime
    loop {
        poll.poll(&mut events, None).unwrap();          // timeout None: never time out and block until it receives an event notification
        // loop through every events received by poll.poll()
        for e in events.iter() {
            let Token(id) = e.token();
            let wakers = wakers.lock().unwrap();        // get the id-associated waker
            if let Some(waker) = wakers.get(&id) {      // wake only if the waker exists; non-existent if we removed it from the collection already
                waker.wake();
            }
        }
    }
}

// initializes and starts the runtime
pub fn start() {
    use thread::spawn;          // import spawn form std::thread
    let wakers = Arc::new(Mutex::new(HashMap::new()));
    let poll = Poll::new().unwrap();
    let registry = poll.registry().try_clone().unwrap();    // own Registry
    let next_id = AtomicUsize::new(1);
    let reactor = Reactor {
        wakers: wakers.clone(),
        registry,
        next_id,
    };
    REACTOR.set(reactor).ok().expect("Reactor already running");
    // spawn a new OS thread and start `event_loop`
    spawn(move || event_loop(poll, wakers));    // returns a JoinHandler, can be used to join the thread
    /*
    Example of using the JoinHandler:
        ```
        let handler = spawn(|| { ... });
        handler.join().unwrap();                // join waits for the spawned thread to finish; need a logic in event_loop to finish and return
        ```
     */
}