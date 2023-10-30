use std::{
    error::Error,
    io::Write,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use clap::Parser;
use rustls::{ClientConfig, RootCertStore, ServerName};
use structured_logger::{async_json::new_writer, Builder};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::TcpListener,
};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long, default_value_t = String::from("127.0.0.1"))]
    bind_address: String,

    #[arg(short, long, default_value_t = 5432)]
    port: u16,

    #[arg(short, long)]
    database_hostname: String,
}

fn create_tls_conn(server_name: String) -> Result<rustls::ClientConnection, impl Error> {
    let root_store = RootCertStore::empty();
    let cfg = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let parsed_server_name = ServerName::try_from(server_name.as_str()).unwrap();

    return Ok::<rustls::client::ClientConnection, Box<rustls::Error>>(
        rustls::ClientConnection::new(Arc::new(cfg), parsed_server_name)?,
    );
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    Builder::with_level("info")
        .with_target_writer("*", new_writer(tokio::io::stdout()))
        .init();

    let mut conn = create_tls_conn(args.database_hostname)?;

    let listener = TcpListener::bind((args.bind_address.clone(), args.port)).await?;

    let term = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGTERM, term.clone())?;

    log::info!("Binding to {} on port {}", args.bind_address, args.port);
    while !term.load(Ordering::Relaxed) {
        let (socket, _) = listener.accept().await?;
        let mut reader = BufReader::new(socket);
        let buf = reader.fill_buf().await?;
        let mut writer = conn.writer();

        match writer.write_all(buf) {
            Err(err) => log::error!("Error proxying packet: {}", err),
            _ => {}
        }
    }

    Ok(())
}
