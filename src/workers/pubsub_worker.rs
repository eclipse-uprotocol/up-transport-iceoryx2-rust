// ################################################################################
// Copyright (c) 2025 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache License Version 2.0 which is available at
// https: //www.apache.org/licenses/LICENSE-2.0
//
// SPDX-License-Identifier: Apache-2.0
// ################################################################################

use iceoryx2::{
    port::{publisher::Publisher, subscriber::Subscriber},
    prelude::ServiceName,
    service::ipc,
};
use std::{
    collections::HashMap,
    sync::{Arc, atomic::AtomicBool},
};
use tokio::sync::{Mutex, mpsc::Receiver};
use up_rust::{ComparableListener, UCode, UStatus, UUri};

use crate::{
    service_name_mapping::ServiceNameMapper,
    transport::UTransportIceoryx2,
    types::{ListenerMap, PublisherSet, SubscriberSet},
    umessage::UMessageZeroCopy,
    uprotocolheader::UProtocolHeader,
    workers::{command::WorkerCommand, worker::Iceoryx2Worker},
};

pub struct Iceoryx2PubSubWorker {
    command_receiver: Arc<Mutex<Receiver<WorkerCommand>>>,
    keep_alive: Arc<AtomicBool>,
    transport: UTransportIceoryx2<ipc::Service>,
    messaging_pattern: iceoryx2::prelude::MessagingPattern,
    publishers: PublisherSet,
    subscribers: SubscriberSet,
    listeners: ListenerMap,
}

impl Iceoryx2PubSubWorker {
    pub fn new(
        rx: Arc<Mutex<Receiver<WorkerCommand>>>,
        transport: UTransportIceoryx2<ipc::Service>,
        messaging_pattern: iceoryx2::prelude::MessagingPattern,
    ) -> Self {
        Self {
            transport,
            command_receiver: rx,
            keep_alive: Arc::new(AtomicBool::new(true)),
            messaging_pattern,
            publishers: HashMap::new(),
            subscribers: HashMap::new(),
            listeners: HashMap::new(),
        }
    }
}

impl Iceoryx2PubSubWorker {
    fn create_subscriber(
        &self,
    ) -> Result<Subscriber<ipc::Service, UMessageZeroCopy, UProtocolHeader>, UStatus> {
        // Placeholder implementation
        let service_name: ServiceName = "example_service".try_into().unwrap();
        let service = self
            .transport
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

    fn create_publisher(
        &mut self,
        service_name: ServiceName,
    ) -> Result<&Publisher<ipc::Service, UMessageZeroCopy, UProtocolHeader>, UStatus> {
        if self.publishers.contains_key(&service_name) {
            return Ok(self.publishers.get(&service_name).unwrap());
        }
        let service_name_res: Result<ServiceName, _> = service_name.as_str().try_into();
        let service = self
            .transport
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
        self.publishers.insert(service_name.clone(), publisher);
        Ok(self.publishers.get(&service_name).unwrap())
    }
}

impl Iceoryx2Worker for Iceoryx2PubSubWorker {
    fn send(&mut self, message: up_rust::UMessage) -> Result<(), UStatus> {
        let service_name = {
            let source_filter = &message.attributes.source;
            let sink_filter = message.attributes.sink.as_ref();
            ServiceNameMapper::compute_service_name(
                source_filter,
                sink_filter,
                self.messaging_pattern,
            )?
        };
        let publisher = self.create_publisher(service_name)?;
        let message = UMessageZeroCopy(message);
        let sample = publisher.loan_uninit().map_err(|e| {
            UStatus::fail_with_code(UCode::INTERNAL, format!("Failed to loan sample: {e}"))
        })?;
        let sample_final = sample.write_payload(message);
        sample_final.send().map_err(|e| {
            UStatus::fail_with_code(UCode::INTERNAL, format!("Failed to send: {e}"))
        })?;
        Ok(())
    }

    fn register_listener(
        &mut self,
        source_filter: UUri,
        sink_filter: Option<UUri>,
        listener: Arc<dyn up_rust::UListener>,
    ) -> Result<(), up_rust::UStatus> {
        let service_name = ServiceNameMapper::compute_service_name(
            &source_filter,
            sink_filter.as_ref(),
            self.messaging_pattern,
        )?;
        if !self.subscribers.contains_key(&service_name) {
            let subscriber = self.create_subscriber()?;
            self.subscribers.insert(service_name.clone(), subscriber);
        }
        self.listeners
            .entry(service_name)
            .or_default()
            .insert(ComparableListener::new(listener));
        Ok(())
    }

    fn unregister_listener(
        &mut self,
        source_filter: UUri,
        sink_filter: Option<UUri>,
        listener: Arc<dyn up_rust::UListener>,
    ) -> Result<(), up_rust::UStatus> {
        let service_name = ServiceNameMapper::compute_service_name(
            &source_filter,
            sink_filter.as_ref(),
            self.messaging_pattern,
        )?;
        let comparable_listener = ComparableListener::new(listener.clone());
        if let Some(existing_listeners) = self.listeners.get_mut(&service_name) {
            existing_listeners.retain(|l| !l.eq(&comparable_listener));

            if existing_listeners.is_empty() {
                self.listeners.remove(&service_name);
                self.subscribers.remove(&service_name);
            }
        }
        Ok(())
    }

    fn get_command_receiver(&self) -> Arc<Mutex<Receiver<WorkerCommand>>> {
        self.command_receiver.clone()
    }

    fn keep_alive(&self) -> &std::sync::atomic::AtomicBool {
        &self.keep_alive
    }

    async fn receive_and_notify_listeners(&self) -> Result<(), UStatus> {
        let current_thread_runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| {
                UStatus::fail_with_code(
                    UCode::INTERNAL,
                    format!("Failed to build current_thread runtime: {e}"),
                )
            })?;
        for (service_name, subscriber) in self.subscribers.iter() {
            match subscriber.receive() {
                Ok(Some(sample)) => {
                    let payload = sample;
                    if let Some(listeners_to_notify) = self.listeners.get(service_name) {
                        for listener in listeners_to_notify.iter() {
                            let listener: &ComparableListener = listener;
                            // FIXME
                            // How would this be done without cloning the payload?
                            // Pretty sure the whole point of using iceoryx2 was to prevent cloning samples
                            // Maybe I'm misunderstanding the clone function right now :(
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
