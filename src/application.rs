use std::{io, time, thread};
use std::io::Write;
use notifier::{Message, NotifierStrategy};
use notifier::twitter_eggmode::TwitterEggMode;
use status_checker::{StatusChecker, StatusDifference, StatusFormats, FormatError};
use config::Config;

pub struct Application {
    config: Config,
}

impl Application {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn run(&self) {
        let config_twitter = &self.config.twitter;
        let notifier_strategy = TwitterEggMode::new(
            &config_twitter.consumer_key,
            &config_twitter.consumer_secret,
            &config_twitter.access_key,
            &config_twitter.access_secret
            );

        let config_formats = &self.config.formats;
        let status_formats = StatusFormats {
            recover_msg: config_formats.recover_msg.clone(),
            down_msg:    config_formats.down_msg.clone(),
            join_fmt:    config_formats.join_fmt.clone(),
            leave_fmt:   config_formats.leave_fmt.clone(),
            players_fmt: config_formats.players_fmt.clone(),
            time_fmt:    config_formats.time_fmt.clone(),
        };

        let interval = time::Duration::from_secs(self.config.mcnotify.check_interval as u64);
        let mut status_checker = StatusChecker::new(&self.config.address.hostname, self.config.address.port);

        println!("Start checking.");

        loop {
            Self::check_and_notify(&mut status_checker, &status_formats, &notifier_strategy);
            thread::sleep(interval);
        }
    }

    fn check_and_notify<S>(
        status_checker: &mut StatusChecker,
        status_formats: &StatusFormats,
        notifier_strategy: &S)
        where S: NotifierStrategy
    {
        let status_difference = status_checker.get_status_difference();

        match &status_difference {
            &StatusDifference::Down { ref reason } => {
                writeln!(&mut io::stderr(), "Error occurred while checking a status: {}", reason).unwrap();
                return;
            },
            _ => {}
        }

        let message_result = status_formats.format(&status_difference);
        let message_opt = match message_result {
            Ok(message) => message,
            Err(FormatError::FormatError(reason)) => {
                writeln!(&mut io::stderr(), "Error occurred while formatting a status: {}", reason).unwrap();
                return;
            }
        };

        let message = match message_opt {
            Some(message) => message,
            None => return,
        };

        match notifier_strategy.notify(&Message::new(&message)) {
            Ok(()) => {},
            Err(e) => {
                writeln!(&mut io::stderr(), "Failed to notify. {:?}", e).unwrap();
                return;
            }
        }
    }
}
