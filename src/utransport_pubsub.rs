// // ################################################################################
// // Copyright (c) 2025 Contributors to the Eclipse Foundation
// //
// // See the NOTICE file(s) distributed with this work for additional
// // information regarding copyright ownership.
// //
// // This program and the accompanying materials are made available under the
// // terms of the Apache License Version 2.0 which is available at
// // https: //www.apache.org/licenses/LICENSE-2.0
// //
// // SPDX-License-Identifier: Apache-2.0
// // ################################################################################

use async_trait::async_trait;
use iceoryx2::prelude::{AllocationStrategy, MessagingPattern};
use iceoryx2::sample_mut::SampleMut;
use iceoryx2::{
    node::{Node, NodeBuilder},
    port::{publisher::Publisher, subscriber::Subscriber},
    prelude::ServiceName,
    service::ipc_threadsafe,
};
use iceoryx2_bb_container::vec::FixedSizeVec;
use protobuf::Message;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use up_rust::{ComparableListener, UCode, UListener, UMessage, UStatus, UTransport, UUri};

use crate::UPROTOCOL_MAJOR_VERSION;
use crate::uprotocolheader::MAX_FEASIBLE_UATTRIBUTES_SERIALIZED_LENGTH;
use crate::workers::dispatcher::Iceoryx2WorkerDispatcher;
use crate::{
    ListenerMap, PublisherSet, SubscriberSet, service_name_mapping::compute_service_name,
    uprotocolheader::UProtocolHeader,
};

#[derive(Debug)]
pub struct Iceoryx2PubSub {
    node: Node<ipc_threadsafe::Service>,
    pub publishers: PublisherSet<ipc_threadsafe::Service>,
    pub subscribers: SubscriberSet<ipc_threadsafe::Service>,
    pub listeners: ListenerMap,
}

impl Iceoryx2PubSub {
    pub fn new() -> Arc<Self> {
        let node = NodeBuilder::new()
            .create::<ipc_threadsafe::Service>()
            .expect("Failed to create Iceoryx2 Node");
        let transport = Arc::new(Self {
            node,
            publishers: RwLock::new(HashMap::new()),
            subscribers: RwLock::new(HashMap::new()),
            listeners: RwLock::new(HashMap::new()),
        });
        Iceoryx2WorkerDispatcher::start_listener_worker(transport.clone());
        transport
    }

    pub fn create_subscriber(
        &self,
        service_name: ServiceName,
    ) -> Result<Subscriber<ipc_threadsafe::Service, [u8], UProtocolHeader>, UStatus> {
        let service = self
            .node
            .service_builder(&service_name)
            .publish_subscribe::<[u8]>()
            .user_header::<UProtocolHeader>()
            .open_or_create()
            .map_err(|e| {
                UStatus::fail_with_code(UCode::INTERNAL, format!("Failed to create service: {e}"))
            })?;
        let subscriber = service.subscriber_builder().create().map_err(|e| {
            UStatus::fail_with_code(UCode::INTERNAL, format!("Failed to create subscriber: {e}"))
        })?;
        Ok(subscriber)
    }

    pub async fn get_or_create_publisher(
        &self,
        service_name: ServiceName,
    ) -> Result<Arc<Publisher<ipc_threadsafe::Service, [u8], UProtocolHeader>>, UStatus> {
        let publisher = self.get_publisher(service_name.clone()).await;
        if let Some(publisher) = publisher {
            return Ok(publisher);
        }
        self.create_publisher(service_name).await
    }

    async fn create_publisher(
        &self,
        service_name: ServiceName,
    ) -> Result<Arc<Publisher<ipc_threadsafe::Service, [u8], UProtocolHeader>>, UStatus> {
        let service = self
            .node
            .service_builder(&service_name)
            .publish_subscribe::<[u8]>()
            .user_header::<UProtocolHeader>()
            .open_or_create()
            .map_err(|e| {
                UStatus::fail_with_code(UCode::INTERNAL, format!("Failed to create service: {e}"))
            })?;

        let publisher = service
            .publisher_builder()
            .allocation_strategy(AllocationStrategy::PowerOfTwo)
            .create()
            .map_err(|e| {
                UStatus::fail_with_code(UCode::INTERNAL, format!("Failed to create publisher: {e}"))
            })?;
        let mut publishers = self.publishers.write().await;
        publishers.insert(service_name.clone(), Arc::new(publisher));
        let publisher = publishers.get(&service_name).unwrap();
        Ok(publisher.clone())
    }

    async fn get_publisher(
        &self,
        service_name: ServiceName,
    ) -> Option<Arc<Publisher<ipc_threadsafe::Service, [u8], UProtocolHeader>>> {
        let publishers = self.publishers.read().await;
        if publishers.contains_key(&service_name) {
            let publisher = publishers.get(&service_name).unwrap();
            return Some(publisher.clone());
        }
        None
    }

