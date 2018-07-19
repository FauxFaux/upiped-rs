#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
extern crate mio;
extern crate pretty_env_logger;

mod client;
mod server;

use failure::Error;

fn main() -> Result<(), Error> {
    pretty_env_logger::formatted_builder()?
        .filter_level(log::LevelFilter::Info)
        .init();
    client::serve()
}
