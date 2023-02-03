use std::io;

use crate::{
    engine::{EngineInput, EngineOutput, EnginePos, Limit},
    io::plugin::{self, Inp, PosInp},
};

pub fn plugin_listener(
    precv: &mut plugin::Receiver,
    einp: &mut EngineInput,
    limit: Limit,
) -> io::Result<()> {
    loop {
        let inp = match precv.receive() {
            Ok(v) => v,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => Err(e).unwrap(),
        };
        let mut pos = EnginePos::new();
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

pub fn engine_listener(psend: &mut plugin::Sender, eout: &mut EngineOutput) -> io::Result<()> {
    loop {
        let esend = eout.input()?;
        psend.send(esend)?;
    }
}
