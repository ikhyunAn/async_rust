pub trait Future {
    type Output;
    fn poll(&mut self) -> PollState<Self::Output>;
}

pub enum PollState<T> {
    Ready(T),
    NotReady,
}

// c-coroutine code below

// pub struct JoinAll<F: Future> {
//     futures: Vec<(bool, F)>,
//     finished_count: usize,
// }

// impl<F: Future> Future for JoinAll<F> {
//     type Output = String;  // to match Corofy
    
//     // Future is lazy, initial JoinAll::poll will kick off all futures in the collection
//     fn poll(&mut self) -> PollState<Self::Output> {
//         // Iterate over (bool, F) tuples to poll each poll and track finisehd futures
//         for (finished, fut) in self.futures.iter_mut() {
//             if *finished {
//                 continue;
//             }
//             match fut.poll() {
//                 PollState::Ready(_) => {
//                     *finished = true;
//                     self.finished_count += 1;
//                 }
//                 PollState::NotReady => continue,
//             }
//         }
//         if self.finished_count == self.futures.len() {
//             PollState::Ready(String::new())
//         } else {
//             PollState::NotReady
//         }
//     }
// }

// /*
// `join_all`
// - arg: a collection of futures
// - return: `JoinAll<F> future`
// Creates a new collection which contains tuples conisting of: Vec<(bool, F)>
//     - original future received
//     - boolean indicating if the future is resolved or not
// */
// pub fn join_all<F: Future>(futures: Vec<F>) -> JoinAll<F> {
//     let futures = futures.into_iter().map(|f| (false, f)).collect();
//     JoinAll {
//         futures,
//         finished_count: 0,
//     }
// }
