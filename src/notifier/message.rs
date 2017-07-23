#[derive(Clone)]
pub struct Message {
    body: String,
}

impl Message {
    pub fn new(body: &str) -> Self {
        Self { body: body.to_owned() }
    }

    pub fn body(&self) -> &String {
        &self.body
    }

    pub fn truncate(&self, len: usize) -> Self {
        let truncated_body : String = self.body.chars().take(len).collect();
        Self { body: truncated_body }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_truncate() {
        let message = Message::new("こんにちは、世界。").truncate(5);
        assert_eq!(message.body(), "こんにちは");
    }
}
