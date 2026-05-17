use super::*;
use http::HeaderValue;
use http::{Method, header::CONTENT_TYPE};

const SETU_CONTENT_TYPE: &str = "application/setu";

impl HttpRequest {
    fn is_rpc_call(&self) -> bool {
        self.meta.method == Method::POST
            && self
                .meta
                .headers
                .get(CONTENT_TYPE)
                .is_some_and(|v| v == SETU_CONTENT_TYPE)
    }

    pub fn get_rpc_key(&self) -> Option<u32> {
        if !self.is_rpc_call() {
            return None;
        }

        self.meta
            .headers
            .get("rpc-id")?
            .to_str()
            .ok()?
            .parse::<u32>()
            .ok()
    }
}

impl HttpResponse {
    pub fn add_setu_content_type_header(&mut self) {
        self.headers_mut().insert(
            CONTENT_TYPE,
            const { HeaderValue::from_static(SETU_CONTENT_TYPE) },
        );
    }
}
