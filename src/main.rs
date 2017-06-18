#[macro_use]
extern crate serde_derive;

mod minecraft;

use minecraft::client::Client;

fn main() {
    let mut cli = Client::connect("localhost:25565").unwrap();
    cli.handshake();
}
