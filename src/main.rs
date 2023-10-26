use std::{
    io,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use clap::Parser;
use structured_logger::{async_json::new_writer, Builder};
use tokio::net::TcpListener;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long, default_value_t = String::from("127.0.0.1"))]
    bind_address: String,

    #[arg(short, long, default_value_t = 5432)]
    port: u16,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Args::parse();

    Builder::with_level("info")
        .with_target_writer("*", new_writer(tokio::io::stdout()))
        .init();

    let listener = TcpListener::bind((args.bind_address.clone(), args.port)).await?;

    let term = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGTERM, term.clone())?;

    log::info!("Binding to {} on port {}", args.bind_address, args.port);
    while !term.load(Ordering::Relaxed) {
        let (_, _) = listener.accept().await?;
    }

    Ok(())
}
