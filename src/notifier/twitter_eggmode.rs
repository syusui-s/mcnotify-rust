extern crate egg_mode;

use super::notifier::Error;
use super::notifier::NotifierStrategy;

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
    fn post_message(&self, message: &str) -> Result<(), Error> {
        let trunc : String = message.chars().take(140).collect();
        egg_mode::tweet::DraftTweet::new(trunc.as_str())
            .send(&self.token)
            .map_err(|_| Error::FailedToPostMessage)?;
        Ok(())
    }
}
