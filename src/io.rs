use serde::Deserialize;
use std::io::{self, stdin, stdout, Stdin, Stdout, BufReader, BufWriter, Read, Write};
use byteorder::{ReadBytesExt, WriteBytesExt, NativeEndian};
use super::engine::Bestmove;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PosInp {
    Moves(Vec<String>),
    Fen(String),
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "cmd")]
pub enum Inp {
    Go,
    Pos(PosInp),
}

pub struct Plugin {
    stdin: BufReader<Stdin>,
    stdout: BufWriter<Stdout>,
}

impl Plugin {
    pub fn new() -> Self {
        Plugin {stdin: BufReader::new(stdin()), stdout: BufWriter::new(stdout())}
    }

    pub fn receive(&mut self) -> io::Result<Inp> {
        let size = self.stdin.read_u32::<NativeEndian>()?;
        let mut buf = vec![0; size as usize];
        self.stdin.read_exact(&mut buf)?;
        // The json is probably right, so something did go very wrong if it fails to unwrap.
        Ok(serde_json::from_slice(&buf).unwrap_or_else(|e| panic!("Failed decoding json: {e}")))
    }

    pub fn send(&mut self, b: Bestmove) -> io::Result<()> {
        let buf = serde_json::to_string(&b).unwrap_or_else(|e| panic!("Error deserializing JSON: {e}"));

        let l = buf.len() as u32;
        self.stdout.write_u32::<NativeEndian>(l)?;
        self.stdout.write_all(buf.as_bytes())?;
        self.stdout.flush()?;
        Ok(())
    }
}
