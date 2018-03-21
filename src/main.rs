use std::io::prelude::*;
use std::net::{TcpListener, TcpStream, Shutdown};
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

                let (tx1, rx1) = mpsc::channel();
                let (tx2, rx2) = mpsc::channel();

                let t1 = thread::spawn(move || {
                    loop {
                        match rx2.try_recv() {
                            Ok(_) | Err(TryRecvError::Disconnected) => {
                                println!("Ending");
                                break
                            },
                            Err(TryRecvError::Empty) => {}
                        }
                        match stream_to_stream_non_blocking(&incoming, &outgoing) {
                            Err(_) => break,
                            _ => {}
                        }
                    }
                    tx1.send(());
                    incoming.shutdown(Shutdown::Read);
                    outgoing.shutdown(Shutdown::Write);
                });
                let t2 = thread::spawn(move || {
                    loop {
                        match rx1.try_recv() {
                            Ok(_) | Err(TryRecvError::Disconnected) => {
                                println!("Ending");
                                break
                            },
                            Err(TryRecvError::Empty) => {}
                        }
                        match stream_to_stream_non_blocking(&outgoing2, &incoming2) {
                            Err(_) => break,
                            _ => {}
                        }
                    }
                    tx2.send(());
                    outgoing2.shutdown(Shutdown::Read);
                    incoming2.shutdown(Shutdown::Write);
                });

                println!("Here");
                let _ = t1.join();
                let _ = t2.join();
                println!("All done")
            },
            Err(e) => eprintln!("Something went wrong {:?}", e),
        }
    }
}

//fn stream_to_stream_blocking(from_stream: &TcpStream, mut to_stream: &TcpStream) {
//    from_stream.bytes()
//        .filter(|b| b.is_ok())
//        .map(|b| b.unwrap())
//        .map(|b| to_stream.write(&[b]))
//        .filter(|r| r.is_err())
//        .map(|r| r.err())
//        .map(|e| e.unwrap())
//        .for_each(|e| eprintln!("{:?}", e) );
//}

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
