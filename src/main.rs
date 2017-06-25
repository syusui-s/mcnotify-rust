extern crate getopts;
extern crate mcnotify;

use std::{env, process, io};
use std::path::{Path, PathBuf};
use std::io::Write;
use getopts::Options;
use mcnotify::{minecraft, config_loader};
use minecraft::client::Client;
use minecraft::client::ServerAddr;

fn main() {
    let args: Vec<String> = env::args().collect();
    let program_name = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("c", "config", "use specified config file instead of the default", "FILE");
    opts.optflag("v", "version", "print version");
    opts.optflag("h", "help", "print this message");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            writeln!(&mut io::stderr(), "{}", f.to_string()).unwrap();
            process::exit(1);
        },
    };

    if matches.opt_present("h") {
        print_usage(&program_name, &opts);
        return;
    }

    if matches.opt_present("v") {
        print_version(&program_name);
        return;
    }

    // load config
    let config_loader = config_loader::ConfigLoader::new();
    let config = (if let Some(custom_conf) = matches.opt_str("config") {
             config_loader.read_config_from_path(Path::new(&custom_conf))
         } else {
             config_loader.read_config()
         })
        .expect("Couldn't load the configuration file...");

    let address = ServerAddr::new(config.address.hostname.as_str(), config.address.port);

    let mut cli = Client::connect(address).expect("Couldn't connect the server...");
    let res = cli.list().unwrap();

    println!("{}", res.get_status().get_version().get_name());
}

fn print_usage(program_name: &str, opts: &Options) {
    let pathbuf = PathBuf::from(program_name);
    let filename = pathbuf.file_name().unwrap().to_str().unwrap();
    let brief = format!(
r#"Usage: {} [OPTIONS]
Minecraft status notifier"#,
filename);
    print!("{}", opts.usage(&brief));
}

fn print_version(program_name: &str) {
    println!("{} {}", program_name, env!("CARGO_PKG_VERSION"));
}
