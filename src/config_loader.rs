#[cfg(unix)]
extern crate xdg_basedir;
extern crate toml;

#[cfg(unix)]
use std::io;
use std::convert;
use std::path::{Path, PathBuf};
use super::config::Config;

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

const CONFIG_DIR  : &str = "mcnotify";
const CONFIG_PATH : &str = "config.toml";

pub struct ConfigLoader;

impl ConfigLoader {
    pub fn new() -> Self {
        Self { }
    }

    fn build_config_path(pathbuf: &mut PathBuf) -> &Path {
        pathbuf.push(CONFIG_DIR);
        pathbuf.push(CONFIG_PATH);

        pathbuf.as_path()
    }
    pub fn read_config_from_path(&self, path: &Path) -> Result<Config, Error> {
        use std::io::Read;
        use std::fs::File;

        let mut file = File::open(path)?;

        let mut string = String::with_capacity(256);
        file.read_to_string(&mut string)?;

        Ok(toml::from_str(string.as_str())?)
    }

    pub fn read_config(&self) -> Result<Config, Error> {
        // XDG_CONFIG_HOME
        let mut pathbuf = xdg_basedir::get_config_home()?;
        let path = Self::build_config_path(&mut pathbuf);
        if path.exists() {
            return self.read_config_from_path(&path);
        }

        // XDG_CONFIG_DIRS
        let dirs = xdg_basedir::get_config_dirs();

        if dirs.is_empty() {
            return Err(Error::CondigDirsIsEmpty);
        }

        for mut dir in dirs.into_iter() {
            let path = Self::build_config_path(&mut dir);
            if path.exists() {
                return self.read_config_from_path(path);
            }
        }

        return Err(Error::ConfigNotFound);
    }

}
