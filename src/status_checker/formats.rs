extern crate chrono;
extern crate strfmt;

use std::convert;
use std::collections::HashMap;
use self::strfmt::Format;
use status_checker::StatusDifference;
use models::Players;

#[derive(Debug)]
pub enum Error {
    FormatError(String),
}

impl convert::From<strfmt::FmtError> for Error {
    fn from(from: strfmt::FmtError) -> Error {
        Error::FormatError(from.to_string())
    }
}

pub struct StatusFormats {
    pub recover_msg: String,
    pub down_msg: String,
    pub join_fmt: String,
    pub leave_fmt: String,
    pub players_fmt: String,
    pub time_fmt: String,
}

impl StatusFormats {
    pub fn format(&self, status_difference: &StatusDifference) -> Result<Option<String>, Error> {
        use status_checker::StatusDifference::*;

        let mut buffer = String::with_capacity(560); // 140 chars * 4 bytes

        self.format_time(&mut buffer);

        // join / leave
        match status_difference {
            &PlayerChange { ref joined_players, ref left_players, .. } => {
                self.format_join(&mut buffer, joined_players)?;
                self.format_leave(&mut buffer, left_players)?;
            }
            &Recover { .. } => {
                buffer.push_str(&self.recover_msg);
                buffer.push('\n');
            },
            &Down { .. } => {
                buffer.push_str(&self.down_msg);
            },
            &None { } => {
                return Ok(Option::None);
                // do nothing
            },
        }

        // current players
        match status_difference {
            &PlayerChange { online_count, ref current_players, .. } |
            &Recover      { online_count, ref current_players, .. } => {
                self.format_current_players(&mut buffer, online_count, current_players)?;
            },
            _ => { },
        };

        Ok(Some(buffer))
    }

    fn format_time(&self, buffer: &mut String) {
        if ! self.time_fmt.is_empty() {
            let current_time = chrono::Local::now();
            let formatted_time = current_time.format(&self.time_fmt);
            buffer.push_str(&formatted_time.to_string());
            buffer.push('\n');
        }
    }

    fn format_join(&self, buffer: &mut String, players: &Players) -> Result<(), Error> {
        if ! players.is_empty() {
            Self::build_players(buffer, &self.join_fmt, players)?;
            buffer.push('\n');
        }
        Ok(())
    }

    fn format_leave(&self, buffer: &mut String, players: &Players) -> Result<(), Error> {
        if ! players.is_empty() {
            Self::build_players(buffer, &self.leave_fmt, players)?;
            buffer.push('\n');
        }
        Ok(())
    }

    fn format_current_players(&self, buffer: &mut String, online_count: u32, players: &Players) -> Result<(), Error> {
        let mut hashmap = HashMap::new();
        hashmap.insert("count".to_owned(), online_count.to_string());
        Self::build_players_hashmap(buffer, &mut hashmap, &self.players_fmt, players)
    }

    fn build_players(buffer: &mut String, fmt: &str, players: &Players) -> Result<(), Error> {
        let mut hashmap = HashMap::new();
        Self::build_players_hashmap(buffer, &mut hashmap, fmt, players)
    }

    fn build_players_hashmap(buffer: &mut String,
                             hashmap: &mut HashMap<String, String>,
                             fmt: &str,
                             players: &Players) -> Result<(), Error>
    {
        hashmap.insert("players".to_owned(), format!("{}", players));
        buffer.push_str(&fmt.format(&hashmap)?);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_format() -> StatusFormats {
        let format = StatusFormats {
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
    fn status_format_player_change() {
        let format = setup_format();

        let recover = StatusDifference::PlayerChange {
            online_count: 3,
            current_players: Players::from(vec![
                Player::new("idA", "A"),
                Player::new("idB", "B"),
                Player::new("idC", "C"),
            ]),
            joined_players: Players::from(vec![
                Player::new("idA", "A"),
                Player::new("idB", "B"),
            ]),
            left_players: Players::from(vec![
                Player::new("idD", "D"),
            ]),
        };

        assert_eq!(&format.format(&recover).unwrap(), "[]\nA, B\nD\nA, B, C 3");
    }

    #[test]
    fn status_format_recover() {
        let format = setup_format();

        let message = StatusDifference::Recover {
            online_count: 3,
            current_players: Players::from(vec![
                Player::new("idA", "A"),
                Player::new("idB", "B"),
                Player::new("idC", "C"),
            ]),
        };

        assert_eq!(&format.format(&message).unwrap(), "[]\nrecovered\nA, B, C 3");
    }

    #[test]
    fn status_format_down() {
        let format = setup_format();

        let message = StatusDifference::Down { reason: String::from("hoge") };
        assert_eq!(&format.format(&message).unwrap(), "[]\ndown");
    }
}
