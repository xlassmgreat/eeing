mod engine;
mod io;
use io::Plugin;
use engine::EnginePos;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut engine = engine::Engine::new("stockfish").unwrap();
    engine.send_single_line(format!("setoption name Debug Log File value /home/shambhav/code/log.sf\n")).await.unwrap();
    let mut plugin = Plugin::new();
    let mut pos = EnginePos::new();

    loop {
        use io::{Inp, PosInp};
        let inp = plugin.receive().await.unwrap();
        match inp {
            Inp::Pos(p) => match p {
                PosInp::Fen(fen) => pos.set_pos(&fen),
                PosInp::Moves(mvs) => {
                    eprintln!("Moves");
                    for m in mvs {
                        pos.play_move(&m);
                    }
                }
            },
            Inp::Go => {
                engine.update_pos(&pos).await.unwrap();
                let bm = engine.go(engine::Limit::Movetime(1000)).await.unwrap();
                plugin.send(bm).await.unwrap();
            }
        }
    }
}
