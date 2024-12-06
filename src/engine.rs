use derive_more::{Deref, DerefMut};
use serde::{Deserialize, Serialize};
use shakmaty::{fen::Fen, san::San, CastlingMode, Chess, EnPassantMode, Position};
use std::{fmt::Display, io, process::Stdio};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    process::{Child, ChildStdin, ChildStdout, Command},
};

#[derive(Serialize)]
pub struct Bestmove {
    pub bestmove: String,
    pub pondermove: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
// Struct containing the "info" from engines
pub struct Info {
    raw: String,
    // More fields with some parsing expected in the future
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "returned")]
pub enum PlugSend {
    FromEngOut(FromEngOut),
    BoardError(BoardErrorSend),
    NoEngine()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
// Engine data to send to an Engine
pub enum FromEngOut {
    Bestmove(Bestmove),
    Info(Info),
    Unusual(String),
}

impl From<&str> for FromEngOut {
    fn from(value: &str) -> Self {
        let mut words = value.split(' ');
        if let Some("bestmove") = words.next() {
            match words.collect::<Vec<&str>>()[..] {
                [bm, "pondermove", pm] => Self::Bestmove(Bestmove {
                    bestmove: bm.to_owned(),
                    pondermove: pm.to_owned(),
                }),
                [bm, ..] => Self::Bestmove(Bestmove {
                    bestmove: bm.to_owned(),
                    pondermove: String::new(),
                }),
                _ => Self::Unusual(value.to_owned()),
            }
        } else {
            Self::Info(Info {
                raw: value.to_owned(),
            })
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum Limit {
    Movetime(u32),
    Depth(u32),
    Node(u32),
}

pub struct EngineInput(BufWriter<ChildStdin>);
pub struct EngineOutput(BufReader<ChildStdout>);

// Struct to give input to an engine
impl EngineInput {
    fn new(stdin: ChildStdin) -> Self {
        Self(BufWriter::new(stdin))
    }

    pub async fn send_single_line<T: Display>(&mut self, s: T) -> io::Result<()> {
        self.0.write(format!("{s}").as_bytes()).await?;
        self.0.flush().await?;
        Ok(())
    }

    // Sends a "go" command to the engine
    pub async fn go(&mut self, limit: Limit) -> io::Result<()> {
        match limit {
            Limit::Movetime(t) => self.send_single_line(format!("go movetime {t}\n")).await?,
            Limit::Depth(d) => self.send_single_line(format!("go depth {d}\n")).await?,
            Limit::Node(n) => self.send_single_line(format!("go depth {n}\n")).await?,
        };
        Ok(())
    }

    // Sends a "stop" command to the engine
    pub async fn stop(&mut self) -> io::Result<()> {
        self.send_single_line("stop\n").await
    }

    // Updates the position of the engine.
    pub async fn update_pos(&mut self, pos: &EnginePos) -> io::Result<()> {
        let fen = Fen::from_position((*pos).clone(), EnPassantMode::Legal);
        self.send_single_line(format!("position fen {fen}\n"))
            .await?;
        Ok(())
    }

    // Sets the given option to the given value
    pub async fn setoption<T: Display>(&mut self, name: &str, value: T) -> io::Result<()> {
        self.send_single_line(format!("setoption name {name} value {value}\n"))
            .await?;
        Ok(())
    }
}

// Struct to read output from the engine
impl EngineOutput {
    fn new(stdout: ChildStdout) -> Self {
        Self(BufReader::new(stdout))
    }

    async fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        self.0.read_line(buf).await
    }

    pub async fn input(&mut self) -> io::Result<FromEngOut> {
        let mut buf = String::new();
        self.read_line(&mut buf).await?;
        Ok(FromEngOut::from(buf.as_str()))
    }
}

// Struct to represent the engine
pub struct Engine {
    _child: Child,
    pub input: EngineInput,
    pub output: EngineOutput,
}

impl Engine {
    pub fn new(command: &str, args: Option<&Vec<String>>) -> io::Result<Self> {
        let mut child = Command::new(command);
        child.stdin(Stdio::piped()).stdout(Stdio::piped());
        if let Some(args) = args {
            child.args(args);
        }
        let mut child = child.spawn()?;
        let output = EngineOutput::new(child.stdout.take().unwrap());
        let input = EngineInput::new(child.stdin.take().unwrap());

        Ok(Engine {
            _child: child,
            input,
            output,
        })
    }
}

#[derive(Deref, DerefMut)]
// Struct to represent the position given to the engine
pub struct EnginePos(Chess);

#[derive(Serialize)]
pub enum PlayMoveError {
    InvalidSan,
    InvalidMove,
}

#[derive(Serialize)]
pub enum SetPosError {
    IncorrectFen,
    WrongPosition,
}

#[derive(Serialize)]
pub enum BoardError {
    PlayMoveError(PlayMoveError),
    SetPosError(SetPosError),
}

#[derive(Serialize)]
pub struct BoardErrorSend {
    error: BoardError,
    identification: MoveIdentification,
}

impl BoardErrorSend {
    pub fn new(error: BoardError, identification: MoveIdentification) -> Self {
        Self {
            error,
            identification,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MoveIdentification {
    #[serde(rename = "move")]
    PMove(String),
    Fen(String),
}

impl EnginePos {
    pub fn new() -> Self {
        EnginePos(Chess::new())
    }

    pub fn play_move(&mut self, mv: &str) -> Result<(), PlayMoveError> {
        let mv = mv.parse::<San>().map_err(|_| PlayMoveError::InvalidSan);

        let mv = mv.and_then(|x| x.to_move(&self.0).map_err(|_| PlayMoveError::InvalidMove));

        mv.map(|m| self.play_unchecked(&m))
    }

    pub fn set_pos(&mut self, fen: &str) -> Result<(), SetPosError> {
        let fen = fen.parse::<Fen>().map_err(|_| SetPosError::IncorrectFen);

        let pos = fen.and_then(|x| {
            x.into_position(CastlingMode::Standard)
                .map_err(|_| SetPosError::WrongPosition)
        });

        pos.map(|p| self.0 = p)
    }
}
