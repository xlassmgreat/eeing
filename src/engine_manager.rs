use std::collections::HashMap;

use tokio::{sync::mpsc, task::JoinHandle};

use crate::{config_parser::ConfigOptions, engine::Engine};

#[derive(Hash)]
struct EngineProfile {
    engine: Option<Engine>,
    config: ConfigOptions,
}

pub struct Engines {
    engines: HashMap<u32, EngineProfile>,
    counter: u32,
    runner: JoinHandle<()>,
    conf_send: mpsc::Sender<ConfigOptions>,
}

impl Engines {
    pub async fn new() -> Self {
        let (conf_send, conf_recv): (mpsc::Sender<ConfigOptions>, _) = mpsc::channel(1);
        let res = tokio::spawn(async {
            while let Ok(config) = conf_recv.try_recv() {
                let engine = config.setup_engine();
            }
        });

        Self {
            engines: HashMap::new(),
            counter: 0,
            runner: res,
            conf_send,
        }
    }

    pub fn register(&mut self, config: ConfigOptions) -> u32 {
        self.counter += 1;
        self.counter
    }
}
