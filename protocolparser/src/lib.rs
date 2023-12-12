use eyre::{eyre, Result};
use std::str::from_utf8;

use winnow::{
    binary::{self, length_and_then, Endianness},
    combinator::{preceded, terminated},
    token::{tag, take, take_while},
    Bytes, PResult, Parser, Partial,
};

type Stream<'a> = Partial<&'a Bytes>;

#[derive(Debug)]
struct Packet<'a> {
    payload_length: u32, // in reality is an int<3>, but will always fit in u32
    sequence_id: u8,
    payload: &'a [u8],
}

#[derive(Debug, Clone)]
pub struct Handshake<'a> {
    pub server_version: &'a str,
    capability_flags: u32,
}

fn parse_null_terminated_string<'a>(bytes: &mut Stream<'a>) -> PResult<&'a str> {
    terminated(take_while(0.., |c| c != 0), 0u8)
        .try_map(from_utf8)
        .parse_next(bytes)
}

fn packet_length<'a>(bytes: &mut Stream<'a>) -> PResult<u32> {
    // This is to include the sequence number, which would otherwise be lost
    const PACKET_HEADER_SIZE: u32 = 1;

    let l = binary::u24(Endianness::Little)
        .map(|len| len + PACKET_HEADER_SIZE)
        .parse_next(bytes);

    match l {
        Err(ref e) => println!("Err: {}", e),
        Ok(len) => println!("Length: {}", len),
    }

    return l;
}

struct PacketHeader {
    sequence_id: u8,
}

fn parse_packet_header<'a>(bytes: &mut Stream<'a>) -> PResult<PacketHeader> {
    let mut parser = binary::u8;

    let sequence_id = parser.parse_next(bytes)?;

    return Ok(PacketHeader { sequence_id });
}

fn packet_parser<'a>(bytes: &mut Stream<'a>) -> PResult<Handshake<'a>> {
    let handshake_parser = preceded(tag(&[10]), parse_null_terminated_string);
    let mut parser = (parse_packet_header, handshake_parser);

    let (header, handshake) = parser.parse_next(bytes)?;

    return Ok(Handshake {
        // sequence_id: header.sequence_id,
        capability_flags: 0,
        server_version: handshake,
    });
}

pub fn parse_handshake<'a>(bytes: &'a mut &'a [u8]) -> Result<Handshake<'a>> {
    let mut s: Stream = Partial::new(Bytes::new(bytes));
    let handshake_packet = length_and_then(packet_length, packet_parser)
        .parse_next(&mut s)
        .map_err(|e| eyre!(e.to_string()))?;

    Ok(Handshake {
        server_version: handshake_packet.server_version,
        capability_flags: 0,
    })
}
