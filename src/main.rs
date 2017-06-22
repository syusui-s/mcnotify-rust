#[macro_use]
extern crate serde_derive;
extern crate toml;

mod minecraft;

use minecraft::client::Client;

fn main() {
    let mut cli = Client::connect("127.0.0.1:25565").unwrap();
    cli.handshake().unwrap();
    cli.list().unwrap();
}
