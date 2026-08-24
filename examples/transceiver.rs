use std::sync::mpsc;
use std::thread;

use heyrnode::RNodeInterface;
use heyrnode::config::RadioConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = RadioConfig::default();
    let (rnode, rx) = RNodeInterface::new("/dev/ttyACM0", config)?;
    println!("verify = {}", rnode.verify());

    let (line_tx, line_rx) = mpsc::channel::<String>();

    thread::spawn(move || {
        loop {
            let line = line_rx.recv().unwrap();
            rnode.send(line.trim().as_bytes()).unwrap();
        }
    });

    thread::spawn(move || input_loop(line_tx));

    loop {
        match rx.recv() {
            Ok(bytes) => {
                let message = String::from_utf8(bytes).unwrap();
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
