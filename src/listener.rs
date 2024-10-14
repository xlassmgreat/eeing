use crate::{
    engine::{EngineInput, EngineOutput, EnginePos, ErrorFromEngine, Limit},
    io::{
        plugin::{Inp, PosInp, Receiver, Sender},
        ErrorSend,
    },
};

use std::io;

// Listens for information from plugin; gives input to engine
pub async fn plugin_listener(einp: &mut EngineInput, limit: Limit) -> io::Result<()> {
    let mut precv = Receiver::new();
    let mut psend = Sender::new();
    let mut pos = EnginePos::new();

    loop {
        let inp = match precv.receive::<Inp>().await {
            Ok(v) => match v {
                Some(v) => v,
                None => break,
            },

            Err(e) => {
                if e.kind() != io::ErrorKind::UnexpectedEof {
                    log::error!("Error reading from plugin: {e}");
                }

                break;
            }
        };

        let error: Result<(), ErrorFromEngine> = match inp {
            Inp::Pos(p) => match p {
                PosInp::Fen(fen) => pos
                    .set_pos(&fen)
                    .map_err(|e| ErrorFromEngine::SetPosError(ErrorSend::new(e, fen.to_owned()))),
                PosInp::Moves(mvs) => mvs.into_iter().try_for_each(|m| match pos.play_move(&m) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(ErrorFromEngine::PlayMoveError(ErrorSend::new(
                        e,
                        m.to_owned(),
                    ))),
                }),
            },

            Inp::Go => {
                einp.update_pos(&pos).await?;
                einp.go(limit).await?;
                Ok(())
            }

            Inp::Stop => {
                einp.stop().await?;
                Ok(())
            }
        };

        if let Err(error) = error {
            psend.send(error).await?;
        }
    }

    Ok(())
}

// Listens for information from engine; outputs to plugin
pub async fn engine_listener(eout: &mut EngineOutput) -> io::Result<()> {
    let mut psend = Sender::new();
    loop {
        let esend = eout.input().await?;
        psend.send(esend).await?;
    }
}
