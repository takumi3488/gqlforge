use bytes::{Bytes, BytesMut};

/// Incrementally parses SSE (Server-Sent Events) from a byte stream.
///
/// Each SSE event is delimited by a blank line (`\n\n`).
/// Only `data:` lines are extracted and concatenated; `event:`, `id:`,
/// and `retry:` lines are ignored.
pub struct SseEventParser {
    buffer: BytesMut,
}

impl Default for SseEventParser {
    fn default() -> Self {
        Self::new()
    }
}

impl SseEventParser {
    pub fn new() -> Self {
        Self { buffer: BytesMut::new() }
    }

    /// Feed a chunk of bytes and extract complete SSE event data payloads.
    pub fn decode(&mut self, chunk: Bytes) -> Vec<String> {
        self.buffer.extend_from_slice(&chunk);
        let mut events = Vec::new();

        loop {
            let buf = &self.buffer[..];
            // Find the event boundary: a blank line (\n\n)
            let boundary = find_double_newline(buf);
            let Some(pos) = boundary else {
                break;
            };

            // Extract the event block (everything before the \n\n)
            let event_block = &buf[..pos];
            let event_str = String::from_utf8_lossy(event_block);

            // Parse data lines from the event block
            let data = extract_data_lines(&event_str);
            if !data.is_empty() {
                events.push(data);
            }

            // Advance past the event block and the \n\n delimiter
            let advance = pos + 2;
            let _ = self.buffer.split_to(advance);
        }

        events
    }
}

/// Find the position of the first `\n\n` in the buffer.
fn find_double_newline(buf: &[u8]) -> Option<usize> {
    buf.windows(2).position(|w| w == b"\n\n")
}

/// Extract and join the `data:` fields from an SSE event block.
fn extract_data_lines(block: &str) -> String {
    let mut parts = Vec::new();
    for line in block.lines() {
        if let Some(value) = line.strip_prefix("data:") {
            parts.push(value.trim_start().to_string());
        } else if let Some(value) = line.strip_prefix("data") {
            // "data" without colon means empty data line per SSE spec
            if value.is_empty() {
                parts.push(String::new());
            }
        }
    }
    parts.join("\n")
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use super::SseEventParser;

    #[test]
    fn test_single_event() {
        let mut parser = SseEventParser::new();
        let input = Bytes::from("data: {\"hello\":\"world\"}\n\n");
        let events = parser.decode(input);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], "{\"hello\":\"world\"}");
    }

    #[test]
    fn test_multiple_events() {
        let mut parser = SseEventParser::new();
        let input = Bytes::from("data: first\n\ndata: second\n\n");
        let events = parser.decode(input);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0], "first");
        assert_eq!(events[1], "second");
    }

    #[test]
    fn test_split_across_chunks() {
        let mut parser = SseEventParser::new();

        // First chunk: partial event
        let events = parser.decode(Bytes::from("data: hel"));
        assert_eq!(events.len(), 0);

        // Second chunk: rest of event
        let events = parser.decode(Bytes::from("lo\n\n"));
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], "hello");
    }

    #[test]
    fn test_multiline_data() {
        let mut parser = SseEventParser::new();
        let input = Bytes::from("data: line1\ndata: line2\n\n");
        let events = parser.decode(input);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], "line1\nline2");
    }

    #[test]
    fn test_event_and_id_lines_ignored() {
        let mut parser = SseEventParser::new();
        let input = Bytes::from("event: message\nid: 42\ndata: payload\nretry: 1000\n\n");
        let events = parser.decode(input);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], "payload");
    }

    #[test]
    fn test_empty_payload() {
        let mut parser = SseEventParser::new();
        // An event block with no data: lines should not produce an event
        let input = Bytes::from("event: ping\n\n");
        let events = parser.decode(input);
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_empty_chunk() {
        let mut parser = SseEventParser::new();
        let events = parser.decode(Bytes::new());
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_data_without_space() {
        let mut parser = SseEventParser::new();
        let input = Bytes::from("data:no-space\n\n");
        let events = parser.decode(input);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], "no-space");
    }
}
