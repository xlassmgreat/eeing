mod engine;
mod io;
use std::{error::Error, collections::HashMap};

use io::Plugin;
use engine::{EnginePos, Engine, MovePlayer, RandomMover, Limit};
use serde::Deserialize;
use toml::Value;

 fn run<T: MovePlayer>(mut mover: T, limit: Limit) -> Result<(), Box<dyn Error>> {
    let mut plugin = Plugin::new();
    let mut pos = EnginePos::new();

    loop {
        use io::{Inp, PosInp};
        let inp = match plugin.receive() {
            Ok(v) => v,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => Err(e).unwrap(),
        };

        match inp {
            Inp::Pos(p) => match p {
                PosInp::Fen(fen) => pos.set_pos(&fen),
                PosInp::Moves(mvs) => {
                    for m in mvs {
                        pos.play_move(&m);
                    }
                }
            },
            Inp::Go => {
                mover.pos(&pos);
                let bm = mover.bestmove(limit);
                plugin.send(bm)?;
            }
        }
    }

    Ok(())
}

#[derive(Deserialize)]
struct Eng {
    command: String,
    args: Option<Vec<String>>,
    config: Option<HashMap<String, Value>>
}


#[derive(Deserialize)]
struct ConfigOptions {
    random_moves: bool,
    engine: Eng,
    limit: Limit,
}

fn main() -> Result<(), Box<dyn Error>> {
    let buf = std::fs::read_to_string("config.toml").unwrap_or_else(|e| panic!("Could not open config.toml: {e}"));
    let config: ConfigOptions = toml::from_str(&buf).expect("Unable to parse config.toml.");

    if config.random_moves {
        let mover = RandomMover::new();
        run(mover, config.limit)?;
    } else {
        let mut engine = Engine::new(&config.engine.command, config.engine.args.as_ref())?;
        
        if let Some(config) = &config.engine.config {
            for (key, value) in config {
                engine.setoption(key, value)?;
            }
        }
        run(engine, config.limit)?;
    }

    Ok(())
}
