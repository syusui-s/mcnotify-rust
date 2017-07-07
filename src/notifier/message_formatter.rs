extern crate strfmt;
extern crate chrono;

use std::convert;
use std::collections::HashMap;
use strfmt::Format;
use super::message::Message;
use super::message::Message::*;

#[derive(Debug)]
pub enum Error {
    MessageFormatError(String),
}

impl convert::From<strfmt::FmtError> for Error {
    fn from(err: strfmt::FmtError) -> Error {
        let message = err.to_string();
        Error::MessageFormatError(message)
    }
}

pub struct MessageFormatter {
    pub recover_msg: String,
    pub down_msg: String,
    pub join_fmt: String,
    pub leave_fmt: String,
    pub players_fmt: String,
    pub time_fmt: String,
}

impl MessageFormatter {
    pub fn format(&self, message: &Message) -> Result<String, Error> {
        let mut buffer = String::with_capacity(560); // 140 chars * 4 bytes

        if ! self.time_fmt.is_empty() {
            let current_time = chrono::Local::now();
            let formatted_time = current_time.format(self.time_fmt.as_str());
            buffer.push_str(formatted_time.to_string().as_str());
            buffer.push('\n');
        }

        // join / leave
        match message {
            &PlayerChange { joined_players, left_players, .. } => {
                if ! joined_players.is_empty() {
                    self.format_join(&mut buffer, joined_players)?;
                    buffer.push('\n');
                }

                if ! left_players.is_empty() {
                    self.format_leave(&mut buffer, left_players)?;
                    buffer.push('\n');
                }
            }
            &Recover { .. } => {
                buffer.push_str(self.recover_msg.as_str());
                buffer.push('\n');
            },
            &Down { } => {
                buffer.push_str(self.down_msg.as_str());
            },
        }

        // current players
        match message {
            &PlayerChange { online_count, current_players, .. } |
            &Recover      { online_count, current_players, .. } => {
                self.format_current_players(&mut buffer, online_count, current_players)?;
            },
            _ => { },
        };

        Ok(buffer)
    }

    fn format_join(&self, buffer: &mut String, players: &Vec<String>) -> Result<(), Error> {
        Self::build_players(buffer, self.join_fmt.as_str(), players)
    }

    fn format_leave(&self, buffer: &mut String, players: &Vec<String>) -> Result<(), Error> {
        Self::build_players(buffer, self.leave_fmt.as_str(), players)
    }

    fn format_current_players(&self, buffer: &mut String, online_count: u32, players: &Vec<String>) -> Result<(), Error> {
        let mut hashmap = HashMap::new();
        hashmap.insert("count".to_owned(), online_count.to_string());
        Self::build_players_hashmap(buffer, &mut hashmap, self.players_fmt.as_str(), players)
    }

    fn build_players(buffer: &mut String, fmt: &str, players: &Vec<String>) -> Result<(), Error> {
        let mut hashmap = HashMap::new();
        Self::build_players_hashmap(buffer, &mut hashmap, fmt, players)
    }

    fn build_players_hashmap(buffer: &mut String,
                             hashmap: &mut HashMap<String, String>,
                             fmt: &str,
                             players: &Vec<String>) -> Result<(), Error>
    {
        hashmap.insert("players".to_owned(), players.join(", "));
        buffer.push_str(fmt.format(&hashmap)?.as_str());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_formatter() -> MessageFormatter {
        let formatter = MessageFormatter {
            recover_msg: "recovered".to_owned(),
            down_msg:    "down".to_owned(),
            join_fmt:    "{players}".to_owned(),
            leave_fmt:   "{players}".to_owned(),
            players_fmt: "{players} {count}".to_owned(),
            time_fmt:    "[]".to_owned(),
        };

        formatter
    }

    #[test]
    fn message_formatter_player_change() {
        let formatter = setup_formatter();

        let recover = Message::PlayerChange {
            online_count: 3,
            current_players: &vec![
                "A".to_owned(),
                "B".to_owned(),
                "C".to_owned(),
            ],
            joined_players: &vec![
                "A".to_owned(),
                "B".to_owned(),
            ],
            left_players: &vec![
                "D".to_owned(),
            ],
        };

        assert_eq!(formatter.format(&recover).unwrap().as_str(), "[]\nA, B\nD\nA, B, C 3");
    }

    #[test]
    fn message_formatter_recover() {
        let formatter = setup_formatter();

        let message = Message::Recover {
            online_count: 3,
            current_players: &vec![
                "A".to_owned(),
                "B".to_owned(),
                "C".to_owned(),
            ],
        };

        assert_eq!(formatter.format(&message).unwrap().as_str(), "[]\nrecovered\nA, B, C 3");
    }

    #[test]
    fn message_formatter_down() {
        let formatter = setup_formatter();

        let message = Message::Down;
        assert_eq!(formatter.format(&message).unwrap().as_str(), "[]\ndown");
    }
}
