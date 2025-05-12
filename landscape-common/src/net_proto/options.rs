use std::net::Ipv4Addr;

use super::error::{DecodeResult, EncodeResult};
use dhcproto::Decoder as DHCPDecoder;
use dhcproto::Encoder as DHCPEncoder;

pub type Decoder<'a> = DHCPDecoder<'a>;
pub type Encoder<'a> = DHCPEncoder<'a>;

pub trait Decodable: Sized {
    fn decode(decoder: &mut Decoder<'_>) -> DecodeResult<Self>;
}

pub trait Encodable {
    fn encode(&self, e: &mut Encoder<'_>) -> EncodeResult<()>;
}

// ()

impl Decodable for () {
    fn decode(_: &mut Decoder<'_>) -> DecodeResult<()> {
        Ok(())
    }
}

impl Encodable for () {
    fn encode(&self, _: &mut Encoder<'_>) -> EncodeResult<()> {
        Ok(())
    }
}

// Ipv4Addr

impl Decodable for Ipv4Addr {
    fn decode(decoder: &mut Decoder<'_>) -> DecodeResult<Ipv4Addr> {
        Ok(decoder.read_ipv4(4)?)
    }
}

impl Encodable for Ipv4Addr {
    fn encode(&self, e: &mut Encoder<'_>) -> EncodeResult<()> {
        Ok(e.write_u32(self.to_bits().to_be())?)
    }
}
