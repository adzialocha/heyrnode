//! Async tokio API of heyrnode.

use heyrnode::{RNodeInterfaceAsync, RadioConfig};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = RadioConfig::default();
    let mut rnode = RNodeInterfaceAsync::new("/dev/ttyACM0", config)?;

    rnode.send(b"Hello, World!").await?;

    while let Some(data) = rnode.recv().await {
        println!("{:?}", data);
    }

    Ok(())
}
