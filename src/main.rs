mod config_parser;
mod engine;
mod io;
mod listener;
use std::io::Write;

use anyhow::Result;

use config_parser::ConfigOptions;
use engine::{Engine, Limit};

async fn run(mut engine: Engine, limit: Limit) -> Result<()> {
    let plug = listener::plugin_listener(&mut engine.input, limit);
    let eng = listener::engine_listener(&mut engine.output);

    tokio::try_join!(plug, eng)?;

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    simple_logging::log_to_stderr(log::LevelFilter::Off);

    let configurer = config_parser::Configurer::new();
    let config = configurer.opt.unwrap_or(ConfigOptions::default());
    let engine = config.setup_engine().await?;

    configurer
        .report
        .map(|r| log::warn!("Something wrong with configuration: {r}"));

    if let Some(mut file) = configurer.file {
        if configurer.write {
            let text = serde_json::to_string(&ConfigOptions::default()).unwrap();
            file.write_all(text.as_bytes())
                .unwrap_or_else(|e| log::warn!("Unable to write to configuration file: {e}"));
        }
    }

    run(engine, config.limit()).await?;
    Ok(())
}
