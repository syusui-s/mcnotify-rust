use crate::config::Config;
use crate::notifier::command_executor::CommandExecutor;
use crate::notifier::ifttt_webhook::IFTTTWebhook;
use crate::notifier::stdout_printer::StdoutPrinter;
use crate::notifier::twitter_eggmode::TwitterEggMode;
use crate::notifier::{Error as NotifierError, Message, NotifierStrategy};
use crate::status_checker::{FormatError, Status, StatusChecker, StatusDifference, StatusFormats};
use std::{thread, time};

pub struct Application {
    config: Config,
}

impl Application {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn run(&self) {
        let mut notifier_strategies: Vec<Box<dyn NotifierStrategy>> = Vec::new();

        if let Some(conf) = &self.config.twitter {
            let strategy = TwitterEggMode::new(
                &conf.consumer_key,
                &conf.consumer_secret,
                &conf.access_key,
                &conf.access_secret,
            );
            notifier_strategies.push(Box::new(strategy));
        }

        if let Some(conf) = &self.config.ifttt {
            let strategy = IFTTTWebhook::new(&conf.endpoint_url, conf.truncate);
            notifier_strategies.push(Box::new(strategy));
        }

        if let Some(conf) = &self.config.command {
            let strategy = CommandExecutor::new(&conf.command, conf.args.clone(), conf.pipe);
            notifier_strategies.push(Box::new(strategy));
        }

        if self.config.stdout.is_some() {
            let strategy = StdoutPrinter::default();
            notifier_strategies.push(Box::new(strategy));
        }

        if notifier_strategies.is_empty() {
            error!("No strategies available!");
            return;
        }

        let config_formats = &self.config.formats;
        let status_formats = StatusFormats {
            recover_msg: config_formats.recover_msg.clone(),
            down_msg: config_formats.down_msg.clone(),
            join_fmt: config_formats.join_fmt.clone(),
            leave_fmt: config_formats.leave_fmt.clone(),
            players_fmt: config_formats.players_fmt.clone(),
            time_fmt: config_formats.time_fmt.clone(),
        };

        let interval = time::Duration::from_secs(self.config.mcnotify.check_interval as u64);
        let mut status_checker =
            StatusChecker::new(&self.config.address.hostname, self.config.address.port);

        info!("Start checking.");

        loop {
            Self::check_and_notify(&mut status_checker, &status_formats, &notifier_strategies);
            thread::sleep(interval);
        }
    }

    fn check_and_notify(
        status_checker: &mut StatusChecker,
        status_formats: &StatusFormats,
        notifier_strategies: &[Box<dyn NotifierStrategy>],
    ) {
        let status_difference = status_checker.get_status_difference();

        match status_difference {
            StatusDifference::Down { ref reason } => {
                error!("Server is down: {}", reason);
                return;
            }
            StatusDifference::None {
                latest_status: Status::Unavailable { ref reason },
            } => {
                error!("Server is unavailable: {}", reason);
                return;
            }
            _ => {}
        }

        let message_result = status_formats.format(&status_difference);
        let message_opt = match message_result {
            Ok(message) => message,
            Err(FormatError::FormatError(reason)) => {
                error!("Error occurred while formatting a status: {}", reason);
                return;
            }
        };

        let message = match message_opt {
            Some(message) => message,
            None => return,
        };

        for notifier in notifier_strategies.iter() {
            match notifier.notify(&Message::new(&message)) {
                Ok(()) => {}
                Err(NotifierError::FailedToPostMessage(ref msg)) => {
                    error!("Failed to notify. {:?}", msg)
                }
            }
        }
    }
}
