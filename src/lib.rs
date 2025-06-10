use async_trait::async_trait;
use std::sync::Arc;
use up_rust::{UListener, UMessage, UStatus, UTransport, UUri, UCode};
use iceoryx2::prelude::*;
use std::sync::Mutex;


mod custom_header;
mod transmission_data;
pub use custom_header::CustomHeader;
pub use transmission_data::TransmissionData;

/// This will be the main struct for our uProtocol transport.
/// It will hold the state necessary to communicate with iceoryx2,
/// such as the service connection and active listeners.
pub struct Iceoryx2Transport {
    node: Node<ipc::Service>,
}

impl Iceoryx2Transport {
    pub fn new() -> Result<Self, UStatus> {
        let node = NodeBuilder::new().create::<ipc::Service>()
            .map_err(|e| UStatus::fail_with_code(UCode::INTERNAL, &format!("Node error: {e}")))?;

        let service = node
            .service_builder(&"My/Funk/ServiceName".try_into().unwrap())
            .publish_subscribe::<TransmissionData>()
            .user_header::<CustomHeader>()
            .open_or_create()
            .map_err(|e| UStatus::fail_with_code(UCode::INTERNAL, &format!("Service error: {e}")))?;

        let publisher = service.publisher_builder().create()
            .map_err(|e| UStatus::fail_with_code(UCode::INTERNAL, &format!("Publisher error: {e}")))?;

        Ok(Self {
            node,
        })
    }
}

// The #[async_trait] attribute enables async functions in our trait impl.
#[async_trait]
impl UTransport for Iceoryx2Transport {
    async fn send(&self, _message: UMessage) -> Result<(), UStatus> {
        todo!();
    }

    async fn register_listener(
        &self,
        _source_filter: &UUri,
        _sink_filter: Option<&UUri>,
        _listener: Arc<dyn UListener>,
    ) -> Result<(), UStatus> {
        todo!()
    }

    async fn unregister_listener(
        &self,
        _source_filter: &UUri,
        _sink_filter: Option<&UUri>,
        _listener: Arc<dyn UListener>,
    ) -> Result<(), UStatus> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}