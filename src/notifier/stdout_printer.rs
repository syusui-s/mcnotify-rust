use super::notifier_base::Error;
use super::Message;
use super::NotifierStrategy;

pub struct StdoutPrinter {}

impl StdoutPrinter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for StdoutPrinter {
    fn default() -> Self {
        Self::new()
    }
}

impl NotifierStrategy for StdoutPrinter {
    fn notify(&self, message: &Message) -> Result<(), Error> {
        println!("{}", message.body());

        Ok(())
    }
}
