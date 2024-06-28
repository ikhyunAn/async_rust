pub trait Future {
    type Ouput;
    fn poll(&mut self) -> PollState<Self::Output>;
}

pub enum PollState {
    Ready(T),
    Pending,
}

