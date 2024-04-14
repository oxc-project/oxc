use serde::Deserialize;

use crate::typecheck::response::Response;

use super::{ProtocolError, EOL_LENGTH};

pub fn read_message<T>(mut result_stream: impl std::io::Read) -> Result<T, ProtocolError>
where
    T: for<'de> Deserialize<'de>,
{
    const PREFIX_LENGTH: usize = 16;

    let mut buf = [0u8; 40]; // ["Content-Length: " + usize + "\r\n\r\n"]
    let mut offset = 0;
    let mut prefix_length = PREFIX_LENGTH;
    loop {
        let read_bytes = result_stream.read(&mut buf[offset..]);
        match read_bytes {
            Ok(0) => return Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof).into()),
            Ok(n) => {
                offset += n;

                // Message too short we need more bytes
                if offset <= PREFIX_LENGTH {
                    continue;
                }

                for i in prefix_length..offset {
                    match buf[i] {
                        b'0'..=b'9' => {}
                        b'\r' => {
                            let length =
                                std::str::from_utf8(&buf[PREFIX_LENGTH..i])?.parse::<usize>()? - 1;

                            let msg_start = i + 4;
                            let mut msg = vec![0u8; length + EOL_LENGTH];

                            let mut msg_writer = msg.as_mut_slice();
                            if offset < msg_start {
                                result_stream.read_exact(&mut buf[offset..msg_start])?;
                            } else if msg_start < offset {
                                std::io::Write::write(&mut msg_writer, &buf[msg_start..offset])?;
                            }

                            result_stream.read_exact(msg_writer)?;
                            msg.truncate(length);

                            let mut deserializer = serde_json::Deserializer::from_slice(&msg);
                            let response = Response::<T>::deserialize(&mut deserializer)?;
                            return Result::<T, ProtocolError>::from(response);
                        }
                        _ => return Err(ProtocolError::UnexpectedCharacter),
                    }
                }

                prefix_length = offset;
            }
            Err(err) if err.kind() == std::io::ErrorKind::Interrupted => {}
            Err(err) => return Err(err.into()),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::typecheck::response::StatusResponse;

    use super::*;

    const EOL_STR: &'static str = if EOL_LENGTH == 1 { "\n" } else { "\r\n" };

    struct ChunkedReader<'a, T: Iterator<Item = &'a [u8]>> {
        iter: T,
    }

    impl<'a, T: Iterator<Item = &'a [u8]>> std::io::Read for ChunkedReader<'a, T> {
        fn read(&mut self, mut buf: &mut [u8]) -> std::io::Result<usize> {
            let chunk = self.iter.next();
            match chunk {
                None => Ok(0),
                Some(data) => std::io::Write::write(&mut buf, data),
            }
        }
    }

    #[test]
    fn given_single_chunk_then_reads_message() {
        let msg = r#"{"seq":0,"type":"response","command":"status","request_seq":0,"success":true,"body":{"version":"5.3.3"}}"#;
        let str = format!("{}{}", ["Content-Length: 105", "", msg].join("\r\n"), EOL_STR);
        let mut buf = str.as_bytes();
        let result: StatusResponse = read_message(&mut buf).unwrap();
        assert_eq!(result, StatusResponse { version: "5.3.3".into() });
    }

    #[test]
    fn given_header_in_separate_chunk_then_reads_message() {
        let msg = r#"{"seq":0,"type":"response","command":"status","request_seq":0,"success":true,"body":{"version":"5.3.3"}}"#;
        let msg_chunk = format!("{}{}", msg, EOL_STR);
        let reader = ChunkedReader {
            iter: [b"Content-Length: 105\r\n\r\n".as_slice(), msg_chunk.as_bytes()].into_iter(),
        };

        let result: StatusResponse = read_message(reader).unwrap();
        assert_eq!(result, StatusResponse { version: "5.3.3".into() });
    }

    #[test]
    fn given_chunked_header_then_reads_message() {
        let msg = r#"{"seq":0,"type":"response","command":"status","request_seq":0,"success":true,"body":{"version":"5.3.3"}}"#;
        let msg_chunk = format!("{}{}", msg, EOL_STR);
        let reader = ChunkedReader {
            iter: [b"Content-Length: 1".as_slice(), b"05\r\n\r\n", msg_chunk.as_bytes()]
                .into_iter(),
        };

        let result: StatusResponse = read_message(reader).unwrap();
        assert_eq!(result, StatusResponse { version: "5.3.3".into() });
    }

    #[test]
    fn given_chunked_delimiter_then_reads_message() {
        let msg = r#"{"seq":0,"type":"response","command":"status","request_seq":0,"success":true,"body":{"version":"5.3.3"}}"#;
        let msg_chunk = format!("{}{}", msg, EOL_STR);
        let reader = ChunkedReader {
            iter: [b"Content-Length: 105\r\n\r".as_slice(), b"\n", msg_chunk.as_bytes()]
                .into_iter(),
        };

        let result: StatusResponse = read_message(reader).unwrap();
        assert_eq!(result, StatusResponse { version: "5.3.3".into() });
    }

    #[test]
    fn given_error_response_then_deserializes_into_error() {
        let msg = r#"{"seq":0,"type":"response","command":"status","request_seq":0,"success":false,"message":"Error processing request.\n at bar (foo.js:123:45)"}"#;
        let str = format!("{}{}", ["Content-Length: 142", "", msg].join("\r\n"), EOL_STR);
        let mut buf = str.as_bytes();
        let result = read_message::<StatusResponse>(&mut buf).unwrap_err();
        if let ProtocolError::CommandFailed(error) = result {
            assert_eq!(error, "Error processing request.\n at bar (foo.js:123:45)");
        } else {
            panic!("{:#?}", result)
        }
    }
}
