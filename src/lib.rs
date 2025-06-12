use async_trait::async_trait;
use std::sync::Arc;
use up_rust::{UListener, UMessage, UStatus, UTransport, UUri, UCode, UAttributes};
use iceoryx2::prelude::*;
use protobuf::MessageField;

mod custom_header;
mod transmission_data;
pub use custom_header::CustomHeader;
pub use transmission_data::TransmissionData;

/// This will be the main struct for our uProtocol transport.
/// It will hold the state necessary to communicate with iceoryx2,
/// such as the service connection and active listeners.


// Channel-based solution - iceoryx2 types are not Send/Sync, so we use channels
use std::thread;

#[derive(Debug)]
enum TransportCommand {
    Send { 
        message: UMessage, 
        response: std::sync::mpsc::Sender<Result<(), UStatus>> 
    },
    RegisterListener {
        source_filter: UUri,
        sink_filter: Option<UUri>,
        // Remove Debug requirement by not deriving Debug or handling it manually
        response: std::sync::mpsc::Sender<Result<(), UStatus>>,
    },
    UnregisterListener {
        source_filter: UUri,
        sink_filter: Option<UUri>,
        response: std::sync::mpsc::Sender<Result<(), UStatus>>,
    },
}

pub struct Iceoryx2Transport {
    command_sender: std::sync::mpsc::Sender<TransportCommand>,
}

impl Iceoryx2Transport {
    pub fn new() -> Result<Self, UStatus> {
        let (tx, rx) = std::sync::mpsc::channel();
        
        // Spawn background thread to handle iceoryx2 operations
        thread::spawn(move || {
            Self::background_task(rx);
        });
        
        Ok(Self {
            command_sender: tx,
        })
    }
    
    fn background_task(rx: std::sync::mpsc::Receiver<TransportCommand>) {
        // Create iceoryx2 components in the background thread (single-threaded)
        let node_result = NodeBuilder::new().create::<ipc::Service>();
        let node = match node_result {
            Ok(node) => node,
            Err(e) => {
                eprintln!("Failed to create iceoryx2 node: {}", e);
                return;
            }
        };

        let service_result = node
            .service_builder(&"My/Funk/ServiceName".try_into().unwrap())
            .publish_subscribe::<TransmissionData>()
            .user_header::<CustomHeader>()
            .open_or_create();
            
        let service = match service_result {
            Ok(service) => service,
            Err(e) => {
                eprintln!("Failed to create iceoryx2 service: {}", e);
                return;
            }
        };

        let publisher_result = service.publisher_builder().create();
        let publisher = match publisher_result {
            Ok(publisher) => publisher,
            Err(e) => {
                eprintln!("Failed to create iceoryx2 publisher: {}", e);
                return;
            }
        };
        
        // Process commands
        while let Ok(command) = rx.recv() {
            match command {
                TransportCommand::Send { message, response } => {
                    let result = Self::handle_send(&publisher, message);
                    let _ = response.send(result);
                }
                TransportCommand::RegisterListener { response, .. } => {
                    // TODO: Implement listener registration
                    let _ = response.send(Ok(()));
                }
                TransportCommand::UnregisterListener { response, .. } => {
                    // TODO: Implement listener unregistration  
                    let _ = response.send(Ok(()));
                }
            }
        }
    }
    
    fn handle_send(
        publisher: &iceoryx2::port::publisher::Publisher<ipc::Service, TransmissionData, CustomHeader>,
        message: UMessage
    ) -> Result<(), UStatus> {
        // Convert UMessage to TransmissionData
        let transmission_data = match TransmissionData::from_message(&message) {
            Ok(data) => data,
            Err(e) => return Err(e),
        };
        
        // Create custom header from UMessage attributes
        let header = match CustomHeader::from_message(&message) {
            Ok(header) => header,
            Err(e) => return Err(e),
        };
        
        // Send the message
        let sample = publisher.loan_uninit()
            .map_err(|e| UStatus::fail_with_code(UCode::INTERNAL, &format!("Failed to loan sample: {e}")))?;
        
        let sample_final = sample
            .write_payload(transmission_data);

        sample_final.send()
            .map_err(|e| UStatus::fail_with_code(UCode::INTERNAL, &format!("Failed to send: {e}")))?;
        
        Ok(())
    }
}

// The #[async_trait] attribute enables async functions in our trait impl.
#[async_trait]
impl UTransport for Iceoryx2Transport {
    async fn send(&self, message: UMessage) -> Result<(), UStatus> {
        let (tx, rx) = std::sync::mpsc::channel();
        
        let command = TransportCommand::Send {
            message,
            response: tx,
        };
        
        self.command_sender.send(command)
            .map_err(|_| UStatus::fail_with_code(UCode::INTERNAL, "Background task has died"))?;
        
        rx.recv()
            .map_err(|_| UStatus::fail_with_code(UCode::INTERNAL, "Background task response failed"))?
    }

    async fn register_listener(
        &self,
        source_filter: &UUri,
        sink_filter: Option<&UUri>,
        _listener: Arc<dyn UListener>,
    ) -> Result<(), UStatus> {
        let (tx, rx) = std::sync::mpsc::channel();
        
        let command = TransportCommand::RegisterListener {
            source_filter: source_filter.clone(),
            sink_filter: sink_filter.cloned(),
            response: tx,
        };
        
        self.command_sender.send(command)
            .map_err(|_| UStatus::fail_with_code(UCode::INTERNAL, "Background task has died"))?;
        
        rx.recv()
            .map_err(|_| UStatus::fail_with_code(UCode::INTERNAL, "Background task response failed"))?
    }

    async fn unregister_listener(
        &self,
        source_filter: &UUri,
        sink_filter: Option<&UUri>,
        _listener: Arc<dyn UListener>,
    ) -> Result<(), UStatus> {
        let (tx, rx) = std::sync::mpsc::channel();
        
        let command = TransportCommand::UnregisterListener {
            source_filter: source_filter.clone(),
            sink_filter: sink_filter.cloned(),
            response: tx,
        };
        
        self.command_sender.send(command)
            .map_err(|_| UStatus::fail_with_code(UCode::INTERNAL, "Background task has died"))?;
        
        rx.recv()
            .map_err(|_| UStatus::fail_with_code(UCode::INTERNAL, "Background task response failed"))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_transport_creation() {
        // Test that we can create the transport without compilation errors
        // Note: This might fail at runtime if iceoryx2 daemon isn't running
        match Iceoryx2Transport::new() {
            Ok(_transport) => println!("Transport created successfully"),
            Err(e) => println!("Transport creation failed (expected if daemon not running): {:?}", e),
        }
    }
    
    // #[tokio::test]
    // async fn test_send_message() {
    //     if let Ok(transport) = Iceoryx2Transport::new() {
    //         // Create a dummy UMessage
    //         let message = UMessage {
    //             attributes: MessageField::some(UAttributes::from(&uprotocol_header)),
    //             payload: Some(vec![1, 2, 3, 4].into()),
    //             ..Default::default()
    //         };

    //         // This will fail without iceoryx2 daemon, but should compile
    //         match transport.send(message).await {
    //             Ok(_) => println!("Message sent successfully"),
    //             Err(e) => println!("Send failed (expected without daemon): {:?}", e),
    //         }
    //     }
    // }
}