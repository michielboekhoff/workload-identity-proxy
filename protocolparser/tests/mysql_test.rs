use eyre::Result;
use std::io::Read;
use std::net::TcpStream;
use testcontainers::clients::Cli;

use protocolparser;

mod common;

#[test]
fn it_reads_handshake() -> Result<()> {
    let client = Cli::default();
    let container = common::setup_mysql(&client);
    let mysql_address = container.get_host_port_ipv4(3306);
    let mut conn = TcpStream::connect(format!("localhost:{}", mysql_address))?;
    let mut bytes: [u8; 128] = [0; 128];
    conn.read(&mut bytes)?;

    println!("bytes: {:?}", bytes);

    let mut binding = &bytes[..];
    let handshake = protocolparser::parse_handshake(&mut binding)?;
    assert_eq!(handshake.server_version, "8.1.0");

    Ok(())
}
