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
use iceoryx2::prelude::MessagingPattern;
use iceoryx2::{
    node::{Node, NodeBuilder},
    port::{publisher::Publisher, subscriber::Subscriber},
    prelude::ServiceName,
    service::ipc_threadsafe,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use up_rust::{ComparableListener, UCode, UListener, UMessage, UStatus, UTransport, UUri};

use crate::workers::dispatcher::Iceoryx2WorkerDispatcher;
use crate::{
    ListenerMap, PublisherSet, SubscriberSet, service_name_mapping::compute_service_name,
    umessage::UMessageZeroCopy, uprotocolheader::UProtocolHeader,
};

pub struct Iceoryx2PubSub {
    node: Node<ipc_threadsafe::Service>,
    listener_worker_runtime: tokio::runtime::Runtime,
    pub publishers: PublisherSet<ipc_threadsafe::Service>,
    pub subscribers: SubscriberSet<ipc_threadsafe::Service>,
    pub listeners: ListenerMap,
}

impl Iceoryx2PubSub {
    pub fn new() -> Arc<Self> {
        let node = NodeBuilder::new()
            .create::<ipc_threadsafe::Service>()
            .expect("Failed to create Iceoryx2 Node");
        let listener_worker_runtime =
            tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        let transport = Arc::new(Self {
            node,
            publishers: RwLock::new(HashMap::new()),
            subscribers: RwLock::new(HashMap::new()),
            listeners: RwLock::new(HashMap::new()),
            listener_worker_runtime,
        });
        Iceoryx2WorkerDispatcher::start_listener_worker(
            &transport.listener_worker_runtime,
            transport.clone(),
        );
        transport
    }

    pub fn create_subscriber(
        &self,
    ) -> Result<Subscriber<ipc_threadsafe::Service, UMessageZeroCopy, UProtocolHeader>, UStatus>
    {
        // Placeholder implementation
        let service_name: ServiceName = "example_service".try_into().unwrap();
        let service = self
            .node
            .service_builder(&service_name)
            .publish_subscribe::<UMessageZeroCopy>()
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
    ) -> Result<Arc<Publisher<ipc_threadsafe::Service, UMessageZeroCopy, UProtocolHeader>>, UStatus>
    {
        let publishers = self.publishers.read().await;
        if publishers.contains_key(&service_name) {
            let publisher = publishers.get(&service_name).unwrap();
            return Ok(publisher.clone());
        }
        let service_name_res: Result<ServiceName, _> = service_name.as_str().try_into();
        let service = self
            .node
            .service_builder(&service_name_res.unwrap())
            .publish_subscribe::<UMessageZeroCopy>()
            .user_header::<UProtocolHeader>()
            .open_or_create()
            .map_err(|e| {
                UStatus::fail_with_code(UCode::INTERNAL, format!("Failed to create service: {e}"))
            })?;

        let publisher = service.publisher_builder().create().map_err(|e| {
            UStatus::fail_with_code(UCode::INTERNAL, format!("Failed to create publisher: {e}"))
        })?;
        let mut publishers = self.publishers.write().await;
        publishers.insert(service_name.clone(), Arc::new(publisher));
        let publisher = publishers.get(&service_name).unwrap();
        Ok(publisher.clone())
    }

    pub async fn relay(&self) -> Result<(), UStatus> {
        let current_thread_runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| {
                UStatus::fail_with_code(
                    UCode::INTERNAL,
                    format!("Failed to build current_thread runtime: {e}"),
                )
            })?;
        for (service_name, subscriber) in self.subscribers.read().await.iter() {
            match subscriber.receive() {
                Ok(Some(sample)) => {
                    let payload = sample;
                    if let Some(listeners_to_notify) = self.listeners.read().await.get(service_name)
                    {
                        for listener in listeners_to_notify.iter() {
                            let listener: &ComparableListener = listener;
                            current_thread_runtime.block_on(listener.on_receive(payload.0.clone()));
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
}

#[async_trait]
impl UTransport for Iceoryx2PubSub {
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

        let sample = publisher.loan_uninit().map_err(|e| {
            UStatus::fail_with_code(UCode::INTERNAL, format!("Failed to loan sample: {e}"))
        })?;
        let sample_final = sample.write_payload(UMessageZeroCopy(message));
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
            &source_filter,
            sink_filter,
            MessagingPattern::PublishSubscribe,
        )?;
        let subscribers = self.subscribers.read().await;
        // insert subscriber for service name if it does not already exist
        if !subscribers.contains_key(&service_name) {
            let subscriber = self.create_subscriber()?;
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
            &source_filter,
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
