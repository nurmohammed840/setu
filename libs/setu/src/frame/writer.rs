use crate::transport::http::HttpResponse;
use crate::{
    Trailer,
    frame::{FrameHeader, LenBE},
};

impl HttpResponse {
    pub(crate) fn send_final_message(mut self, msg: Vec<u8>) {
        self.add_setu_content_type_header();
        let _ = self.write_unbound(encode_as_frame(msg));
    }
}

fn encode_as_frame(msg: Vec<u8>) -> Vec<u8> {
    let len = LenBE::new(msg.len());
    let mut frame =
        Vec::with_capacity(1 + len.size as usize + msg.len() + Trailer::OK_ENCODED.len());

    frame.push(FrameHeader::new(None, len.size).encode());
    frame.extend_from_slice(&len);
    frame.extend_from_slice(&msg);
    frame.extend_from_slice(&Trailer::OK_ENCODED);
    frame
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_frame() {
        let raw = encode_as_frame(b"67".to_vec());
        assert_eq!(raw, [0, 2, 54, 55, 2, 0]);
    }
}
