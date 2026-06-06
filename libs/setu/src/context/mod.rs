mod store;

use crate::Timeout;
use std::{
    cell::{RefCell, UnsafeCell},
    net::SocketAddr,
    rc::Rc,
};
pub use store::Store;

thread_local! {
    pub static CTX: UnsafeCell<Option<Rc<Context>>> = const { UnsafeCell::new(None) };
}

#[derive(Debug)]
pub struct Context {
    pub timeout: Option<Timeout>,
    pub(crate) state: Rc<State>,

    pub(crate) http_headers: http::HeaderMap<http::HeaderValue>,
}

impl Context {
    pub fn as_mut(&self) -> std::cell::RefMut<'_, Store> {
        self.state.state.borrow_mut()
    }

    /// this field only available for HTTP transport.
    pub fn http_headers(&self) -> &http::HeaderMap<http::HeaderValue> {
        &self.http_headers
    }

    pub fn addr(&self) -> &SocketAddr {
        &self.state.addr
    }
}

impl Context {
    pub(crate) fn swap(this: &mut Option<Rc<Self>>) {
        CTX.with(|cell| unsafe {
            std::mem::swap(&mut (*cell.get()), this);
        });
    }

    pub fn get() -> Rc<Context> {
        Context::with(|c| c.clone())
    }

    pub fn with<F, R>(f: F) -> R
    where
        F: FnOnce(&Rc<Context>) -> R,
    {
        CTX.with(|cell| unsafe { f((*cell.get()).as_ref().unwrap()) })
    }
}

#[derive(Debug)]
pub struct State {
    pub addr: SocketAddr,
    pub state: RefCell<Store>,
}

impl State {
    pub fn new(addr: SocketAddr) -> Rc<Self> {
        Rc::new(State {
            addr,
            state: RefCell::new(Store::new()),
        })
    }
}
