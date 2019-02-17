extern crate egg_mode;
extern crate tokio;

use super::notifier::Error;
use super::NotifierStrategy;
use super::Message;
use self::tokio::runtime::current_thread::block_on_all;
use std::string::ToString;

pub struct TwitterEggMode {
    token: egg_mode::Token,
}

impl TwitterEggMode {
    pub fn new(consumer_key: &str, consumer_secret: &str, access_key: &str, access_secret: &str) -> Self {
        // create twitter client
        let consumer = egg_mode::KeyPair::new(consumer_key.to_owned(), consumer_secret.to_owned());
        let access   = egg_mode::KeyPair::new(access_key.to_owned(), access_secret.to_owned());
        let token    = egg_mode::Token::Access { consumer, access };

        Self { token }
    }
}

impl NotifierStrategy for TwitterEggMode {
    fn notify(&self, message: &Message) -> Result<(), Error> {
        let truncated = message.truncate(140);
        block_on_all(
            egg_mode::tweet::DraftTweet::new(truncated.body())
            .send(&self.token)
        ).map_err(|e| Error::FailedToPostMessage(e.to_string()))?;

        Ok(())
    }
}
