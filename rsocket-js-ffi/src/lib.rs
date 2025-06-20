use neon::prelude::*;
use rsocket_rust::prelude::*;
use rsocket_rust_transport_tcp::TcpClientTransport;
use rsocket_rust_transport_websocket::WebsocketClientTransport;
use std::sync::Arc;
use tokio::runtime::Runtime;

pub struct JsRSocketClient {
    inner: Arc<Box<dyn RSocket>>,
    runtime: Arc<Runtime>,
}

impl Finalize for JsRSocketClient {}

impl JsRSocketClient {
    fn js_new(mut cx: FunctionContext) -> JsResult<JsBox<JsRSocketClient>> {
        let transport_config = cx.argument::<JsObject>(0)?;
        let transport_type = transport_config
            .get(&mut cx, "type")?
            .downcast::<JsString, _>(&mut cx)
            .or_throw(&mut cx)?
            .value(&mut cx);
        
        let address = transport_config
            .get(&mut cx, "address")?
            .downcast::<JsString, _>(&mut cx)
            .or_throw(&mut cx)?
            .value(&mut cx);
        
        let runtime = Arc::new(Runtime::new().unwrap());
        
        let client = runtime.block_on(async {
            match transport_type.as_str() {
                "tcp" => {
                    let transport = TcpClientTransport::from(address);
                    RSocketFactory::connect()
                        .transport(transport)
                        .start()
                        .await
                }
                "websocket" => {
                    let transport = WebsocketClientTransport::from(address.as_str());
                    RSocketFactory::connect()
                        .transport(transport)
                        .start()
                        .await
                }
                _ => return Err(RSocketError::Other("Unsupported transport type".into())),
            }
        });
        
        match client {
            Ok(client) => Ok(cx.boxed(JsRSocketClient {
                inner: Arc::new(client),
                runtime,
            })),
            Err(_) => cx.throw_error("Failed to create RSocket client"),
        }
    }
    
    fn js_request_response(mut cx: FunctionContext) -> JsResult<JsPromise> {
        let client = cx.argument::<JsBox<JsRSocketClient>>(0)?;
        let payload_obj = cx.argument::<JsObject>(1)?;
        
        let data = payload_obj
            .get(&mut cx, "data")?
            .downcast::<JsString, _>(&mut cx)
            .or_throw(&mut cx)?
            .value(&mut cx);
        
        let (deferred, promise) = cx.promise();
        let runtime = client.runtime.clone();
        let inner = client.inner.clone();
        
        runtime.spawn(async move {
            let payload = Payload::builder()
                .set_data_utf8(&data)
                .build();
            
            match inner.request_response(payload).await {
                Ok(response) => {
                    deferred.settle_with(&cx.channel(), move |mut cx| {
                        let obj = cx.empty_object();
                        let data = cx.string(response.data_utf8().unwrap_or_default());
                        obj.set(&mut cx, "data", data)?;
                        Ok(obj)
                    });
                }
                Err(e) => {
                    deferred.settle_with(&cx.channel(), move |mut cx| {
                        cx.throw_error(format!("Request failed: {}", e))
                    });
                }
            }
        });
        
        Ok(promise)
    }
    
    fn js_fire_and_forget(mut cx: FunctionContext) -> JsResult<JsPromise> {
        let client = cx.argument::<JsBox<JsRSocketClient>>(0)?;
        let payload_obj = cx.argument::<JsObject>(1)?;
        
        let data = payload_obj
            .get(&mut cx, "data")?
            .downcast::<JsString, _>(&mut cx)
            .or_throw(&mut cx)?
            .value(&mut cx);
        
        let (deferred, promise) = cx.promise();
        let runtime = client.runtime.clone();
        let inner = client.inner.clone();
        
        runtime.spawn(async move {
            let payload = Payload::builder()
                .set_data_utf8(&data)
                .build();
            
            match inner.fire_and_forget(payload).await {
                Ok(_) => {
                    deferred.settle_with(&cx.channel(), move |mut cx| {
                        Ok(cx.undefined())
                    });
                }
                Err(e) => {
                    deferred.settle_with(&cx.channel(), move |mut cx| {
                        cx.throw_error(format!("Fire and forget failed: {}", e))
                    });
                }
            }
        });
        
        Ok(promise)
    }
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("createClient", JsRSocketClient::js_new)?;
    cx.export_function("requestResponse", JsRSocketClient::js_request_response)?;
    cx.export_function("fireAndForget", JsRSocketClient::js_fire_and_forget)?;
    Ok(())
}
