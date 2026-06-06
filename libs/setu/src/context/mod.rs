mod store;

use crate::Timeout;
use std::{
    cell::{RefCell, UnsafeCell},
    net::SocketAddr,
    rc::Rc,
};
pub use store::Store;

thread_local! {
    pub static CTX: UnsafeCell<Option<Context>> = const { UnsafeCell::new(None) };
}

#[derive(Debug)]
pub struct Context {
    pub timeout: Option<Timeout>,
    pub state: Rc<State>,
    pub http_headers: Option<Rc<http::HeaderMap<http::HeaderValue>>>,
}

impl Context {
    pub fn as_mut(&self) -> std::cell::RefMut<'_, Store> {
        self.state.state.borrow_mut()
    }
    pub fn http_headers() -> Option<Rc<http::HeaderMap<http::HeaderValue>>> {
        Context::get(|c| c.http_headers.as_ref().map(Rc::clone))
    }
    pub fn addr() -> SocketAddr {
        Context::get(|c| c.state.addr.clone())
    }
}

impl Context {
    pub(crate) fn swap(this: &mut Option<Self>) {
        CTX.with(|cell| unsafe {
            std::mem::swap(&mut (*cell.get()), this);
        });
    }

    pub fn get<F, R>(f: F) -> R
    where
        F: FnOnce(&Context) -> R,
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
