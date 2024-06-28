use std::time::Instant;

mod http;
mod future;

use future::*;
use crate::http::Http;

/*
`request`
- arg: integer to create GET request
- wait for the GET reqest response and prints the result
*/
coroutine fn request(i: usize) {
    let path = format!("/{}/HelloWorld{i}", i * 1000);
    let txt = Http::get(&path).wait;
    println!("{txt}");
}

/*
`async_main`
- stores a set of coroutines 
- Creates JoinAll future and wait on it

Our use case:
    request with 0, 1, 2, 3, 4 is created
    our delayserver will process these requests by delaying with the given number of miliseconds (i * 1000 in `request`)
    All these requests will be run concurrently
    This program will end in approximately 4 seconds - actual output: "ELAPSED TIME: 4.017663"
 */
coroutine fn async_main() {
    println!("Program starting");
    let mut futures = vec![];

    for i in 0..5 {
        futures.push(request(i));
    }

    future::join_all(futures).wait;
}


fn main() {
    let start = Instant::now();
    let mut future = async_main();

    loop {
        match future.poll() {
            PollState::NotReady => (),
            PollState::Ready(_) => break,
        }
    }

    println!("\nELAPSED TIME: {}", start.elapsed().as_secs_f32());
}