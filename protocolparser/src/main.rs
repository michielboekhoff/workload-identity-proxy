use std::{error::Error, io::Read, net::TcpStream, str::from_utf8};

use nom::{
    bytes::streaming::{tag, take, take_till},
    combinator::map_res,
    number::{streaming, Endianness},
    sequence::{self, tuple},
    IResult,
};

#[derive(Debug)]
struct Packet<'a> {
    payload_length: u32, // in reality is an int<3>, but will always fit in u32
    sequence_id: u8,
    payload: &'a [u8],
}

#[derive(Debug)]
struct Handshake<'a> {
    server_version: &'a str,
    capability_flags: u32,
}

fn parse_null_terminated_string<'a>(bytes: &'a [u8]) -> IResult<&'a [u8], &'a str> {
    let parser = sequence::terminated(take_till(|b| b == 0u8), tag(&[0]));
    return map_res(parser, from_utf8)(bytes);
}

fn parse_packet<'a>(bytes: &'a [u8]) -> Result<Packet, nom::Err<nom::error::Error<&'a [u8]>>> {
    // Integers in MySQL's wire protocol are LSB-first, so little endian 🤷
    let mut parser = tuple((streaming::u24(Endianness::Little), streaming::u8));
    let (i, (payload_length, sequence_id)) = parser(bytes)?;

    Ok(Packet {
        payload_length,
        sequence_id,
        payload: i,
    })
}

fn parse_handshake<'a>(bytes: &'a [u8]) -> IResult<&'a [u8], Handshake<'a>> {
    let (i, _) = tag(&[10])(bytes)?;
    let (i, server_version) = parse_null_terminated_string(i)?;
    // Ignore 13 bytes which includes the thread ID & auth-plugin-data-part-1
    let (i, _) = take(13usize)(i)?;
    let (i, capability_flags_1) = streaming::u16(Endianness::Little)(i)?;
    let (i, _) = take(1usize)(i)?; // character_set
    let (i, _) = take(2usize)(i)?; // status_flags
    let (i, capability_flags_2) = streaming::u16(Endianness::Little)(i)?;

    let capability_flags: u32 =
        (u32::from(capability_flags_2) << 16) | u32::from(capability_flags_1);

    let handshake = Handshake {
        server_version,
        capability_flags,
    };

    Ok((i, handshake))
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut client = TcpStream::connect(("misp-nhss-soc-db.mysql.database.azure.com", 3306))?;
    let mut buff: [u8; 64] = [0; 64];
    let _num_bytes = client.read_exact(&mut buff)?;

    let packet = parse_packet(&buff).unwrap();

    println!("{:?}", packet);

    let handshake = parse_handshake(packet.payload);

    println!("{:?}", handshake);

    match handshake {
        Ok((_, msg)) => println!("{:?}", (msg.capability_flags & 2048) > 0),
        Err(_) => {}
    }

    Ok(())
}
