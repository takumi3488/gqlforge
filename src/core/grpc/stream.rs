use bytes::{Buf, Bytes, BytesMut};

/// Decodes gRPC-framed messages from a byte stream.
///
/// Each gRPC frame consists of a 5-byte header:
///   - 1 byte: compressed flag (0 = uncompressed)
///   - 4 bytes: big-endian message length
///
/// followed by the protobuf-encoded message payload.
pub struct GrpcFrameDecoder {
    buffer: BytesMut,
}

impl Default for GrpcFrameDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl GrpcFrameDecoder {
    pub fn new() -> Self {
        Self { buffer: BytesMut::new() }
    }

    /// Feed a chunk of bytes and extract any complete gRPC frames.
    pub fn decode(&mut self, chunk: Bytes) -> Vec<Bytes> {
        self.buffer.extend_from_slice(&chunk);
        let mut frames = Vec::new();

        loop {
            if self.buffer.len() < 5 {
                break;
            }

            // Read the 4-byte message length (skip 1-byte compressed flag)
            let msg_len = u32::from_be_bytes([
                self.buffer[1],
                self.buffer[2],
                self.buffer[3],
                self.buffer[4],
            ]) as usize;

            let frame_len = 5 + msg_len;
            if self.buffer.len() < frame_len {
                break;
            }

            // Skip the 5-byte header
            self.buffer.advance(5);
            let payload = self.buffer.split_to(msg_len).freeze();
            frames.push(payload);
        }

        frames
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use super::GrpcFrameDecoder;

    fn make_frame(payload: &[u8]) -> Vec<u8> {
        let len = payload.len() as u32;
        let mut frame = Vec::with_capacity(5 + payload.len());
        frame.push(0); // compressed flag
        frame.extend_from_slice(&len.to_be_bytes());
        frame.extend_from_slice(payload);
        frame
    }

    #[test]
    fn test_single_frame() {
        let mut decoder = GrpcFrameDecoder::new();
        let data = make_frame(b"hello");
        let frames = decoder.decode(Bytes::from(data));
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].as_ref(), b"hello");
    }

    #[test]
    fn test_multiple_frames_in_one_chunk() {
        let mut decoder = GrpcFrameDecoder::new();
        let mut data = make_frame(b"first");
        data.extend_from_slice(&make_frame(b"second"));
        let frames = decoder.decode(Bytes::from(data));
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0].as_ref(), b"first");
        assert_eq!(frames[1].as_ref(), b"second");
    }

    #[test]
    fn test_split_across_chunks() {
        let mut decoder = GrpcFrameDecoder::new();
        let data = make_frame(b"split-test");

        // Feed first 3 bytes (partial header)
        let frames = decoder.decode(Bytes::from(data[..3].to_vec()));
        assert_eq!(frames.len(), 0);

        // Feed the rest
        let frames = decoder.decode(Bytes::from(data[3..].to_vec()));
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].as_ref(), b"split-test");
    }

    #[test]
    fn test_empty_payload() {
        let mut decoder = GrpcFrameDecoder::new();
        let data = make_frame(b"");
        let frames = decoder.decode(Bytes::from(data));
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].as_ref(), b"");
    }

    #[test]
    fn test_empty_chunk() {
        let mut decoder = GrpcFrameDecoder::new();
        let frames = decoder.decode(Bytes::new());
        assert_eq!(frames.len(), 0);
    }
}
