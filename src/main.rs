use std::fmt::{Debug, Formatter};
use std::io::{stdin, stdout, Read, Stdin, Stdout, Write};
use byteorder::{ByteOrder, ReadBytesExt};
use serde::{Serialize, Deserialize};

fn main() {
    eprintln!("Hello, world!");
    let stdin = stdin();
    let version_header = dbg!(Header::read(stdin));
    eprintln!("Version header: {:?}", version_header);
    let version = Version { version: "Some random string".to_string() };
    eprintln!("Version: {version:?}");
    let mut version_response_header = version_header.respond();
    version_response_header.body = dbg!(rmp_serde::encode::to_vec_named(&version)).unwrap();
    dbg!(version_response_header.write(&mut stdout()));
    eprintln!("done");
}

struct Header {
    sequence: u32,
    // topic len 2
    // body len 4
    reserved: u32,
    topic: String,
    body: Vec<u8>,
}

impl Header {
    fn read(mut stdin: Stdin) -> Header {
        let mut fixed_buffer = vec![0u8; 14];
        dbg!(stdin.read(&mut fixed_buffer)).unwrap();
        assert_eq!(fixed_buffer.len(), 14);
        let sequence = dbg!(byteorder::BigEndian::read_u32(&fixed_buffer[0..4]));
        let topic_length = dbg!(byteorder::BigEndian::read_u16(&fixed_buffer[4..6]));
        let body_length = dbg!(byteorder::BigEndian::read_u32(&fixed_buffer[6..10]));
        let reserved = dbg!(byteorder::BigEndian::read_u32(&fixed_buffer[10..14]));
        let mut topic = vec![0u8; topic_length as usize];
        dbg!(stdin.read(&mut topic).unwrap());
        eprintln!("Topic length: {}", topic_length);
        let mut body = vec![0u8; dbg!(body_length) as usize];
        eprintln!("trying to read body ({} bytes)", body_length);
        if body_length > 0 {
            dbg!(stdin.read(&mut body)).unwrap();
        }
        eprintln!("finished reading");
        Header {
            sequence,
            reserved,
            topic: String::from_utf8(dbg!(topic)).unwrap(),
            body,
        }
    }

    fn write(&self, stdout: &mut Stdout) {
        let mut buf = vec![0u8; 14 + self.body.len() + self.topic.len()];
        byteorder::BigEndian::write_u32(&mut buf[0..4], self.sequence);
        byteorder::BigEndian::write_u16(&mut buf[4..6], self.topic.len() as u16);
        byteorder::BigEndian::write_u32(&mut buf[6..10], self.body.len() as u32);
        byteorder::BigEndian::write_u32(&mut buf[10..14], self.reserved as u32);
        buf[14..14 + self.topic.len()].copy_from_slice(dbg!(&self.topic).as_bytes());
        buf[14 + self.topic.len()..].copy_from_slice(&self.body);

        dbg!(stdout.write(dbg!(&buf))).unwrap();
    }

    fn respond(&self) -> Self {
        Header {
            sequence: self.sequence,
            reserved: 0,
            topic: self.topic.clone(),
            body: vec![],
        }
    }
}

impl Debug for Header {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Header").field("sequence", &self.sequence).field("topic", &self.topic).field("bodyBytes", &self.body.len()).finish()
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Version {
    version: String,
}