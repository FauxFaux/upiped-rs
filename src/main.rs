extern crate byteorder;
extern crate cast;
extern crate chacha20_poly1305_aead;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
extern crate mio;
#[macro_use]
extern crate more_asserts;
extern crate pretty_env_logger;
extern crate rand;
extern crate x25519_dalek;

mod client;
mod proto;
mod server;

use failure::Error;

fn main() -> Result<(), Error> {
    pretty_env_logger::formatted_builder()?
        .filter_level(log::LevelFilter::Info)
        .init();
    client::serve()
}
