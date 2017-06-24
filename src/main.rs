extern crate clap;
extern crate mcnotify;

use std::path::Path;
use clap::{App, Arg};
use mcnotify::{minecraft, config_loader};
use minecraft::client::Client;
use minecraft::client::ServerAddr;

fn main() {
    let matches = App::new("mcnotify_rust")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Minecraft status notifier")
        .arg(Arg::with_name("config")
             .short("c")
             .long("config")
             .value_name("FILE")
             .help("Sets a custom config file")
             .takes_value(true))
        .get_matches();

    // load config
    let config_loader = config_loader::ConfigLoader::new();
    let config = (
        if let Some(custom_conf) = matches.value_of("config") {
            config_loader.read_config_from_path(Path::new(custom_conf))
        } else {
            config_loader.read_config()
        }).expect("Couldn't load the configuration file...");

    let address = ServerAddr::new(config.address.hostname.as_str(), config.address.port);

    let mut cli = Client::connect(address).expect("Couldn't connect the server...");
    let res = cli.list().unwrap();

    println!("{}", res.get_status().get_version().get_name());
}
