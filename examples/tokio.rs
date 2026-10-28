//! Async tokio API of heyrnode.

use futures_util::StreamExt;
use heyrnode::{RNodeInterfaceAsync, RadioConfig};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = RadioConfig::default();
    let (rnode, mut rx) = RNodeInterfaceAsync::new("/dev/ttyACM0", config)?;

    rnode.send(b"Hello, World!".to_vec()).await?;

    while let Some(data) = rx.next().await {
        println!("{:?}", data);
    }

    Ok(())
}
