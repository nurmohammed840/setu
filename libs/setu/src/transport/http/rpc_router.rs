use http::{Method, header::CONTENT_TYPE};

use super::*;

pub async fn process_rpc_request(req: HttpRequest, res: HttpResponse) {
    if let Some(call_id) = req.get_rpc_key() {
        // nio::spawn_local(async move {});
    }
}

impl HttpRequest {
    fn is_rpc_call(&self) -> bool {
        self.meta.method == Method::POST
            && self
                .meta
                .headers
                .get(CONTENT_TYPE)
                .is_some_and(|v| v == "application/setu")
    }

    fn get_rpc_key(&self) -> Option<u32> {
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
