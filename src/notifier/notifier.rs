use std::convert;
use super::Message;
use super::{MessageFormat, MessageFormatError};

#[derive(Debug)]
pub enum Error {
    MessageFormatError(MessageFormatError),
    FailedToPostMessage,
}

impl_convert_for_error!(MessageFormatError, Error::MessageFormatError);

pub trait NotifierStrategy {
    /// post a given message to the service
    fn post_message(&self, message: &str) -> Result<(), Error>;
}

pub struct Notifier<S> where S: NotifierStrategy {
    strategy: S,
    format: MessageFormat,
}

impl<S> Notifier<S> where S: NotifierStrategy {
    pub fn new(strategy: S, format: MessageFormat) -> Self {
        Self { strategy, format }
    }

    pub fn notify(&self, message: &Message) -> Result<(), Error> {
        self.strategy.post_message(self.format.format(message)?.as_str())?;
        Ok(())
    }
}
