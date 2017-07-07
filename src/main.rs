#[macro_use]
extern crate serde_derive;
extern crate getopts;
extern crate strfmt;
extern crate egg_mode;

#[macro_use]
pub mod util;
pub mod config;
pub mod config_loader;
pub mod minecraft;
pub mod notifier;
pub mod status_checker;

use std::{env, process, io, thread};
use std::path::{Path, PathBuf};
use std::io::Write;
use getopts::Options;
use notifier::twitter_eggmode::TwitterEggMode;
use status_checker::Status::{Available, Unavailable};

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

    // load config
    let config_loader = config_loader::ConfigLoader::new();
    let config_result =
        match matches.opt_str("config") {
            Some(custom_conf) =>
                config_loader.read_config_from_path(Path::new(&custom_conf)),
            None =>
                config_loader.read_config(),
        };

    let config = config_result
        .expect("Couldn't load the configuration...");

    let strategy = TwitterEggMode::new(
        config.twitter.consumer_key.as_str(),
        config.twitter.consumer_secret.as_str(),
        config.twitter.access_key.as_str(),
        config.twitter.access_secret.as_str()
    );

    let formats = config.formats;
    let formatter = notifier::MessageFormatter {
        recover_msg: formats.recover_msg,
        down_msg:    formats.down_msg,
        join_fmt:    formats.join_fmt,
        leave_fmt:   formats.leave_fmt,
        players_fmt: formats.players_fmt,
        time_fmt:    formats.time_fmt,
    };

    let notifier = notifier::Notifier::new(strategy, formatter);

    let interval = std::time::Duration::from_secs(config.mcnotify.check_interval as u64);
    let mut status_checker = status_checker::StatusChecker::new(config.address.hostname.as_str(), config.address.port);
    let mut last_status = status_checker::Status::unavailable("On start");

    println!("Start checking.");

    loop {
        let status = status_checker.check_status();
        let message_opt = match status {
            Unavailable { ref reason } => {
                writeln!(&mut io::stderr(), "{}", reason).unwrap();
                match last_status {
                    Available { .. } => Some(notifier::Message::Down {}),
                    _                => None,
                }
            },
            Available { online_count, ref current_players, ref joined_players, ref left_players } => {
                if let Unavailable { .. } = last_status {
                    Some(notifier::Message::Recover {
                        online_count,
                        current_players: current_players,
                    })
                } else if ! joined_players.is_empty() || ! left_players.is_empty() {
                    Some(notifier::Message::PlayerChange {
                        online_count,
                        current_players: current_players,
                        joined_players: joined_players,
                        left_players: left_players,
                    })
                } else {
                    None
                }
            }
        };

        if let Some(msg) = message_opt {
            let notify_result = notifier.notify(&msg);

            if let Err(e) = notify_result {
                writeln!(&mut io::stderr(), "Failed to notify. {:?}", e).unwrap();
            }
        }

        last_status = status.clone();
        thread::sleep(interval);
    }
}
