use std::sync::{Arc, Weak};
use crossbeam_channel::{unbounded, Receiver, Sender};
use crossbeam_skiplist::SkipMap;
use crossbeam_deque::{Injector, Stealer, Worker};


use super::misc::{debug_frame, StreamID};
use crate::error::{self, RSocketError};
use crate::frame::{self, Body, Frame};
use crate::payload::{Payload, SetupPayload};
use crate::spi::{Flux, RSocket, ServerResponder};
use crate::Result;
use super::fragmentation::{Joiner, Splitter};
use futures::future::{AbortHandle, Abortable};
use tokio::sync::{oneshot, RwLock};

type HandlerMap = SkipMap<u32, Handler>;
type JoinerMap = SkipMap<u32, Joiner>;
type AbortHandleMap = SkipMap<u32, AbortHandle>;

#[derive(Debug)]
enum Handler {
    ReqRR(oneshot::Sender<Result<Option<Payload>>>),
    ReqRS(crossbeam_channel::Sender<Result<Payload>>),
    ReqRC(crossbeam_channel::Sender<Result<Payload>>),
}

struct CrossbeamDuplexSocketInner {
    seq: StreamID,
    responder: Responder,
    tx: Sender<Frame>,
    handlers: HandlerMap,
    joiners: JoinerMap,
    abort_handles: Arc<AbortHandleMap>,
    frame_queue: Injector<Frame>,
    workers: Vec<Worker<Frame>>,
    stealers: Vec<Stealer<Frame>>,
    splitter: Option<Splitter>,
}

#[derive(Clone)]
struct Responder {
    inner: Arc<RwLock<Box<dyn RSocket>>>,
}

impl Responder {
    fn new() -> Self {
        use crate::utils::EmptyRSocket;
        Self {
            inner: Arc::new(RwLock::new(Box::new(EmptyRSocket))),
        }
    }
    
    fn set(&self, rs: Box<dyn RSocket>) {
        let inner = self.inner.clone();
        tokio::spawn(async move {
            let mut guard = inner.write().await;
            *guard = rs;
        });
    }
}

#[derive(Clone)]
pub struct CrossbeamClientRequester {
    inner: Arc<CrossbeamDuplexSocketInner>,
}

#[derive(Clone)]
pub struct CrossbeamServerRequester {
    inner: Weak<CrossbeamDuplexSocketInner>,
}

pub struct CrossbeamDuplexSocket {
    inner: Arc<CrossbeamDuplexSocketInner>,
}

impl CrossbeamDuplexSocketInner {
    fn new(
        first_stream_id: u32,
        tx: Sender<Frame>,
        num_workers: usize,
    ) -> Self {
        let frame_queue = Injector::new();
        let mut workers = Vec::with_capacity(num_workers);
        let mut stealers = Vec::with_capacity(num_workers);
        
        for _ in 0..num_workers {
            let worker = Worker::new_fifo();
            stealers.push(worker.stealer());
            workers.push(worker);
        }
        
        Self {
            seq: StreamID::from(first_stream_id),
            tx,
            responder: Responder::new(),
            handlers: SkipMap::new(),
            joiners: SkipMap::new(),
            abort_handles: Arc::new(SkipMap::new()),
            frame_queue,
            workers,
            stealers,
            splitter: None,
        }
    }
    
    fn process_frame_lockfree(&self, frame: Frame) {
        if let Some(worker) = self.workers.first() {
            worker.push(frame);
        } else {
            self.frame_queue.push(frame);
        }
    }
    
    fn register_handler_lockfree(&self, sid: u32, handler: Handler) {
        self.handlers.insert(sid, handler);
    }
    
    fn remove_handler_lockfree(&self, sid: u32) -> Option<Handler> {
        self.handlers.remove(&sid).map(|entry| entry.value().clone())
    }
}

impl CrossbeamDuplexSocket {
    pub(crate) fn new(
        first_stream_id: u32,
        tx: Sender<Frame>,
        num_workers: usize,
    ) -> CrossbeamDuplexSocket {
        CrossbeamDuplexSocket {
            inner: Arc::new(CrossbeamDuplexSocketInner::new(first_stream_id, tx, num_workers)),
        }
    }
    
    pub(crate) fn register_handler_lockfree(&self, sid: u32, handler: Handler) {
        self.inner.register_handler_lockfree(sid, handler);
    }
    
    pub(crate) fn remove_handler_lockfree(&self, sid: u32) -> Option<Handler> {
        self.inner.remove_handler_lockfree(sid)
    }
    
    pub(crate) fn client_requester(&self) -> CrossbeamClientRequester {
        CrossbeamClientRequester {
            inner: self.inner.clone(),
        }
    }
    
    pub(crate) fn server_requester(&self) -> CrossbeamServerRequester {
        CrossbeamServerRequester {
            inner: Arc::downgrade(&self.inner),
        }
    }
}

impl Clone for Handler {
    fn clone(&self) -> Self {
        match self {
            Handler::ReqRR(_) => panic!("Cannot clone ReqRR handler"),
            Handler::ReqRS(tx) => Handler::ReqRS(tx.clone()),
            Handler::ReqRC(tx) => Handler::ReqRC(tx.clone()),
        }
    }
}
