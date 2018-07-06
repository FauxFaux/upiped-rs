use std::io::Read;

use failure::Error;
use mio;
use mio::net::TcpListener;
use mio::net::TcpStream;
use mio::Token;

pub fn serve() -> Result<(), Error> {
    const SERVER: Token = Token(0);
    const CLIENT: Token = Token(1);

    let addr = "127.0.0.1:13265".parse()?;
    let server = TcpListener::bind(&addr)?;
    let poll = mio::Poll::new()?;
    poll.register(
        &server,
        SERVER,
        mio::Ready::readable(),
        mio::PollOpt::edge(),
    )?;

    let mut events = mio::Events::with_capacity(1024);

    let mut streams = Vec::new();

    loop {
        poll.poll(&mut events, None)?;

        for event in events.iter() {
            match event.token() {
                SERVER => {
                    let (stream, addr) = server.accept()?;
                    poll.register(
                        &stream,
                        Token(2),
                        mio::Ready::readable(),
                        mio::PollOpt::edge(),
                    )?;
                    streams.push(stream);
                    println!("reg");
                }
                Token(2) => {
                    let mut stream = streams.pop().unwrap();
                    let mut buf = [0u8; 16];
                    stream.read(&mut buf)?;
                    println!("{:?}", String::from_utf8_lossy(&buf));
                }
                _ => unreachable!(),
            }
        }
    }

    Ok(())
}
