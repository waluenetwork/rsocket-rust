use jni::JNIEnv;
use jni::objects::{JClass, JString, JObject};
use jni::sys::{jlong, jint, jobject};
use rsocket_rust::prelude::*;
use rsocket_rust_transport_tcp::TcpServerTransport;
use tokio::runtime::Runtime;
use super::{JavaRSocketPayload, JavaRSocketError, JavaCallback};
use std::ptr;
use std::net::SocketAddr;

#[repr(C)]
pub struct JavaRSocketServer {
    runtime: Box<Runtime>,
    addr: String,
}

impl JavaRSocketServer {
    pub fn new(addr: String) -> Result<Self, JavaRSocketError> {
        let runtime = Runtime::new()
            .map_err(|e| JavaRSocketError::new(-1, &format!("Failed to create runtime: {}", e)))?;
        
        Ok(Self {
            runtime: Box::new(runtime),
            addr,
        })
    }
}

#[no_mangle]
pub extern "system" fn Java_com_rsocket_rust_RSocketServer_create(
    env: JNIEnv,
    _class: JClass,
    address: JString,
) -> jlong {
    let addr_str = match env.get_string(address) {
        Ok(s) => s.into(),
        Err(_) => return 0,
    };
    
    match JavaRSocketServer::new(addr_str) {
        Ok(server) => Box::into_raw(Box::new(server)) as jlong,
        Err(_) => 0,
    }
}

#[no_mangle]
pub extern "system" fn Java_com_rsocket_rust_RSocketServer_startTcp(
    env: JNIEnv,
    _class: JClass,
    server_ptr: jlong,
    request_handler: JObject,
) -> jint {
    if server_ptr == 0 {
        return -1;
    }
    
    let server = unsafe { &mut *(server_ptr as *mut JavaRSocketServer) };
    let addr = server.addr.clone();
    
    let java_callback = match JavaCallback::new(&env, request_handler) {
        Ok(cb) => cb,
        Err(_) => return -1,
    };
    
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
                    let rsocket = JavaRSocket {
                        callback: java_callback,
                    };
                    Ok(Box::new(rsocket) as Box<dyn RSocket>)
                }))
                .serve()
                .await;
        });
    });
    
    0
}

#[no_mangle]
pub extern "system" fn Java_com_rsocket_rust_RSocketServer_free(
    _env: JNIEnv,
    _class: JClass,
    server_ptr: jlong,
) {
    if server_ptr != 0 {
        unsafe {
            let _ = Box::from_raw(server_ptr as *mut JavaRSocketServer);
        }
    }
}

struct JavaRSocket {
    callback: JavaCallback,
}

unsafe impl Send for JavaRSocket {}
unsafe impl Sync for JavaRSocket {}

#[async_trait::async_trait]
impl RSocket for JavaRSocket {
    async fn request_response(&self, req: Payload) -> Result<Option<Payload>, anyhow::Error> {
        let req_payload = JavaRSocketPayload::new(req);
        self.callback.call_response(Some(&req_payload), None);
        Ok(Some(Payload::builder().set_data_utf8("Echo response").build()))
    }
    
    async fn fire_and_forget(&self, req: Payload) -> Result<(), anyhow::Error> {
        let req_payload = JavaRSocketPayload::new(req);
        self.callback.call_response(Some(&req_payload), None);
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
