//! Async tokio API of heyrnode.

use futures_util::StreamExt;
use heyrnode::{RNodeInterfaceAsync, RadioConfig};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // This works well with Heltec WiFi LoRa 32 v4.
    let config = RadioConfig::default()
        .bandwidth(125_000)
        .frequency(868_000_000)
        .tx_power(2)
        .spread_factor(7)
        .coding_rate(5);

    let mut rnode = RNodeInterfaceAsync::new("/dev/ttyACM0", config)?;

    rnode.send(b"Hello, World!").await?;

    while let Some(data) = rnode.next().await {
        println!("{:?}", data);
    }

    Ok(())
}
