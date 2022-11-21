use serde::Deserialize;
use tokio::io::{stdin, AsyncReadExt, self, stdout, AsyncWriteExt, BufReader, Stdin, BufWriter, Stdout};
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

    pub async fn receive(&mut self) -> io::Result<Inp> {
        eprintln!("Before size read");
        let size = self.stdin.read_u32_le().await?;
        let mut buf = vec![0; size as usize];
        eprintln!("Before read exact: {size}");
        self.stdin.read_exact(&mut buf).await?;
        eprintln!("Send buf: {}", String::from_utf8(buf.clone()).unwrap());
        // The json is probably right, so something did go very wrong if it fails to unwrap.
        Ok(serde_json::from_slice(&buf).unwrap_or_else(|e| panic!("Failed decoding json: {e}")))
    }

    pub async fn send(&mut self, b: Bestmove) -> io::Result<()> {
        let buf = serde_json::to_string(&b).unwrap_or_else(|e| panic!("Error deserializing JSON: {e}"));
        eprintln!("Send size: {}", buf.len());
        eprintln!("Send info: {buf}");
        self.stdout.write_u32_le(buf.len() as u32).await?;
        self.stdout.write_all(buf.as_bytes()).await?;
        self.stdout.flush().await?;
        Ok(())
    }
}
