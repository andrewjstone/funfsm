/// This module is used for framing data with a 4 byte header. It can be used on any type
/// implementing `std::io::Read`.
///

use error::StdError;
use std::io::{Read, Write};
use std::mem;
use std::slice;

/// Convert a 4 byte array in big endian to a u32 in native order
pub fn array_to_u32(bytes: [u8; 4]) -> u32 {
    let len: u32 = unsafe { mem::transmute(bytes) };
    u32::from_be(len)
}

/// Convert a u32 in native order to a 4 byte vec in big endian
pub fn u32_to_vec(n: u32) -> Vec<u8> {
    unsafe {
        let ptr: *const u8 = mem::transmute(&n.to_be());
        slice::from_raw_parts(ptr, 4).to_vec()
    }
}

// The first parameter of each tuple is the number of bytes currently read into the buffer
pub enum ReadState {
    Header(usize, [u8;4]),
    Msg(usize, Vec<u8>)
}

impl ReadState {
    pub fn new() -> ReadState {
        ReadState::Header(0, [0;4])
    }

    pub fn read<T: Read>(self, reader: &mut T) -> (ReadState, Result<Option<Vec<u8>>, StdError>) {
        match self {
            ReadState::Header(bytes_read, mut buf) => {
                match reader.read(&mut buf[bytes_read..]) {
                    Ok(0) => (ReadState::new(), Err(StdError::Eof)),
                    Ok(n) => {
                        let total = n + bytes_read;
                        if total == 4 {
                            let len = array_to_u32(buf);
                            (ReadState::Msg(0, vec![0; len as usize]), Ok(None))
                        } else {
                            (ReadState::Header(total, buf), Ok(None))
                        }
                    },
                    Err(err) => (ReadState::new(), Err(StdError::from(err)))
                }
            },
            ReadState::Msg(bytes_read, mut buf) => {
                match reader.read(&mut buf[bytes_read..]) {
                    Ok(0) => (ReadState::new(), Err(StdError::Eof)),
                    Ok(n) => {
                        let total = n + bytes_read;
                        if total == buf.len() {
                            return (ReadState::new(), Ok(Some(buf)));
                        } else {
                            (ReadState::Msg(total, buf), Ok(None))
                        }
                    },
                    Err(err) => (ReadState::new(), Err(StdError::from(err)))
                }
            }
        }
    }
}

pub enum WriteState {
    Writing {pending: Vec<Vec<u8>>, written: usize, writing: Vec<u8>},
    Empty
}

impl WriteState {
    pub fn new() -> WriteState {
        WriteState::Empty
    }

    pub fn push(self, msg: Vec<u8>) -> WriteState {
        let header = u32_to_vec(msg.len() as u32);
        match self {
            WriteState::Empty =>
                WriteState::Writing {pending: vec![msg], written: 0, writing: header},
            WriteState::Writing{mut pending, written, writing} => {
                pending.push(header);
                pending.push(msg);
                WriteState::Writing {pending: pending, written: written, writing: writing}
            }
        }
    }

    // Return Ok(true) if there is more data to write
    pub fn write<T: Write>(self, writer: &mut T) -> Result<(bool, WriteState), StdError> {
        match self {
            WriteState::Empty => Ok((false, WriteState::Empty)),
            WriteState::Writing {mut pending, mut written, mut writing} => {
                let n = try!(writer.write(&writing[written..]));
                written += n;
                if written == writing.len() {
                    match pending.pop() {
                        None => return Ok((false, WriteState::Empty)),
                        Some(msg) => {
                            written = 0;
                            writing = msg;
                        }
                    }
                };
                let new_state = WriteState::Writing { pending: pending,
                                                      written: written,
                                                      writing: writing};
                Ok((true, new_state))
            }
        }
    }
}
