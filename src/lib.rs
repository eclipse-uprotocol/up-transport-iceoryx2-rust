use async_trait::async_trait;
use std::sync::Arc;
use up_rust::{UListener, UMessage, UStatus, UTransport, UUri, UCode, UAttributes};
use iceoryx2::prelude::*;

mod custom_header;
mod transmission_data;
pub use custom_header::CustomHeader;
pub use transmission_data::TransmissionData;

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
        
        thread::spawn(move || {
            Self::background_task(rx);
        });
        
        Ok(Self {
            command_sender: tx,
        })
    }
    
    fn background_task(rx: std::sync::mpsc::Receiver<TransportCommand>) {
        let node = match NodeBuilder::new().create::<ipc::Service>() {
            Ok(node) => node,
            Err(e) => {
                eprintln!("Failed to create iceoryx2 node: {}", e);
                return;
            }
        };

        let service = match node
            .service_builder(&"My/Funk/ServiceName".try_into().unwrap())
            .publish_subscribe::<TransmissionData>()
            .user_header::<CustomHeader>()
            .open_or_create()
        {
            Ok(service) => service,
            Err(e) => {
                eprintln!("Failed to create iceoryx2 service: {}", e);
                return;
            }
        };

        let publisher = match service.publisher_builder().create() {
            Ok(publisher) => publisher,
            Err(e) => {
                eprintln!("Failed to create iceoryx2 publisher: {}", e);
                return;
            }
        };
        
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
        let transmission_data = TransmissionData::from_message(&message)?;
        
        let header = CustomHeader::from_message(&message)?;
        
        // Loan sample and write payload
        let sample = publisher.loan_uninit()
            .map_err(|e| UStatus::fail_with_code(UCode::INTERNAL, &format!("Failed to loan sample: {e}")))?;
        
        let sample_final = sample.write_payload(transmission_data);

        sample_final.send()
            .map_err(|e| UStatus::fail_with_code(UCode::INTERNAL, &format!("Failed to send: {e}")))?;
        
        Ok(())
    }
}

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
mod test;