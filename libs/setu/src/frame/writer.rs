use http::HeaderValue;
use http::header::CONTENT_TYPE;

use crate::transport::http::HttpResponse;
use crate::{
    Trailer,
    frame::{FrameHeader, LenBE},
};

impl HttpResponse {
    pub(crate) fn send_final_message(mut self, msg: Vec<u8>) {
        self.add_setu_content_type_header();
        let _ = self.write_unbound(encode_payload(msg));
    }
}

fn encode_payload(msg: Vec<u8>) -> Vec<u8> {
    let len = LenBE::new(msg.len());
    let header = FrameHeader::new(None, len.size);

    let mut frame =
        Vec::with_capacity(1 + len.size as usize + msg.len() + Trailer::OK_ENCODED.len());

    frame.push(header.encode());
    frame.extend_from_slice(&*len);
    frame.extend_from_slice(&msg);
    frame.extend_from_slice(&Trailer::OK_ENCODED);
    frame
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Status;

    #[test]
    fn test_encode_frame() {
        let msg = b"Hello, World".to_vec();
        let raw = encode_payload(msg);
        assert_eq!(raw.len(), 16);
    }

    #[test]
    fn test_name() {
        let header = FrameHeader::new(Some(Status::Ok), 3);
        let len = LenBE::new(3);
        println!("header.encode(): {:#?}", header.encode());
        println!("len: {:?}", &*len);
    }
}
