use super::Message;

#[derive(Debug)]
pub enum Error {
    FailedToPostMessage(String),
}

pub trait NotifierStrategy {
    /// post a given message to the service
    fn notify(&self, message: &Message) -> Result<(), Error>;
}
