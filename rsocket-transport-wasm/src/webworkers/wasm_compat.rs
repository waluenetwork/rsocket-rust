
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use futures_channel::mpsc;
use futures_util::{Sink, Stream};
use wasm_bindgen_futures::spawn_local;

#[derive(Debug)]
pub enum WasmTryRecvError {
    Empty,
    Disconnected,
}

impl std::fmt::Display for WasmTryRecvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WasmTryRecvError::Empty => write!(f, "channel is empty"),
            WasmTryRecvError::Disconnected => write!(f, "channel is disconnected"),
        }
    }
}

impl std::error::Error for WasmTryRecvError {}

pub fn wasm_channel<T>(buffer: usize) -> (WasmSender<T>, WasmReceiver<T>) {
    let (tx, rx) = mpsc::channel(buffer);
    (WasmSender { inner: tx }, WasmReceiver { inner: rx })
}

pub struct WasmSender<T> {
    inner: mpsc::Sender<T>,
}

pub struct WasmReceiver<T> {
    inner: mpsc::Receiver<T>,
}

impl<T> WasmSender<T> {
    pub async fn send(&mut self, item: T) -> Result<(), mpsc::SendError> {
        use futures_util::SinkExt;
        self.inner.send(item).await
    }
    
    pub fn try_send(&mut self, item: T) -> Result<(), mpsc::TrySendError<T>> {
        self.inner.try_send(item)
    }
}

impl<T> WasmReceiver<T> {
    pub async fn recv(&mut self) -> Option<T> {
        use futures_util::StreamExt;
        self.inner.next().await
    }
    
    pub fn try_recv(&mut self) -> Result<T, WasmTryRecvError> {
        match self.inner.try_next() {
            Ok(Some(item)) => Ok(item),
            Ok(None) => Err(WasmTryRecvError::Empty),
            Err(_) => Err(WasmTryRecvError::Disconnected),
        }
    }
}

pub fn wasm_spawn<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    spawn_local(future);
}

pub async fn wasm_timeout<F, T>(duration_ms: u32, future: F) -> Result<T, WasmTimeoutError>
where
    F: Future<Output = T> + std::marker::Unpin,
{
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::JsFuture;
    use js_sys::Promise;
    
    let timeout_promise = Promise::new(&mut |resolve, _reject| {
        let window = web_sys::window().unwrap();
        window.set_timeout_with_callback_and_timeout_and_arguments_0(
            &resolve,
            duration_ms as i32,
        ).unwrap();
    });
    
    let timeout_future = JsFuture::from(timeout_promise);
    
    use futures_util::future::{select, Either};
    use std::pin::Pin;
    
    let future = Box::pin(future);
    let timeout_future = Box::pin(timeout_future);
    
    match select(future, timeout_future).await {
        Either::Left((result, _)) => Ok(result),
        Either::Right(_) => Err(WasmTimeoutError),
    }
}

#[derive(Debug)]
pub struct WasmTimeoutError;

impl std::fmt::Display for WasmTimeoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WASM timeout error")
    }
}

impl std::error::Error for WasmTimeoutError {}

pub struct WasmMutex<T> {
    inner: std::cell::RefCell<T>,
}

impl<T> WasmMutex<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: std::cell::RefCell::new(value),
        }
    }
    
    pub async fn lock(&self) -> WasmMutexGuard<'_, T> {
        WasmMutexGuard {
            guard: self.inner.borrow_mut(),
        }
    }
}

pub struct WasmMutexGuard<'a, T> {
    guard: std::cell::RefMut<'a, T>,
}

impl<'a, T> std::ops::Deref for WasmMutexGuard<'a, T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        &*self.guard
    }
}

impl<'a, T> std::ops::DerefMut for WasmMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.guard
    }
}

pub struct WasmNotify {
    notified: std::cell::Cell<bool>,
}

impl WasmNotify {
    pub fn new() -> Self {
        Self {
            notified: std::cell::Cell::new(false),
        }
    }
    
    pub fn notify_one(&self) {
        self.notified.set(true);
    }
    
    pub async fn notified(&self) {
        while !self.notified.get() {
            wasm_bindgen_futures::JsFuture::from(
                js_sys::Promise::resolve(&wasm_bindgen::JsValue::from(1))
            ).await.ok();
        }
        self.notified.set(false);
    }
}
