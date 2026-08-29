use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::Bytes;
use futures_util::{Stream, StreamExt};
use tokio::sync::{mpsc, oneshot};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tracing::error;

use crate::error::Result;
use crate::{RNodeInterface, RadioConfig, Stats};

#[derive(Debug)]
pub struct RNodeInterfaceAsync {
    tx: mpsc::UnboundedSender<Command>,
    rx: UnboundedReceiverStream<Bytes>,
}

enum Command {
    SendData {
        data: Bytes,
        ready_tx: oneshot::Sender<Result<()>>,
    },
    Verify {
        ready_tx: oneshot::Sender<bool>,
    },
    Stats {
        ready_tx: oneshot::Sender<Stats>,
    },
    Bitrate {
        ready_tx: oneshot::Sender<f32>,
    },
}

impl RNodeInterfaceAsync {
    pub fn new(port: &str, config: RadioConfig) -> Result<Self> {
        let (tx, mut inner_rx) = mpsc::unbounded_channel::<Command>();
        let (inner_tx, rx) = mpsc::unbounded_channel::<Bytes>();

        let (iface, iface_rx) = RNodeInterface::new(port, config)?;

        tokio::task::spawn_blocking(move || {
            loop {
                let Some(command) = inner_rx.blocking_recv() else {
                    // Close thread if tx dropped.
                    break;
                };

                match command {
                    Command::SendData { data, ready_tx } => {
                        let result = iface.send(data);
                        let _ = ready_tx.send(result);
                    }
                    Command::Verify { ready_tx } => {
                        let result = iface.verify();
                        let _ = ready_tx.send(result);
                    }
                    Command::Stats { ready_tx } => {
                        let result = iface.stats();
                        let _ = ready_tx.send(result);
                    }
                    Command::Bitrate { ready_tx } => {
                        let result = iface.bitrate();
                        let _ = ready_tx.send(result);
                    }
                }
            }
        });

        tokio::task::spawn_blocking(move || {
            loop {
                match iface_rx.recv() {
                    Ok(data) => {
                        if inner_tx.send(data).is_err() {
                            // Close thread if rx dropped.
                            break;
                        }
                    }
                    Err(err) => {
                        error!("unexpected dropped tx: {err}");
                        break;
                    }
                }
            }
        });

        Ok(Self {
            tx,
            rx: UnboundedReceiverStream::new(rx),
        })
    }

    pub async fn send(&self, data: impl Into<Bytes>) -> Result<()> {
        let data = data.into();
        let (ready_tx, ready_rx) = oneshot::channel::<Result<()>>();

        self.tx.send(Command::SendData { data, ready_tx })?;

        let result = ready_rx.await?;
        result?;

        Ok(())
    }

    pub async fn recv(&mut self) -> Option<Bytes> {
        self.rx.next().await
    }

    pub async fn verify(&self) -> Result<bool> {
        let (ready_tx, ready_rx) = oneshot::channel::<_>();
        self.tx.send(Command::Verify { ready_tx })?;
        let result = ready_rx.await?;
        Ok(result)
    }

    pub async fn stats(&self) -> Result<Stats> {
        let (ready_tx, ready_rx) = oneshot::channel::<_>();
        self.tx.send(Command::Stats { ready_tx })?;
        let result = ready_rx.await?;
        Ok(result)
    }

    pub async fn bitrate(&self) -> Result<f32> {
        let (ready_tx, ready_rx) = oneshot::channel::<_>();
        self.tx.send(Command::Bitrate { ready_tx })?;
        let result = ready_rx.await?;
        Ok(result)
    }
}

impl Stream for RNodeInterfaceAsync {
    type Item = Bytes;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.rx.poll_next_unpin(cx)
    }
}
