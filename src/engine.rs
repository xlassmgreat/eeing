use std::{
    process::{Stdio, Child, ChildStdout, ChildStdin, Command},
    io::{self, BufReader, BufWriter, BufRead, Write},
    ops::Deref,
    fmt::Display,
};
use serde::{Serialize, Deserialize};
use shakmaty::{Chess, fen::Fen, Position, CastlingMode, san::San, EnPassantMode};
use rand::{Rng, rngs::ThreadRng};

#[derive(Serialize)]
pub struct Bestmove {
    pub bestmove: String,
    pub pondermove: String,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum Limit {
    Movetime(u32),
    Depth(u32),
    Node(u32),
}

pub trait MovePlayer {
    fn bestmove(&mut self, limit: Limit) -> Bestmove;
    fn pos(&mut self, pos: &EnginePos);
}

pub struct Engine {
    _child: Child,
    reader: BufReader<ChildStdout>,
    writer: BufWriter<ChildStdin>,
}

impl Engine {
    pub fn new(command: &str, args: Option<&Vec<String>>) -> io::Result<Self> {
        let mut child = Command::new(command);
        child.stdin(Stdio::piped()).stdout(Stdio::piped());
        if let Some(args) = args {child.args(args);}
        let mut child = child.spawn()?;
        let reader = BufReader::new(child.stdout.take().unwrap());
        let writer = BufWriter::new(child.stdin.take().unwrap());

        Ok(Engine {_child: child, reader, writer})
    }

    fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        self.reader.read_line(buf)
    }

    fn send_single_line(&mut self, s: String) -> io::Result<()> {
        self.writer.write(s.as_bytes())?;
        self.writer.flush()?;
        Ok(())
    }

    fn go(&mut self, limit: Limit) -> io::Result<Bestmove> {
        match limit {
            Limit::Movetime(t) => self.send_single_line(format!("go movetime {t}\n"))?,
            Limit::Depth(d) => self.send_single_line(format!("go depth {d}\n"))?,
            Limit::Node(n) => self.send_single_line(format!("go depth {n}\n"))?,
        };

        let mut buf = String::new();
        loop {
            self.read_line(&mut buf)?;
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

    fn update_pos(&mut self, pos: &EnginePos) -> io::Result<()> {
        let fen = Fen::from_position((*pos).clone(), EnPassantMode::Legal);
        self.send_single_line(format!("position fen {fen}\n"))?;
        Ok(())
    }

    
    pub fn setoption<T: Display>(&mut self, name: &str, value: T) -> io::Result<()> {
        self.send_single_line(format!("setoption name {name} value {value}\n"))?;
        Ok(())
    }
}

impl MovePlayer for Engine {
    fn bestmove(&mut self, limit: Limit) -> Bestmove {
        self.go(limit).unwrap()
    }

    fn pos(&mut self, pos: &EnginePos) {
        self.update_pos(pos).unwrap();
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


pub struct RandomMover{
    pos: Chess,
    rng: ThreadRng,
}

impl RandomMover {
    pub fn new() -> Self {
        RandomMover {pos: Chess::new(), rng: rand::thread_rng()}
    }
}


impl MovePlayer for RandomMover {
    fn bestmove(&mut self, _limit: Limit) -> Bestmove {
        let mut gen = |pos: &Chess| {
            let mut moves = pos.legal_moves();
            let r = self.rng.gen_range(0..moves.len());
            moves.pop_at(r)
        };

        let mut new_pos = self.pos.clone();
        let bm = gen(&self.pos).expect("No more legal moves");
        new_pos.play_unchecked(&bm);
        let bm = bm.to_uci(CastlingMode::Standard).to_string();
        let pm = gen(&new_pos).and_then(|p| Some(p.to_uci(CastlingMode::Standard).to_string())).unwrap_or(String::new());

        Bestmove {bestmove: bm, pondermove: pm}
    }

    fn pos(&mut self, pos: &EnginePos) {
        self.pos = (*pos).clone();
    }
}
