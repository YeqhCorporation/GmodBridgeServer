#[derive(Debug, Clone, PartialEq)]
pub enum FrameError {
    MissingContentLength,
    InvalidContentLength,
    MessageTooLarge { length: usize, max: usize },
}

pub const MAX_FRAME_BYTES: usize = 1024 * 1024;

pub fn encode_frame(value: &[u8]) -> Vec<u8> {
    let mut out = format!(
        "Content-Length: {}\r\nContent-Type: application/json\r\n\r\n",
        value.len()
    )
    .into_bytes();
    out.extend_from_slice(value);
    out
}

pub struct FrameDecoder {
    buffer: Vec<u8>,
    max_frame_bytes: usize,
}

impl FrameDecoder {
    pub fn new(max_frame_bytes: usize) -> Self {
        Self {
            buffer: Vec::new(),
            max_frame_bytes,
        }
    }

    pub fn push(&mut self, bytes: &[u8]) -> Result<Vec<Vec<u8>>, FrameError> {
        self.buffer.extend_from_slice(bytes);
        let mut messages = Vec::new();

        loop {
            let Some(header_end) = find_header_end(&self.buffer) else {
                return Ok(messages);
            };

            let header = String::from_utf8_lossy(&self.buffer[..header_end]);
            let length = parse_content_length(&header)?;

            if length > self.max_frame_bytes {
                return Err(FrameError::MessageTooLarge {
                    length,
                    max: self.max_frame_bytes,
                });
            }

            let body_start = header_end + 4;
            let body_end = body_start + length;

            if self.buffer.len() < body_end {
                return Ok(messages);
            }

            messages.push(self.buffer[body_start..body_end].to_vec());
            self.buffer.drain(..body_end);
        }
    }
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n")
}

fn parse_content_length(header: &str) -> Result<usize, FrameError> {
    for line in header.lines() {
        let mut parts = line.splitn(2, ':');
        let Some(name) = parts.next() else {
            continue;
        };
        let Some(value) = parts.next() else {
            continue;
        };

        if name.eq_ignore_ascii_case("Content-Length") {
            return value
                .trim()
                .parse::<usize>()
                .map_err(|_| FrameError::InvalidContentLength);
        }
    }

    Err(FrameError::MissingContentLength)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_lsp_style_frame() {
        let frame = encode_frame(br#"{"ok":true}"#);
        let text = String::from_utf8(frame).unwrap();

        assert!(text.starts_with("Content-Length: 11\r\n"));
        assert!(text.contains("Content-Type: application/json\r\n\r\n"));
        assert!(text.ends_with(r#"{"ok":true}"#));
    }

    #[test]
    fn decodes_fragmented_frame() {
        let frame = encode_frame(br#"{"v":"gabp/1"}"#);
        let mut decoder = FrameDecoder::new(MAX_FRAME_BYTES);

        assert!(decoder.push(&frame[..10]).unwrap().is_empty());
        let messages = decoder.push(&frame[10..]).unwrap();

        assert_eq!(messages, vec![br#"{"v":"gabp/1"}"#.to_vec()]);
    }
}
