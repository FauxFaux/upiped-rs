#[macro_use]
extern crate failure;
extern crate mio;

mod client;
mod server;

use failure::Error;

fn main() -> Result<(), Error> {
    client::serve()
}
