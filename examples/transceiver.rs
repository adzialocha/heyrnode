//! Send and receive UTF-8 byte-strings
use std::sync::mpsc;
use std::thread;

use heyrnode::RNodeInterface;
use heyrnode::config::RadioConfig;
use tracing::info;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logging();

    // This works well with Heltec WiFi LoRa 32 v4.
    let config = RadioConfig::default()
        .bandwidth(125_000)
        .frequency(868_000_000)
        .tx_power(2)
        .spread_factor(7)
        .coding_rate(5);

    let (rnode, rx) = RNodeInterface::new("/dev/ttyACM0", config)?;
    info!("verify = {}", rnode.verify());

    let (line_tx, line_rx) = mpsc::channel::<String>();

    thread::spawn(move || {
        loop {
            let line = line_rx.recv().expect("receive stdin string");

            if line.starts_with("/stats") {
                println!("{:?}", rnode.stats());
            } else {
                rnode.send(line.trim().as_bytes()).expect("send to RNode");
            }
        }
    });

    thread::spawn(move || input_loop(line_tx));

    loop {
        match rx.recv() {
            Ok(bytes) => {
                let message = String::from_utf8(bytes).expect("decode UTF-8 bytes");
                println!("> {message}");
            }
            Err(err) => {
                eprintln!("failed receiving message: {err}");
            }
        }
    }
}

fn input_loop(line_tx: mpsc::Sender<String>) -> Result<(), std::io::Error> {
    let mut buffer = String::new();
    let stdin = std::io::stdin();
    loop {
        stdin.read_line(&mut buffer)?;
        line_tx
            .send(buffer.clone())
            .map_err(|err| std::io::Error::other(err))?;
        buffer.clear();
    }
}

fn setup_logging() {
    if std::env::var("RUST_LOG").is_ok() {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .try_init();
    }
}
