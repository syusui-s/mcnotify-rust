#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate getopts;

#[macro_use]
pub mod util;
pub mod minecraft;
pub mod notifier;
pub mod models;
pub mod status_checker;
pub mod config;
pub mod application;

use std::{env, process, io};
use std::path::{Path, PathBuf};
use std::io::Write;
use getopts::Options;
use config::Config;
use application::Application;

fn print_usage(program_name: &str, opts: &Options) {
    let pathbuf = PathBuf::from(program_name);
    let filename = pathbuf.file_name().unwrap().to_str().unwrap();
    let brief = format!("Usage: {} [OPTIONS]\nMinecraft status notifier", filename);
    print!("{}", opts.usage(&brief));
}

fn print_version(program_name: &str) {
    println!("{} {}",
             program_name,
             env!("CARGO_PKG_VERSION"));
}

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let program_name = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("c", "config", "use specified config file instead of the default", "FILE");
    opts.optflag("v", "version", "print version");
    opts.optflag("h", "help", "print this message");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            writeln!(&mut io::stderr(), "Option parse error: {}", f.to_string()).unwrap();
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

    let config =
        match matches.opt_str("config") {
            Some(custom_conf) =>
                Config::read_path(Path::new(&custom_conf)),
            None =>
                Config::read_default(),
        }.expect("Couldn't load the configuration...");

    let app = Application::new(config);
    app.run();
}
