use std::{io, time, thread};
use std::io::Write;
use notifier::{Notifier, Message, MessageFormat};
use notifier::twitter_eggmode::TwitterEggMode;
use status_checker::{StatusChecker, Status};
use status_checker::Status::{Available, Unavailable};
use config::Config;

pub struct Application;

impl Application {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&self, config: Config) {
        let strategy = TwitterEggMode::new(
            config.twitter.consumer_key.as_str(),
            config.twitter.consumer_secret.as_str(),
            config.twitter.access_key.as_str(),
            config.twitter.access_secret.as_str()
            );

        let formats = config.formats;
        let formatter = MessageFormat {
            recover_msg: formats.recover_msg,
            down_msg:    formats.down_msg,
            join_fmt:    formats.join_fmt,
            leave_fmt:   formats.leave_fmt,
            players_fmt: formats.players_fmt,
            time_fmt:    formats.time_fmt,
        };

        let notifier = Notifier::new(strategy, formatter);

        let interval = time::Duration::from_secs(config.mcnotify.check_interval as u64);
        let mut status_checker = StatusChecker::new(config.address.hostname.as_str(), config.address.port);
        let mut last_status = Status::unavailable("On start");

        println!("Start checking.");

        loop {
            let status = status_checker.check_status();
            let message_opt = match (&last_status, &status) {
                ( &Unavailable { .. },
                  &Unavailable { .. }
                ) => {
                    None
                },
                ( &Unavailable { .. },
                  &Available { online_count, ref current_players, .. }
                ) => {
                    Some(Message::Recover { online_count, current_players })
                },
                ( &Available { .. },
                  &Unavailable { ref reason }
                ) => {
                    writeln!(&mut io::stderr(), "{}", reason).unwrap();
                    Some(Message::Down {})
                },
                ( &Available { .. },
                  &Available {
                      online_count,
                      ref current_players,
                      ref joined_players,
                      ref left_players
                  }
                ) if ! joined_players.is_empty() || ! left_players.is_empty() => {
                    Some(Message::PlayerChange {
                        online_count,
                        current_players,
                        joined_players,
                        left_players
                    })
                },
                _ => None,
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
}
