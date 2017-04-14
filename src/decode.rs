use protob::PqrsDecoder;
use std::io::{Read, Write};
use protobuf::{CodedInputStream, parse_from_reader};
use error::PqrsError;
use std::result::Result;
use serde::Deserialize;
use serde_protobuf::de::Deserializer;
use serde_protobuf::descriptor::Descriptors;
use serde_value::Value;

pub fn decode_single(pqrs_decoder: &PqrsDecoder,
                     buf: &[u8],
                     mut out: &mut Write,
                     force: bool)
                     -> Result<(), PqrsError> {
    if !force {
        return pqrs_decoder.decode_message(buf, &mut out);
    }
    let mut offset = 0;
    let buflen = buf.len();
    while offset < buflen {
        for n in 0..offset + 1 {
            if pqrs_decoder
                   .decode_message(&buf[n..(buflen - offset + n)], &mut out)
                   .is_ok() {
                return Ok(());
            }
        }
        offset += 1;
    }
    Err(PqrsError::CouldNotDecodeError())
}

pub fn decode_leading_varint(lead: &[u8], resulting_size: &mut u64) -> Result<(), PqrsError> {
    let mut leading_varint: &'static [u8] = b"
K
leading_varint.protoxyz.sevag.pqrs\"#
\rLeadingVarint
size (Rsize";

    let proto = parse_from_reader(&mut leading_varint).unwrap();
    let descriptors = Descriptors::from_proto(&proto);
    let byte_is = CodedInputStream::from_bytes(lead);

    let mut deserializer =
        Deserializer::for_named_message(&descriptors, ".xyz.sevag.pqrs.LeadingVarint", byte_is)
            .unwrap();
    *resulting_size = match Value::deserialize(&mut deserializer) {
        Ok(Value::Map(x)) => {
            match *x.values().nth(0).unwrap() {
                Value::U8(ref y) => *y as u64,
                Value::U16(ref y) => *y as u64,
                Value::U32(ref y) => *y as u64,
                Value::U64(ref y) => *y as u64,
                _ => return Err(PqrsError::NoLeadingVarintError()),
            }
        }
        Ok(_) | Err(_) => return Err(PqrsError::CouldNotDecodeError()),
    };
    Ok(())
}

pub fn discover_leading_varint_size(infile: &mut Read) -> Result<(i32, u64), PqrsError> {
    let mut leading_varint_bytesize = 0;
    let mut next_proto_size = 0;
    let mut buf = Vec::new();
    while !decode_leading_varint(&buf, &mut next_proto_size).is_ok() {
        let mut tmpbuf = vec![0; 1];
        infile.read_exact(&mut tmpbuf).unwrap();
        buf.append(&mut tmpbuf);
        leading_varint_bytesize += 1;
    }
    Ok((leading_varint_bytesize, next_proto_size))
}
