use derive_more::{Deref, DerefMut};
use serde::{Deserialize, Serialize};
use shakmaty::{fen::Fen, san::San, CastlingMode, Chess, EnPassantMode, Position};
use std::{
    fmt::Display,
    io::{self, BufRead, BufReader, BufWriter, Write},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
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
// Engine data to send to an Engine
pub enum EngSend {
    Bestmove(Bestmove),
    Info(Info),
}

impl From<&str> for EngSend {
    fn from(value: &str) -> Self {
        let mut words = value.split(' ');
        if let Some("bestmove") = words.next() {
            match words.collect::<Vec<&str>>()[..] {
                [bm, "pondermove", pm] => EngSend::Bestmove(Bestmove {
                    bestmove: bm.to_owned(),
                    pondermove: pm.to_owned(),
                }),
                [bm, ..] => EngSend::Bestmove(Bestmove {
                    bestmove: bm.to_owned(),
                    pondermove: String::new(),
                }),
                _ => panic!("Engine output wrong"),
            }
        } else {
            EngSend::Info(Info {
                raw: value.to_owned(),
            })
        }
    }
}

#[derive(Deserialize, Clone, Copy)]
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

    pub fn send_single_line<T: Display>(&mut self, s: T) -> io::Result<()> {
        self.0.write(format!("{s}").as_bytes())?;
        self.0.flush()?;
        Ok(())
    }

    // Sends a "go" command to the engine
    pub fn go(&mut self, limit: Limit) -> io::Result<()> {
        match limit {
            Limit::Movetime(t) => self.send_single_line(format!("go movetime {t}\n"))?,
            Limit::Depth(d) => self.send_single_line(format!("go depth {d}\n"))?,
            Limit::Node(n) => self.send_single_line(format!("go depth {n}\n"))?,
        };
        Ok(())
    }

    // Sends a "stop" command to the engine
    pub fn stop(&mut self) -> io::Result<()> {
        self.send_single_line("stop\n")
    }

    // Updates the position of the engine.
    pub fn update_pos(&mut self, pos: &EnginePos) -> io::Result<()> {
        let fen = Fen::from_position((*pos).clone(), EnPassantMode::Legal);
        self.send_single_line(format!("position fen {fen}\n"))?;
        Ok(())
    }

    // Sets the given option to the given value
    pub fn setoption<T: Display>(&mut self, name: &str, value: T) -> io::Result<()> {
        self.send_single_line(format!("setoption name {name} value {value}\n"))?;
        Ok(())
    }
}

// Struct to read output from the engine
impl EngineOutput {
    fn new(stdout: ChildStdout) -> Self {
        Self(BufReader::new(stdout))
    }

    fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        self.0.read_line(buf)
    }

    pub fn input(&mut self) -> io::Result<EngSend> {
        let mut buf = String::new();
        self.read_line(&mut buf)?;
        Ok(EngSend::from(buf.as_str()))
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

impl EnginePos {
    pub fn new() -> Self {
        EnginePos(Chess::new())
    }

    pub fn play_move(&mut self, mv: &str) {
        let mv = mv
            .parse::<San>()
            .expect("Invalid san")
            .to_move(&self.0)
            .expect("Invalid move");
        self.play_unchecked(&mv);
    }

    pub fn set_pos(&mut self, fen: &str) {
        let fen = fen.parse::<Fen>().expect("Incorrect fen");
        self.0 = fen
            .into_position(CastlingMode::Standard)
            .expect("Wrong position");
    }
}
