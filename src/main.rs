#[macro_use]
extern crate serde_derive;
extern crate toml;

mod minecraft;

use minecraft::client::Client;

fn main() {
    let mut cli = Client::connect("127.0.0.1:25565").expect("Couldn't connect the server...");
    let res = cli.list().unwrap();

    println!("{}", res.get_status().get_version().get_name());
}
