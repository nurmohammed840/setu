pub use setu_message;
pub use type_id;

use crate::transport::http::HttpResponse;

pub fn unknown_rpc(id: u32, mut res: HttpResponse) {
    *res.status_mut() = http::StatusCode::NOT_IMPLEMENTED;
    let _ = res.write_unbound(format!("Unknown call id {id}"));
}
