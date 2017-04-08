use std::io::Write;
use std::path::PathBuf;
use protob::{named_message, guess_message};
use error::PqrsError;

pub fn forcefully_decode(buf: &[u8],
                         msgtype: &Option<String>,
                         mut out: &mut Write,
                         fdsets: &[PathBuf])
                         -> Result<(), PqrsError> {
    let mut offset = 0;
    let buflen = buf.len();
    while offset < buflen {
        for n in 0..offset + 1 {
            if decode_single_slice(&buf[n..(buflen - offset + n)], msgtype, out, fdsets) {
                return Ok(());
            }
        }
        offset += 1;
    }
    Err(PqrsError::CouldNotDecodeError(String::from("Could not decode")))
}

fn decode_single_slice(buf: &[u8],
                       msgtype: &Option<String>,
                       mut out: &mut Write,
                       fdsets: &[PathBuf])
                       -> bool {
    match *msgtype {
        Some(ref x) => {
            match named_message(buf, x, &mut out, fdsets) {
                Ok(_) => true,
                Err(_) => false,
            }
        }
        None => {
            match guess_message(buf, &mut out, fdsets) {
                Ok(_) => true,
                Err(_) => false,
            }
        }
    }
}
