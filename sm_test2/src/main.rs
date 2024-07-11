/*
FSM that parses data into json format?

*/


enum State {
    Start,
    Processing,
    Done,
    Error,
}

struct StateMachine {
    state: State,
    data: String,
}

impl StateMachine {
    fn new() -> Self {
        StateMachine {
            state: State::Start,
            data: String::new(),
        }
    }

    fn process (&mut self, input: &str) {
        match self.state {
            State::Start => {
                self.data = input.to_string();
                self.state = State::Processing;
            }
            State::Processing => {
                if input == "done" {
                    self.state = State::Done;
                } else if input == "error" {
                    self.state = State::Error;
                } else {
                    self.data.push_str(input);
                }
            }
            State::Done | State::Error => {
                // No state change in these terminal states
            }
        }
    }

    fn get_result(&self) -> Option<&str> {
        match self.state {
            State::Done => Some(&self.data),
            State::Error => Some("Error by user"),
            _ => None,
        }
    }
}

fn main() {
    let mut machine = StateMachine::new();

    loop {
        use std::io::{stdin, stdout, Write};
        let mut s = String::new();
        println!("Enter some text!");
        let _ = stdout().flush();
        stdin().read_line(&mut s).expect("Did not enter a correct string");

        if let Some('\n') = s.chars().next_back() {
            s.pop();
        }
        if let Some('\r') = s.chars().next_back() {
            s.pop();
        }
        /* The above two if blocks can be replaced by the following `trim()` function */
        // let s = s.trim();
        if s == "done" {
            machine.process(&s);
            break;
        }
        if s == "error" {
            machine.process(&s);
            break;
        }
        machine.process(&s);
    }
    
    // machine.process("Hello, ");
    // machine.process("World!");
    // machine.process("done");
    
    if let Some(result) = machine.get_result() {
        println!("Result: {}", result);
    } else {
        println!("Processing not complete");
    }
}