use std::sync::Arc;
use std::pin::Pin;
use futures::future::{select_all, BoxFuture};
use tokio::task::JoinHandle;

use crate::error::RSocketError;
use crate::spi::ServerResponder;
use crate::transport::{ServerTransport, Transport};
use crate::Result;

pub struct MultiTransportServerBuilder {
    transports: Vec<Box<dyn MultiTransportItem>>,
    acceptor: Option<ServerResponder>,
    start_handler: Option<Box<dyn FnMut() + Send + Sync>>,
    mtu: usize,
}

trait MultiTransportItem: Send + Sync {
    fn start(&mut self) -> BoxFuture<'_, Result<()>>;
    fn spawn_listener(&mut self, acceptor: Arc<Option<ServerResponder>>, mtu: usize) -> JoinHandle<Result<()>>;
    fn name(&self) -> &str;
}

struct TransportWrapper<T, C> 
where
    T: ServerTransport<Item = C> + Send + Sync + 'static,
    C: Transport + Send + Sync + 'static,
{
    name: String,
    transport: Option<T>,
    _phantom: std::marker::PhantomData<C>,
}

impl<T, C> TransportWrapper<T, C>
where
    T: ServerTransport<Item = C> + Send + Sync + 'static,
    C: Transport + Send + Sync + 'static,
{
    fn new(name: String, transport: T) -> Self {
        Self {
            name,
            transport: Some(transport),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, C> MultiTransportItem for TransportWrapper<T, C>
where
    T: ServerTransport<Item = C> + Send + Sync + 'static,
    C: Transport + Send + Sync + 'static,
{
    fn start(&mut self) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move {
            if let Some(ref mut transport) = self.transport {
                transport.start().await
            } else {
                Err(RSocketError::Other(anyhow::anyhow!("Transport already started")).into())
            }
        })
    }

    fn spawn_listener(&mut self, acceptor: Arc<Option<ServerResponder>>, mtu: usize) -> JoinHandle<Result<()>> {
        let mut transport = self.transport.take().expect("Transport not available");
        let name = self.name.clone();
        
        tokio::spawn(async move {
            log::info!("Starting {} transport listener", name);
            
            while let Some(next) = transport.next().await {
                match next {
                    Ok(tp) => {
                        let acceptor = acceptor.clone();
                        let transport_name = name.clone();
                        crate::runtime::spawn(async move {
                            log::debug!("New connection on {} transport", transport_name);
                            if let Err(e) = crate::core::server::ServerBuilder::<T, C>::on_transport(mtu, tp, acceptor).await {
                                log::error!("Handle {} transport failed: {}", transport_name, e);
                            }
                        });
                    }
                    Err(e) => {
                        log::error!("Accept next {} transport failed: {}", name, e);
                    }
                }
            }
            
            log::info!("{} transport listener stopped", name);
            Ok(())
        })
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl MultiTransportServerBuilder {
    pub fn new() -> Self {
        Self {
            transports: Vec::new(),
            acceptor: None,
            start_handler: None,
            mtu: 0,
        }
    }

    pub fn add_transport<T, C>(mut self, name: String, transport: T) -> Self
    where
        T: ServerTransport<Item = C> + Send + Sync + 'static,
        C: Transport + Send + Sync + 'static,
    {
        let wrapper = TransportWrapper::new(name, transport);
        self.transports.push(Box::new(wrapper));
        self
    }

    pub fn acceptor(mut self, handler: ServerResponder) -> Self {
        self.acceptor = Some(handler);
        self
    }

    pub fn fragment(mut self, mtu: usize) -> Self {
        if mtu > 0 && mtu < crate::transport::MIN_MTU {
            panic!("invalid fragment mtu: at least {}!", crate::transport::MIN_MTU)
        }
        self.mtu = mtu;
        self
    }

    pub fn on_start(mut self, handler: Box<dyn FnMut() + Send + Sync>) -> Self {
        self.start_handler = Some(handler);
        self
    }

    pub async fn serve(mut self) -> Result<()> {
        if self.transports.is_empty() {
            return Err(RSocketError::Other(anyhow::anyhow!("No transports configured")).into());
        }

        for transport in &mut self.transports {
            transport.start().await?;
            log::info!("Started {} transport", transport.name());
        }

        if let Some(mut invoke) = self.start_handler {
            invoke();
        }

        let acceptor = Arc::new(self.acceptor);
        let mtu = self.mtu;

        let mut handles = Vec::new();
        for mut transport in self.transports {
            let handle = transport.spawn_listener(acceptor.clone(), mtu);
            handles.push(handle);
        }

        log::info!("Multi-transport server started with {} transports", handles.len());

        let (result, _index, _remaining) = select_all(handles).await;
        
        match result {
            Ok(transport_result) => transport_result,
            Err(e) => Err(RSocketError::Other(anyhow::anyhow!("Transport task failed: {}", e)).into()),
        }
    }
}

impl Default for MultiTransportServerBuilder {
    fn default() -> Self {
        Self::new()
    }
}
