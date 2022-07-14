use std::{
    io::ErrorKind,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
};
use tracing::{debug, info, instrument, warn};

pub enum Reader {
    Simple(SimpleReader),
    Echoing(EchoingReader),
}

impl Reader {
    #[instrument(name = "Reader::run", skip(self))]
    pub async fn run(&mut self) -> anyhow::Result<()> {
        info!("Reader::run");
        match self {
            Reader::Simple(inner) => inner.run().await,
            Reader::Echoing(inner) => inner.run().await,
        }
    }
}

pub struct EchoingReader {
    reader: SimpleReader,
    write_half: OwnedWriteHalf,
    /// Counter for chunks sent
    sent: Arc<AtomicU64>,
}

impl EchoingReader {
    pub fn new(
        stream: TcpStream,
        chunk_size: usize,
        sent: Arc<AtomicU64>,
        received: Arc<AtomicU64>,
    ) -> Self {
        let (read_half, write_half) = stream.into_split();

        let reader = SimpleReader::new(read_half, chunk_size, received);

        debug!("EchoingReader::new");

        Self {
            reader,
            write_half,
            sent,
        }
    }

    pub async fn read_chunk(&mut self) -> std::io::Result<()> {
        // Read
        self.reader.read_chunk().await?;

        // Echo response
        self.write_half.write_all(&self.reader.buf).await?;
        self.sent.fetch_add(1, Ordering::SeqCst);

        Ok(())
    }

    #[instrument(name = "EchoingReader::run", skip(self))]
    pub async fn run(&mut self) -> anyhow::Result<()> {
        info!("start EchoingReader::run");

        loop {
            let res = self.read_chunk().await;
            if let ReadChunkAction::Exit(res) = handle_read_chunk_result(res) {
                info!("end EchoingReader::run ({:?})", res);
                return res;
            }
        }
    }
}

/// Simple (non-echoing) reader
pub struct SimpleReader {
    read_half: OwnedReadHalf,
    pub buf: Vec<u8>,
    /// Counter for chunks received
    received: Arc<AtomicU64>,
}

impl SimpleReader {
    pub fn new(read_half: OwnedReadHalf, chunk_size: usize, received: Arc<AtomicU64>) -> Self {
        let mut buf = Vec::new();
        buf.resize(chunk_size, 0);

        info!("SimpleReader::new");

        Self {
            read_half,
            buf,
            received,
        }
    }

    pub async fn read_chunk(&mut self) -> std::io::Result<()> {
        debug!("read_chunk");
        match self.read_half.read_exact(&mut self.buf).await {
            Ok(_nbytes) => {
                // Increment received chunks counter
                self.received.fetch_add(1, Ordering::SeqCst);

                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    #[instrument(name = "SimpleReader::Run", skip(self))]
    pub async fn run(&mut self) -> anyhow::Result<()> {
        info!("start SimpleReader::run");

        loop {
            let res = self.read_chunk().await;
            if let ReadChunkAction::Exit(res) = handle_read_chunk_result(res) {
                info!("end SimpleReader::run ({:?})", res);
                return res;
            } else {
                debug!("Continue");
            }
        }
    }
}

enum ReadChunkAction {
    Continue,
    Exit(anyhow::Result<()>),
}

fn handle_read_chunk_result(res: std::io::Result<()>) -> ReadChunkAction {
    use ErrorKind::{ConnectionReset, UnexpectedEof};
    if let Err(err) = res {
        return match err.kind() {
            // EOF _is_ expected here.
            UnexpectedEof | ConnectionReset => ReadChunkAction::Exit(Ok(())),
            other => {
                warn!("UNEXPECTED ERROR KIND: {:?} ({})", other, other);
                ReadChunkAction::Exit(Err(err.into()))
            }
        };
    }

    ReadChunkAction::Continue
}
