use crate::engine::Engine;
use serde::Deserialize;

use super::Limit;
use std::{collections::HashMap, io};
use toml::Value;

#[derive(Deserialize)]
struct Eng {
    command: String,
    args: Option<Vec<String>>,
    config: Option<HashMap<String, Value>>,
}

#[derive(Deserialize)]
pub struct ConfigOptions {
    engine: Eng,
    #[serde(flatten)]
    limit: Limit,
}

impl ConfigOptions {
    pub fn get_options() -> ConfigOptions {
        let buf = std::fs::read_to_string("config.toml")
            .unwrap_or_else(|e| panic!("Could not open config.toml: {e}"));
        toml::from_str(&buf).expect("Unable to parse config.toml.")
    }

    pub fn setup_engine(&self) -> io::Result<Engine> {
        let mut engine = Engine::new(&self.engine.command, self.engine.args.as_ref())?;

        if let Some(config) = &self.engine.config {
            for (key, value) in config {
                let key = key.replace("_", " ");
                engine.input.setoption(&key, value)?;
            }
        }

        Ok(engine)
    }

    pub fn limit(&self) -> Limit {
        self.limit
    }
}
