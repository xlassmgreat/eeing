use std::{process::Stdio, ops::Deref};
use serde::Serialize;
use shakmaty::{Chess, fen::Fen, Position, CastlingMode, san::San, EnPassantMode};

use tokio::{process::{Child, ChildStdout, ChildStdin, Command}, io::{BufReader, BufWriter, self, AsyncBufReadExt, AsyncWriteExt}};

#[derive(Serialize)]
pub struct Bestmove {
    pub bestmove: String,
    pub pondermove: String,
}

pub enum Limit {
    Movetime(u32),
    Depth(u32),
}

pub struct Engine {
    child: Child,
    reader: BufReader<ChildStdout>,
    writer: BufWriter<ChildStdin>,
}

impl Engine {
    pub fn new(command: &str) -> io::Result<Self> {
        let mut child = Command::new(command).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn()?;
        let reader = BufReader::new(child.stdout.take().unwrap());
        let writer = BufWriter::new(child.stdin.take().unwrap());

        Ok(Engine {child, reader, writer})
    }

    pub async fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        self.reader.read_line(buf).await
    }

    pub async fn send_single_line(&mut self, s: String) -> io::Result<()> {
        self.writer.write(s.as_bytes()).await?;
        self.writer.flush().await?;
        Ok(())
    }

    pub async fn go(&mut self, limit: Limit) -> io::Result<Bestmove> {
        match limit {
            Limit::Movetime(t) => self.send_single_line(format!("go movetime {t}\n")).await?,
            Limit::Depth(d) => self.send_single_line(format!("go depth {d}\n")).await?,
        };

        let mut buf = String::new();
        loop {
            self.read_line(&mut buf).await?;
            let mut words = buf[0..(buf.len()-1)].split(' ');
            if let Some("bestmove") = words.next() {
                match words.collect::<Vec<&str>>()[..] {
                    [bestmove, "ponder", pondermove, ..] => break Ok(Bestmove {bestmove: bestmove.to_owned(), pondermove: pondermove.to_owned()}),
                    [bestmove, ..] => break Ok(Bestmove {bestmove: bestmove.to_owned(), pondermove: String::new()}),
                    _ => {},
                }
            }
            buf.clear();
        }
    }

    pub async fn update_pos(&mut self, pos: &EnginePos) -> io::Result<()> {
        let fen = Fen::from_position((*pos).clone(), EnPassantMode::Legal);
        eprintln!("Fen {fen}");
        self.send_single_line(format!("position fen {fen}\n")).await?;
        Ok(())
    }
}

pub struct EnginePos {
    pos: Chess
}

impl EnginePos {
    pub fn new() -> Self {
        EnginePos {pos: Chess::new()}
    }

    pub fn play_move(&mut self, mv: &str) {
        let mv = mv.parse::<San>().expect("Invalid san").to_move(&self.pos).expect("Invalid move");
        self.pos.play_unchecked(&mv);
    }

    pub fn set_pos(&mut self, fen: &str) {
        let fen = fen.parse::<Fen>().expect("Incorrect fen");
        self.pos = fen.into_position(CastlingMode::Standard).expect("Wrong position");
    }
}

impl Deref for EnginePos {
    type Target = Chess;
    fn deref(&self) -> &Self::Target {
        &self.pos
    }
}