    pub async fn relay(&self) -> Result<(), UStatus> {
        let subscribers = self.subscribers.read().await;
        for (service_name, subscriber) in subscribers.iter() {
            match subscriber.receive() {
                Ok(Some(sample)) => {
                    let payload = sample.payload();
                    let umessage = UMessage::parse_from_bytes(payload).map_err(|e| {
                        UStatus::fail_with_code(
                            UCode::INTERNAL,
                            format!("Failed to deserialize UMessage: {}", e),
                        )
                    })?;
                    if let Some(listeners_to_notify) = self.listeners.read().await.get(service_name)
                    {
                        for listener in listeners_to_notify.iter() {
                            let listener: &ComparableListener = listener;
                            let payload_clone = umessage.clone();
                            listener.on_receive(payload_clone).await;
                        }
                    }
                }
                Ok(None) => continue, // No sample available
                Err(e) => {
                    return Err(UStatus::fail_with_code(
                        UCode::INTERNAL,
                        format!("Failed to receive sample: {e}"),
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn write_message_to_sample(
        &self,
        publisher: &Publisher<ipc_threadsafe::Service, [u8], UProtocolHeader>,
        message: UMessage,
    ) -> Result<SampleMut<ipc_threadsafe::Service, [u8], UProtocolHeader>, UStatus> {
        let sample_size = message.compute_size();
        let mut sample = publisher
            .loan_slice_uninit(sample_size as usize)
            .map_err(|e| {
                UStatus::fail_with_code(UCode::INTERNAL, format!("Failed to loan sample: {e}"))
            })?;
        let message_bytes = message
            .write_to_bytes()
            .map_err(|e| UStatus::fail_with_code(UCode::INTERNAL, e.to_string()))?;
        let serialized_data = message_bytes.as_slice();
        let user_header: &mut UProtocolHeader = sample.user_header_mut();
        self.set_samples_user_header(user_header, message)?;
        let sample_final = sample.write_from_slice(serialized_data);
        Ok(sample_final)
    }

    fn set_samples_user_header(
        &self,
        user_header: &mut UProtocolHeader,
        message: UMessage,
    ) -> Result<(), UStatus> {
        user_header.uprotocol_major_version = UPROTOCOL_MAJOR_VERSION;
        let serialized_uattributes = message
            .attributes
            .write_to_bytes()
            .map_err(|e| UStatus::fail_with_code(UCode::INTERNAL, e.to_string()))?;
        let mut fixed_sized_vec: FixedSizeVec<u8, MAX_FEASIBLE_UATTRIBUTES_SERIALIZED_LENGTH> =
            FixedSizeVec::new();
        for byte in serialized_uattributes.iter() {
            fixed_sized_vec.push(*byte);
        }
        user_header.uattributes_serialized = fixed_sized_vec;
        Ok(())
    }
}

#[async_trait]
impl UTransport for Iceoryx2PubSub {
    /// ## DISCLAIMER
    ///
    /// This code is a prototype to make UMessage work with iceoryx2's ZeroCopySend system
    ///
    /// UMessage is not ZeroCopySend compatible as-is. If UMessages are sent
    /// directly to an iceoryx2 publisher, it will compile. However, the
    /// subscriber will receive a segmentation fault when receiving theUMessage
    ///
    /// See [ZeroCopySend's safety requirements](https://docs.rs/iceoryx2/latest/iceoryx2/prelude/trait.ZeroCopySend.html#safety) for more details.
    ///
    /// This essentially defeats the purpose of using iceoryx2 and
    /// ZeroCopySend, as it copies the data into a fixed-size array and then out
    /// of the array and back into a UMessage inside the `UTransport.send()` method
    /// and the `UListener.on_receive()` method. The UTransport or UMessage
    /// definition needs to be adjusted for this to truly be a zero-copy transport.
    async fn send(&self, message: UMessage) -> Result<(), UStatus> {
        let service_name = {
            let source_filter = &message.attributes.source;
            let sink_filter = message.attributes.sink.as_ref();
            compute_service_name(
                source_filter,
                sink_filter,
                MessagingPattern::PublishSubscribe,
            )?
        };
        let publisher = self
            .get_or_create_publisher(service_name)
            .await
            .map_err(|e| {
                UStatus::fail_with_code(UCode::INTERNAL, format!("Failed to get publisher: {e}"))
            })?;
        let sample_final = self.write_message_to_sample(publisher.as_ref(), message)?;
        sample_final.send().map_err(|e| {
            UStatus::fail_with_code(UCode::INTERNAL, format!("Failed to send: {e}"))
        })?;
        Ok(())
    }

    async fn register_listener(
        &self,
        source_filter: &UUri,
        sink_filter: Option<&UUri>,
        listener: Arc<dyn UListener>,
    ) -> Result<(), UStatus> {
        up_rust::verify_filter_criteria(source_filter, sink_filter)?;
        let service_name = compute_service_name(
            source_filter,
            sink_filter,
            MessagingPattern::PublishSubscribe,
        )?;
        let has_subscriber = {
            let subscribers = self.subscribers.read().await;
            subscribers.contains_key(&service_name)
        };
        // insert subscriber for service name if it does not already exist
        if !has_subscriber {
            let subscriber = self.create_subscriber(service_name.clone())?;
            let mut subscribers = self.subscribers.write().await;
            subscribers.insert(service_name.clone(), Arc::new(subscriber));
        }
        // insert listener for service name if it does not already exist
        if !self.listeners.read().await.contains_key(&service_name) {
            let mut listeners = self.listeners.write().await;
            listeners
                .entry(service_name)
                .or_default()
                .insert(ComparableListener::new(listener));
        }
        Ok(())
    }

    async fn unregister_listener(
        &self,
        source_filter: &UUri,
        sink_filter: Option<&UUri>,
        listener: Arc<dyn UListener>,
    ) -> Result<(), UStatus> {
        up_rust::verify_filter_criteria(source_filter, sink_filter)?;
        let service_name = compute_service_name(
            source_filter,
            sink_filter,
            MessagingPattern::PublishSubscribe,
        )?;
        let comparable_listener = ComparableListener::new(listener.clone());
        let mut listeners = self.listeners.write().await;
        if let Some(existing_listeners) = listeners.get_mut(&service_name) {
            existing_listeners.retain(|l| !l.eq(&comparable_listener));
            if existing_listeners.is_empty() {
                let mut subscribers = self.subscribers.write().await;
                subscribers.remove(&service_name);
            }
        }
        Ok(())
    }
}
