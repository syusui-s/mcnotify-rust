extern crate toml;
#[cfg(unix)]
extern crate xdg_basedir;

use std::convert;
#[cfg(unix)]
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum Error {
    XDGError(xdg_basedir::Error),
    IoError(io::Error),
    TomlDeserializeError(toml::de::Error),
    ConfigNotFound,
    CondigDirsIsEmpty,
}

impl_convert_for_error!(xdg_basedir::Error, Error::XDGError);
impl_convert_for_error!(io::Error, Error::IoError);
impl_convert_for_error!(toml::de::Error, Error::TomlDeserializeError);

#[derive(Deserialize)]
pub struct Config {
    pub mcnotify: McNotify,
    pub address: Address,
    pub formats: Formats,
    pub twitter: Option<TwitterConfig>,
    pub ifttt: Option<IFTTTConfig>,
    pub command: Option<CommandConfig>,
    pub stdout: Option<StdoutConfig>,
}

#[derive(Deserialize)]
pub struct McNotify {
    pub check_interval: u16,
}

#[derive(Deserialize)]
pub struct Address {
    pub hostname: String,
    pub port: u16,
}

#[derive(Deserialize)]
pub struct Formats {
    /// A notification message sent when the server recovered.
    pub recover_msg: String,

    /// A notification message sent when the server down.
    pub down_msg: String,

    /// A notification message sent when some player joined the server.
    pub join_fmt: String,

    /// A notification message sent when some player left the server.
    pub leave_fmt: String,

    pub players_fmt: String,

    pub time_fmt: String,
}

#[derive(Deserialize)]
pub struct TwitterConfig {
    pub consumer_key: String,
    pub consumer_secret: String,
    pub access_key: String,
    pub access_secret: String,
}

#[derive(Deserialize)]
pub struct IFTTTConfig {
    pub endpoint_url: String,
    pub truncate: Option<usize>,
}

#[derive(Deserialize)]
pub struct CommandConfig {
    pub command: String,
    pub args: Vec<String>,
    pub pipe: bool,
}

#[derive(Deserialize)]
pub struct StdoutConfig {}

impl Config {
    pub fn read_path(path: &Path) -> Result<Config, Error> {
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open(path)?;

        let mut string = String::with_capacity(256);
        file.read_to_string(&mut string)?;

        Ok(toml::from_str(&string)?)
    }

    pub fn read_default() -> Result<Config, Error> {
        // XDG_CONFIG_HOME
        let mut pathbuf = xdg_basedir::get_config_home()?;
        let path = Self::build_config_path(&mut pathbuf);
        if path.exists() {
            return Self::read_path(path);
        }

        // XDG_CONFIG_DIRS
        let dirs = xdg_basedir::get_config_dirs();

        if dirs.is_empty() {
            return Err(Error::CondigDirsIsEmpty);
        }

        for mut dir in dirs.into_iter() {
            let path = Self::build_config_path(&mut dir);
            if path.exists() {
                return Self::read_path(path);
            }
        }

        Err(Error::ConfigNotFound)
    }

    fn build_config_path(pathbuf: &mut PathBuf) -> &Path {
        const CONFIG_DIR: &str = "mcnotify";
        const CONFIG_PATH: &str = "config.toml";

        pathbuf.push(CONFIG_DIR);
        pathbuf.push(CONFIG_PATH);

        pathbuf.as_path()
    }
}
