use crate::engine::Engine;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::Limit;
use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read},
};
use toml::Value;

#[derive(Deserialize, Serialize)]
struct Eng {
    command: String,
    args: Option<Vec<String>>,
    config: Option<HashMap<String, Value>>,
}

#[derive(Deserialize, Serialize)]
pub struct ConfigOptions {
    engine: Eng,
    #[serde(flatten)]
    limit: Limit,
}

impl Default for ConfigOptions {
    fn default() -> Self {
        ConfigOptions {
            engine: Eng {
                command: "stockfish".to_string(),
                args: None,
                config: None,
            },

            limit: Limit::Movetime(1000),
        }
    }
}

#[derive(Error, Debug)]
enum GetOptionError {
    #[error("Cannot read file config.json")]
    Read,
    #[error("Cannot parse file config.json")]
    Parse,
    #[error("File is empty")]
    Empty,
}

pub struct Configurer {
    pub file: Option<File>,
    pub write: bool,
    pub opt: Option<ConfigOptions>,
    pub report: Option<String>,
}

impl Configurer {
    pub fn new() -> Self {
        let file = ConfigOptions::get_file();

        let (write, opt, file, report) = if let Ok(mut file) = file {
            let (write, opt, report) = match ConfigOptions::get_options(&mut file) {
                Ok(opt) => (false, Some(opt), None),
                Err(e) => {
                    let write = match e {
                        GetOptionError::Empty => true,
                        _ => false,
                    };

                    (write, None, Some(e.to_string()))
                }
            };

            (write, opt, Some(file), report)
        } else {
            (false, None, None, Some("Cannot open file".to_string()))
        };

        Configurer {
            file,
            write,
            opt,
            report,
        }
    }
}

impl ConfigOptions {
    fn get_file() -> io::Result<File> {
        File::options()
            .create(true)
            .write(true)
            .read(true)
            .open("config.json")
    }

    fn get_options(file: &mut File) -> Result<ConfigOptions, GetOptionError> {
        let mut buf = String::new();

        if buf.len() == 0 {
            Err(GetOptionError::Empty)?
        }

        file.read_to_string(&mut buf)
            .map_err(|_| GetOptionError::Read)?;

        let parsed =
            serde_json::from_str::<ConfigOptions>(&buf).map_err(|_| GetOptionError::Parse)?;

        Ok(parsed)
    }

    // Sets up an engine with the config options.
    pub async fn setup_engine(&self) -> io::Result<Engine> {
        let mut engine = Engine::new(&self.engine.command, self.engine.args.as_ref())?;

        if let Some(config) = &self.engine.config {
            for (key, value) in config {
                let key = key.replace("_", " ");
                engine.input.setoption(&key, value).await?;
            }
        }

        Ok(engine)
    }

    pub fn limit(&self) -> Limit {
        self.limit
    }
}
