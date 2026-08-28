use std::pin::Pin;
use std::task::{Context, Poll};
use std::thread;

use futures_util::{Stream, StreamExt};
use tokio::sync::{mpsc, oneshot};
use tokio_stream::wrappers::ReceiverStream;

use crate::error::Result;
use crate::{RNodeInterface, RadioConfig};

type Data = Vec<u8>;

type SendCommand = (Data, oneshot::Sender<Result<()>>);

#[derive(Debug)]
pub struct RNodeInterfaceAsync {
    tx: mpsc::Sender<SendCommand>,
    rx: ReceiverStream<Data>,
}

impl RNodeInterfaceAsync {
    pub fn new(port: &str, config: RadioConfig) -> Result<Self> {
        let (tx, mut rx) = mpsc::channel::<SendCommand>(64);
        let (recv_tx, recv_rx) = mpsc::channel::<Data>(64);

        let (iface, iface_rx) = RNodeInterface::new(port, config)?;

        thread::spawn(move || {
            loop {
                let Some((data, ready_tx)) = rx.blocking_recv() else {
                    // Close thread if tx got dropped.
                    break;
                };

                let result = iface.send(data);
                let _ = ready_tx.send(result);
            }
        });

        thread::spawn(move || {
            loop {
                match iface_rx.recv() {
                    Ok(data) => {
                        if recv_tx.blocking_send(data).is_err() {
                            // Close thread if recv_rx got dropped.
                            break;
                        }
                    }
                    Err(_err) => {
                        break;
                    }
                }
            }
        });

        Ok(Self {
            tx,
            rx: ReceiverStream::new(recv_rx),
        })
    }

    pub async fn send(&self, data: impl AsRef<[u8]>) -> Result<()> {
        let data = data.as_ref().to_vec();
        let (ready_tx, ready_rx) = oneshot::channel::<Result<()>>();

        self.tx.send((data, ready_tx)).await?;

        let result = ready_rx.await?;
        result?;

        Ok(())
    }
}

impl Stream for RNodeInterfaceAsync {
    type Item = Data;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.rx.poll_next_unpin(cx)
    }
}
