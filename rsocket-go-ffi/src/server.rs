use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
use std::net::SocketAddr;
use rsocket_rust::prelude::*;
use rsocket_rust_transport_tcp::TcpServerTransport;
use tokio::runtime::Runtime;
use super::{GoRSocketPayload, GoCallback};

#[repr(C)]
pub struct GoRSocketServer {
    runtime: Box<Runtime>,
    addr: String,
}

#[no_mangle]
pub extern "C" fn rsocket_go_server_create(addr: *const c_char) -> *mut GoRSocketServer {
    if addr.is_null() {
        return ptr::null_mut();
    }
    
    let addr_str = unsafe {
        match CStr::from_ptr(addr).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return ptr::null_mut(),
        }
    };
    
    let runtime = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return ptr::null_mut(),
    };
    
    Box::into_raw(Box::new(GoRSocketServer {
        runtime: Box::new(runtime),
        addr: addr_str,
    }))
}

#[no_mangle]
pub extern "C" fn rsocket_go_server_start_tcp(
    server: *mut GoRSocketServer,
    request_handler: GoCallback,
    user_data: *mut c_void,
) -> c_int {
    if server.is_null() {
        return -1;
    }
    
    unsafe {
        let server_ref = &mut *server;
        let addr = server_ref.addr.clone();
        
        let user_data_usize = user_data as usize;
        let callback_fn = request_handler;
        
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                let socket_addr: SocketAddr = match addr.parse() {
                    Ok(addr) => addr,
                    Err(_) => return,
                };
                
                let _result = RSocketFactory::receive()
                    .transport(TcpServerTransport::from(socket_addr))
                    .acceptor(Box::new(move |_setup, _socket| {
                        let rsocket = GoRSocket {
                            callback: callback_fn,
                            user_data: user_data_usize as *mut c_void,
                        };
                        Ok(Box::new(rsocket) as Box<dyn RSocket>)
                    }))
                    .serve()
                    .await;
            });
        });
        
        0
    }
}

#[no_mangle]
pub extern "C" fn rsocket_go_server_free(server: *mut GoRSocketServer) {
    if !server.is_null() {
        unsafe {
            let _ = Box::from_raw(server);
        }
    }
}

struct GoRequestHandler {
    callback: GoCallback,
    user_data: *mut c_void,
}

unsafe impl Send for GoRequestHandler {}
unsafe impl Sync for GoRequestHandler {}

impl GoRequestHandler {
    fn new(callback: GoCallback, user_data: *mut c_void) -> Self {
        Self { callback, user_data }
    }
}

struct GoRSocket {
    callback: GoCallback,
    user_data: *mut c_void,
}

unsafe impl Send for GoRSocket {}
unsafe impl Sync for GoRSocket {}

#[async_trait::async_trait]
impl RSocket for GoRSocket {
    async fn request_response(&self, req: Payload) -> Result<Option<Payload>, anyhow::Error> {
        let req_payload = Box::into_raw(Box::new(GoRSocketPayload {
            inner: Box::new(req),
        }));
        
        (self.callback)(req_payload as *mut c_void, ptr::null_mut(), self.user_data);
        
        Ok(Some(Payload::builder().set_data_utf8("Echo response").build()))
    }
    
    async fn fire_and_forget(&self, req: Payload) -> Result<(), anyhow::Error> {
        let req_payload = Box::into_raw(Box::new(GoRSocketPayload {
            inner: Box::new(req),
        }));
        
        (self.callback)(req_payload as *mut c_void, ptr::null_mut(), self.user_data);
        
        Ok(())
    }
    
    async fn metadata_push(&self, _req: Payload) -> Result<(), anyhow::Error> {
        Ok(())
    }
    
    fn request_stream(&self, _req: Payload) -> futures::stream::BoxStream<'static, Result<Payload, anyhow::Error>> {
        Box::pin(futures::stream::empty())
    }
    
    fn request_channel(&self, _reqs: futures::stream::BoxStream<'static, Result<Payload, anyhow::Error>>) -> futures::stream::BoxStream<'static, Result<Payload, anyhow::Error>> {
        Box::pin(futures::stream::empty())
    }
}
