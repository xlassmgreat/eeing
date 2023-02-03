mod config_parser;
mod engine;
mod io;
mod listener;
use std::error::Error;

use engine::{Engine, Limit};
use io::plugin::{Receiver, Sender};

fn run(mut engine: Engine, limit: Limit) -> Result<(), Box<dyn Error>> {
    let mut receiver = Receiver::new();
    let mut sender = Sender::new();

    std::thread::spawn(move || listener::plugin_listener(&mut receiver, &mut engine.input, limit));
    listener::engine_listener(&mut sender, &mut engine.output).unwrap();
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let config = config_parser::ConfigOptions::get_options();
    let engine = config.setup_engine()?;
    run(engine, config.limit())?;
    Ok(())
}
