use std::io;

use crate::{
    engine::{EngineInput, EngineOutput, EnginePos, Limit},
    io::plugin::{self, Inp, PosInp},
};

// Listens for information from plugin; gives input to engine
pub fn plugin_listener(
    precv: &mut plugin::Receiver,
    einp: &mut EngineInput,
    limit: Limit,
) -> io::Result<()> {
    let mut pos = EnginePos::new();
    loop {
        let inp = match precv.receive() {
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
                einp.update_pos(&pos)?;
                einp.go(limit)?;
            }

            Inp::Stop => einp.stop()?,
        }
    }

    Ok(())
}

// Listens for information from engine; outputs to plugin
pub fn engine_listener(psend: &mut plugin::Sender, eout: &mut EngineOutput) -> io::Result<()> {
    loop {
        let esend = eout.input()?;
        psend.send(esend)?;
    }
}
