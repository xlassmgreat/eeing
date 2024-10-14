// Module to handle communications with the browser plugins
//

use serde::Serialize;

#[derive(Serialize)]
pub struct ErrorSend<ERR: Serialize, IDENT: Serialize> {
    error: ERR,
    identification: IDENT,
}

impl<ERR: Serialize, IDENT: Serialize> ErrorSend<ERR, IDENT> {
    pub fn new(error: ERR, identification: IDENT) -> Self {
        Self {
            error,
            identification,
        }
    }
}

pub mod plugin {
    use derive_more::{Deref, DerefMut};
    use serde::{de::DeserializeOwned, Deserialize, Serialize};
    use std::{io, u32};
    use tokio::io::{
        stdin, stdout, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter, Stdin, Stdout,
    };

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    // Input about the position from the plugin.
    pub enum PosInp {
        Moves(Vec<String>),
        Fen(String),
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    #[serde(tag = "cmd")]
    // Input from the plugin
    pub enum Inp {
        Go,
        Pos(PosInp),
        Stop,
    }

    #[derive(Deref, DerefMut)]
    // Receiver of information from plugins
    pub struct Receiver(BufReader<Stdin>);

    #[derive(Deref, DerefMut)]
    // Sender of information to plugins
    pub struct Sender(BufWriter<Stdout>);

    impl Receiver {
        pub fn new() -> Self {
            Self(BufReader::new(stdin()))
        }

        pub async fn receive<T: DeserializeOwned>(&mut self) -> io::Result<Option<T>> {
            let size = self.read_u32().await?.to_be();

            let mut buf = vec![0; size as usize];
            self.read_exact(&mut buf).await?;
            // The json is probably right, so something did go very wrong if it fails to unwrap.
            Ok(serde_json::from_slice(&buf)
                .inspect_err(|e| log::error!("Failed decoding json: {e}"))
                .ok())
        }
    }

    impl Sender {
        pub fn new() -> Self {
            Self(BufWriter::new(stdout()))
        }

        pub async fn send<T: Serialize>(&mut self, b: T) -> io::Result<()> {
            let buf = serde_json::to_string(&b)
                .inspect_err(|e| log::error!("Error deserializing JSON: {e}"))
                .ok();

            if let Some(buf) = buf {
                let l = (buf.len() as u32).to_be();

                self.write_u32(l).await?;
                self.write_all(buf.as_bytes()).await?;
                self.flush().await?;
            }

            Ok(())
        }
    }
}
