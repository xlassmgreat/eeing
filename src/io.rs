pub mod plugin {
    use super::super::engine::EngSend;
    use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};
    use derive_more::{Deref, DerefMut};
    use serde::Deserialize;
    use std::io::{self, stdin, stdout, BufReader, BufWriter, Read, Stdin, Stdout, Write};

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
        Stop,
    }

    #[derive(Deref, DerefMut)]
    pub struct Receiver(BufReader<Stdin>);

    #[derive(Deref, DerefMut)]
    pub struct Sender(BufWriter<Stdout>);

    impl Receiver {
        pub fn new() -> Self {
            Self(BufReader::new(stdin()))
        }

        pub fn receive(&mut self) -> io::Result<Inp> {
            let size = self.read_u32::<NativeEndian>()?;
            let mut buf = vec![0; size as usize];
            self.read_exact(&mut buf)?;
            // The json is probably right, so something did go very wrong if it fails to unwrap.
            Ok(
                serde_json::from_slice(&buf)
                    .unwrap_or_else(|e| panic!("Failed decoding json: {e}")),
            )
        }
    }

    impl Sender {
        pub fn new() -> Self {
            Self(BufWriter::new(stdout()))
        }

        pub fn send(&mut self, b: EngSend) -> io::Result<()> {
            let buf = serde_json::to_string(&b)
                .unwrap_or_else(|e| panic!("Error deserializing JSON: {e}"));

            let l = buf.len() as u32;
            self.write_u32::<NativeEndian>(l)?;
            self.write_all(buf.as_bytes())?;
            self.flush()?;
            Ok(())
        }
    }
}
