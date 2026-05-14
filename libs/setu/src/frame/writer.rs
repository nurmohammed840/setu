use lipi::Encode;

use crate::transport::http::HttpResponse;
use crate::{
    Trailer,
    frame::{FrameHeader, LenBE},
};

impl HttpResponse {
    pub(crate) fn send_final_message(self, msg: Vec<u8>) -> Result<(), h2::Error> {
        self.write_unbound(encode_payload(msg))
    }
}

fn encode_payload(msg: Vec<u8>) -> Vec<u8> {
    let len = LenBE::new(msg.len());
    let header = FrameHeader::new(len.size, false);

    let mut frame =
        Vec::with_capacity(1 + len.size as usize + msg.len() + Trailer::MIN_ENCODED_LEN);

    frame.push(header.encode());
    frame.extend_from_slice(&*len);
    frame.extend_from_slice(&msg);

    let _ = Trailer::default().encode(&mut frame);
    frame
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_frame() {
        let msg = b"Hello, World".to_vec();
        let raw = encode_payload(msg);
        assert_eq!(raw.len(), 17);
    }
}
