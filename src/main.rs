use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::sync::mpsc::{self, TryRecvError};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:80")
        .expect("Couldn't connect to 'from' port");

    for mut incoming in listener.incoming() {
        match incoming {
            Ok(incoming) => {
                let mut outgoing = TcpStream::connect("127.0.0.1:8080")
                    .expect("Could not connect to 'to' port");

                let mut outgoing2 = outgoing.try_clone().expect("Could not clone");
                let incoming2 = incoming.try_clone().expect("Could not clone");

                let (tx, rx) = mpsc::channel();
                let t = thread::spawn(move || {
                    loop {
                        match rx.try_recv() {
                            Ok(_) | Err(TryRecvError::Disconnected) => {
                                println!("Ending");
                                break
                            },
                            Err(TryRecvError::Empty) => {}
                        }
                        print!("-");
                        stream_to_stream_blocking(&incoming, &outgoing)
                    }
                });
                stream_to_stream_blocking(&outgoing2, &incoming2);
                println!("Here");
                let _ = tx.send(());
                let _ = t.join();
            },
            Err(e) => eprintln!("Something went wrong {:?}", e),
        }
    }
}

fn stream_to_stream_blocking(from_stream: &TcpStream, mut to_stream: &TcpStream) {
    from_stream.bytes()
        .filter(|b| b.is_ok())
        .map(|b| b.unwrap())
        .map(|b| to_stream.write(&[b]))
        .filter(|r| r.is_err())
        .map(|r| r.err())
        .map(|e| e.unwrap())
        .for_each(|e| eprintln!("{:?}", e) );
}

fn stream_to_stream_non_blocking(mut from_stream: &TcpStream, mut to_stream: &TcpStream) -> Result<(), ()>{
    let mut buffer = [0; 1];

    return match from_stream.read(&mut buffer) {
        Ok(0) => Err(()),
        Ok(_) => {
            if let Err(e) = to_stream.write(&buffer) {
                eprintln!("Something bad happened writing to to_steam: {}", e);
                return Err(());
            }
            Ok(())
        },
        Err(e) => {
            eprintln!("Something bad happened reading from from_stream: {}", e);
            Err(())
        }
    }
}
