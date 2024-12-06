use crate::{
    config_parser::ConfigOptions,
    engine::{
        BoardError, BoardErrorSend, Engine, EngineInput, EngineOutput, EnginePos, Limit,
        MoveIdentification, PlugSend,
    },
    io::plugin::{Cmd, Inp, PosInp, Receiver, Sender},
};

use std::io;
use tokio::sync::mpsc;

pub async fn engine_creator(
    mut configs: mpsc::Receiver<ConfigOptions>,
    eng_inp: mpsc::Sender<EngineInput>,
    eng_out: mpsc::Sender<EngineOutput>,
) {
    while let Ok(config) = configs.try_recv() {
        let engine = config.setup_engine().await.unwrap();
        eng_inp.send(engine.input);
        eng_out.send(engine.output);
    }
}

// Listens for information from plugin; gives input to engine
pub async fn plugin_listener(eng_inps: mpsc::Receiver<EngineInput>) -> io::Result<()> {
    let mut precv = Receiver::new();
    let mut psend = Sender::new();
    let mut pos = EnginePos::new();
    let mut eng_inp = None;

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

        eng_inp = eng_inps.try_recv().ok();

        let error: Result<(), BoardErrorSend> = match inp {
            Inp::Cmd(cmd) => match cmd {
                Cmd::Pos(p) => match p {
                    PosInp::Fen(fen) => pos.set_pos(&fen).map_err(|e| {
                        BoardErrorSend::new(
                            BoardError::SetPosError(e),
                            MoveIdentification::Fen(fen.to_owned()),
                        )
                    }),
                    PosInp::Moves(mvs) => {
                        mvs.into_iter().try_for_each(|m| match pos.play_move(&m) {
                            Ok(_) => Ok(()),
                            Err(e) => Err(BoardErrorSend::new(
                                BoardError::PlayMoveError(e),
                                MoveIdentification::PMove(m.to_owned()),
                            )),
                        })
                    }
                },

                Cmd::Go => {
                    if let Some(einp) = eng_inp {
                    einp.update_pos(&pos).await?;
                    einp.go(limit).await?;
                    Ok(())
                    } else {

                    }
                }

                Cmd::Stop => {
                    einp.stop().await?;
                    Ok(())
                }
            },
        };

        if let Err(error) = error {
            psend.send(PlugSend::BoardError(error)).await?;
        }
    }

    Ok(())
}

// Listens for information from engine; outputs to plugin
pub async fn engine_listener(eout: &mut EngineOutput) -> io::Result<()> {
    let mut psend = Sender::new();
    loop {
        let esend = eout.input().await?;
        psend.send(PlugSend::FromEngOut(esend)).await?;
    }
}
