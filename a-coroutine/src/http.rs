use crate::future::{Future, PollState};
use std::io::{ErrorKind, Read, Write};

fn get_req(path: &str) -> String {
    format!(
        "GET {path} HTTP/1.1\r\n\
            Host: localhost\r\n\
            Connection: close\r\n\
            \r\n"
    )
}

pub struct Http;

impl Http {
    // public interface, returns an implementation of Future with a String output
    pub fn get(path: &str) -> impl Future<Output = String> {
        HttpGetFuture::new(path)
    }
}

struct HttpGetFuture {
    stream: Option<mio::net::TcpStream>,    // optional TCP stream for the connection
    bufer: Vec<u8>,                         // buffer to store data
    path: String,                           // path for HTTP GET request
}

impl HttpGetFuture {
    fn new(path: &'static str) -> Self {
        Self {
            stream: None,
            buffer: vec![],
            path: path.to_string(),
        }
    }

    // sends the GET request to the server
    fn write_request(&mut self) {
        // establish TCP connection
        let stream = std::net::TcpStream::connect("127.0.0.1:8080").unwrap();
        stream.set_nonblocking(true).unwrap();
        let mut stream = mio::net::TcpStream::from_std(stream);
        // sending the GET request
        stream.write_all(get_req(&self.path).as_bytest()).unwrap();
        self.stream = Some(stream);
    }
}

// Future trait implementation
impl Future for HttpGetFuture {
    type Output = String;
    
    // poll is called repeatedly to check the state of operation
    fn poll(&mut self) -> PollState<Self::Output> {
        if self.stream.is_none() {
            println!("FIRST POLL - START OPERATION");
            self.write_request();
            return PollState::NotReady;
        }
        // else
        let mut buff = vec![0u8; 4096];
        /*
        Consider this as a state machine with three states:
        1. self.stream => None
        2. self.stream => Some(WouldBlock)
        3. self.stream => Some(0)
         */
        loop {
            match self.stream.as_mut().unwrap().read(&mut buff) {
                // return with 0 bytes read
                Ok(0) => {
                    let s = String::from_utf8_lossy(&self.buffer);
                    break PollState::Ready(s.to_string());
                }
                // return with n bytes read - read the data into the buffer
                Ok(n) => {
                    self.buffer.extend(&buff[0..n]);    // read in data
                    continue;
                }
                // stream is set to nonblocking, poll again to finish the operation
                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    break PollState::NotReady;
                }
                // reads interrupted by a signal, try reading one more time (not poll)
                Err(e) if e.kind() == ErrorKind::Interrupted => {
                    continue;
                }
                Err(e) => panic!("{e:?}"),
            }
        }
    }
}