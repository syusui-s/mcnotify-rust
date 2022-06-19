extern crate futures;
extern crate reqwest;
extern crate tokio;

use futures::executor::block_on;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;

use super::notifier_base::Error;
use super::Message;
use super::NotifierStrategy;

pub struct IFTTTWebhook {
    endpoint: String,
    truncate: Option<usize>,
}

impl IFTTTWebhook {
    pub fn new(endpoint: &str, truncate: Option<usize>) -> Self {
        Self {
            endpoint: endpoint.to_owned(),
            truncate,
        }
    }
}

impl NotifierStrategy for IFTTTWebhook {
    fn notify(&self, message: &Message) -> Result<(), Error> {
        let client = reqwest::Client::new();
        let body = match self.truncate {
            Some(len) => message.truncate(len).body().clone(),
            None => message.body().clone(),
        };

        let mut params = HashMap::new();
        params.insert("value1", body);

        let future = client.post(&self.endpoint).form(&params).send();

        match block_on(timeout(Duration::from_secs(5), future)) {
            Ok(res) => {
                res.and_then(|r| r.error_for_status())
                    .map_err(|e| Error::FailedToPostMessage(format!("{}", e)))?;

                Ok(())
            }
            Err(e) => Err(Error::FailedToPostMessage(format!("Timeout: {}", e))),
        }
    }
}
