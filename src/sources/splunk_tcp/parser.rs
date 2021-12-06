use bytes::{Buf, Bytes};
use std::{fmt, str};
use std::convert::TryInto;

pub struct ParsedSplunkTCPEvent {
    pub(crate) header: SplunkTCPHeader,
}

impl ParsedSplunkTCPEvent {
    pub fn new(header: SplunkTCPHeader) -> Self {
        ParsedSplunkTCPEvent {
            header
        }
    }

    pub fn to_string(&self) -> String {
        // We don't want to disclose the secret
        format!("ParsedSplunkTCPEvent(header-> {})", &self.header)
    }
}

impl fmt::Display for ParsedSplunkTCPEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.header.to_string().as_str())
    }
}

pub struct SplunkTCPHeader {
    protocol: String,
    hostname: String,
    port: i16,
}

impl SplunkTCPHeader {
    pub fn to_string(&self) -> String {
        // We don't want to disclose the secret
        format!("SplunkTCPHeader(protocol-> {}, hostname-> {}, port-> {})", &self.protocol, &self.hostname, &self.port)
    }
}

impl fmt::Display for SplunkTCPHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.to_string().as_str())
    }
}

pub fn parse_header(frame: &Bytes) -> SplunkTCPHeader {
    let head = frame.slice(0..399);
    let protocol = head.slice(0..127);
    let hostname = head.slice(128..255);
    let port = head.slice(256..384);

    // i16::from_be_bytes(port.chunk().try_into().unwrap())
    return SplunkTCPHeader {
        protocol: bytes_to_string(&protocol),
        hostname: bytes_to_string(&hostname),
        port: bytes_to_i16(&port),
    };
}

fn bytes_to_string(b: &Bytes) -> String {
    str::from_utf8(b.chunk()).unwrap().to_string().trim_end_matches(char::from(0)).to_string()
}

fn bytes_to_i16(b: &Bytes) -> i16 {
    println!("{:?}", b);
    i16::from_be_bytes(b.chunk().try_into().unwrap())
}