//! Async tokio API of heyrnode.

use futures_util::StreamExt;
use heyrnode::{RNodeInterfaceAsync, RadioConfig};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = RadioConfig::default();
    let mut rnode = RNodeInterfaceAsync::new("/dev/ttyACM0", config)?;

    rnode.send(b"Hello, World!").await?;

    while let Some(data) = rnode.next().await {
        println!("{:?}", data);
    }

    Ok(())
}
