extern crate strfmt;
extern crate chrono;

use std::convert;
use std::collections::HashMap;
use self::strfmt::Format;
use notifier::Message;
use notifier::Message::*;

#[derive(Debug)]
pub enum Error {
    FormatError(String),
}

impl convert::From<strfmt::FmtError> for Error {
    fn from(from: strfmt::FmtError) -> Error {
        Error::FormatError(from.to_string())
    }
}

pub struct MessageFormat {
    pub recover_msg: String,
    pub down_msg: String,
    pub join_fmt: String,
    pub leave_fmt: String,
    pub players_fmt: String,
    pub time_fmt: String,
}

impl MessageFormat {
    pub fn format(&self, message: &Message) -> Result<String, Error> {
        let mut buffer = String::with_capacity(560); // 140 chars * 4 bytes

        self.format_time(&mut buffer);

        // join / leave
        match message {
            &PlayerChange { joined_players, left_players, .. } => {
                self.format_join(&mut buffer, joined_players)?;
                self.format_leave(&mut buffer, left_players)?;
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

    fn format_time(&self, buffer: &mut String) {
        if ! self.time_fmt.is_empty() {
            let current_time = chrono::Local::now();
            let formatted_time = current_time.format(self.time_fmt.as_str());
            buffer.push_str(formatted_time.to_string().as_str());
            buffer.push('\n');
        }
    }

    fn format_join(&self, buffer: &mut String, players: &Vec<String>) -> Result<(), Error> {
        if ! players.is_empty() {
            Self::build_players(buffer, self.join_fmt.as_str(), players)?;
            buffer.push('\n');
        }
        Ok(())
    }

    fn format_leave(&self, buffer: &mut String, players: &Vec<String>) -> Result<(), Error> {
        if ! players.is_empty() {
            Self::build_players(buffer, self.leave_fmt.as_str(), players)?;
            buffer.push('\n');
        }
        Ok(())
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

    fn setup_format() -> MessageFormat {
        let format = MessageFormat {
            recover_msg: "recovered".to_owned(),
            down_msg:    "down".to_owned(),
            join_fmt:    "{players}".to_owned(),
            leave_fmt:   "{players}".to_owned(),
            players_fmt: "{players} {count}".to_owned(),
            time_fmt:    "[]".to_owned(),
        };

        format
    }

    #[test]
    fn message_format_player_change() {
        let format = setup_format();

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

        assert_eq!(format.format(&recover).unwrap().as_str(), "[]\nA, B\nD\nA, B, C 3");
    }

    #[test]
    fn message_format_recover() {
        let format = setup_format();

        let message = Message::Recover {
            online_count: 3,
            current_players: &vec![
                "A".to_owned(),
                "B".to_owned(),
                "C".to_owned(),
            ],
        };

        assert_eq!(format.format(&message).unwrap().as_str(), "[]\nrecovered\nA, B, C 3");
    }

    #[test]
    fn message_format_down() {
        let format = setup_format();

        let message = Message::Down;
        assert_eq!(format.format(&message).unwrap().as_str(), "[]\ndown");
    }
}
