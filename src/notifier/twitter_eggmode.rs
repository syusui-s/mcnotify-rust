extern crate egg_mode;

use super::notifier::Error;
use super::NotifierStrategy;
use super::Message;

pub struct TwitterEggMode<'a> {
    token: egg_mode::Token<'a>,
}

impl<'a> TwitterEggMode<'a> {
    pub fn new(consumer_key: &'a str, consumer_secret: &'a str, access_key: &'a str, access_secret: &'a str) -> Self {
        // create twitter client
        let consumer = egg_mode::KeyPair::new(consumer_key, consumer_secret);
        let access   = egg_mode::KeyPair::new(access_key, access_secret);
        let token    = egg_mode::Token::Access { consumer, access };

        Self { token }
    }
}

impl<'a> NotifierStrategy for TwitterEggMode<'a> {
    fn notify(&self, message: &Message) -> Result<(), Error> {
        let truncated = message.truncate(140);
        egg_mode::tweet::DraftTweet::new(truncated.body())
            .send(&self.token)
            .map_err(|_| Error::FailedToPostMessage)?;
        Ok(())
    }
}
