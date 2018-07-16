use std::collections::HashMap;
use std::collections::VecDeque;
use std::io;
use std::io::Read;
use std::io::Write;

use failure::Error;
use mio;
use mio::net::TcpListener;
use mio::net::TcpStream;
use mio::Token;

struct Incoming {
    tcp: TcpStream,
    buf: VecDeque<VecDeque<u8>>,
}

macro_rules! continue_if_blocking {
    ($ex:expr) => {
        match $ex {
            Err(ref e) if io::ErrorKind::WouldBlock == e.kind() => continue,
            other => other,
        }
    };
}

pub fn serve() -> Result<(), Error> {
    const SERVER: Token = Token(0);

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

    let mut streams = HashMap::new();
    let mut next_token = 1;

    loop {
        poll.poll(&mut events, None)?;

        'events: for event in events.iter() {
            match event.token() {
                SERVER => {
                    let (stream, addr) = continue_if_blocking!(server.accept())?;
                    let token = Token(next_token);
                    poll.register(&stream, token, mio::Ready::all(), mio::PollOpt::edge())?;
                    streams.insert(
                        next_token,
                        Incoming {
                            tcp: stream,
                            buf: VecDeque::new(),
                        },
                    );
                    next_token += 1;
                }
                Token(token) => {
                    let mut eof = false;
                    {
                        let incom = streams
                            .get_mut(&token)
                            .ok_or_else(|| format_err!("wakeup for invalid token"))?;

                        if event.readiness().is_readable() {
                            match read_until_blocks(&incom.tcp)? {
                                Some(ref buf) if buf.is_empty() => (),
                                Some(found) => incom.buf.push_back(found),
                                None => {
                                    eof = true;
                                }
                            }
                        }

                        if event.readiness().is_writable() {
                            while !incom.buf.is_empty() {
                                drain_some_writeable(&mut incom.buf[0], &incom.tcp)?;
                                incom.buf.pop_front();
                            }
                        }
                    }
                    if eof {
                        streams.remove(&token);
                    }
                }
            }
        }
    }

    Ok(())
}

/// `None` on EOF
fn read_until_blocks<R: Read>(mut from: R) -> Result<Option<VecDeque<u8>>, Error> {
    let mut found = VecDeque::new();
    let mut buf = [0u8; 256];
    while let Some(bytes) = from.read(&mut buf).map_non_block()? {
        if 0 == bytes {
            return Ok(None);
        }
        found.extend(&buf[..bytes]);
    }
    Ok(Some(found))
}

fn drain_some_writeable<W: Write>(reading: &mut VecDeque<u8>, mut into: W) -> Result<(), Error> {
    while !reading.is_empty() {
        let written = {
            let (start, end) = reading.as_slices();
            let slice = if start.is_empty() { end } else { start };

            into.write(slice).map_non_block()?
        };

        match written {
            Some(consumed) => {
                assert_ne!(consumed, 0);
                let _ = reading.drain(..consumed);
            }
            None => break,
        };
    }
    Ok(())
}

trait MapNonBlock<T> {
    fn map_non_block(self) -> Result<Option<T>, io::Error>;
}

impl<T> MapNonBlock<T> for Result<T, io::Error> {
    fn map_non_block(self) -> Result<Option<T>, io::Error> {
        use std::io::ErrorKind::WouldBlock;

        match self {
            Ok(value) => Ok(Some(value)),
            Err(err) => {
                if let WouldBlock = err.kind() {
                    Ok(None)
                } else {
                    Err(err)
                }
            }
        }
    }
}
