use std::{
    collections::{HashMap, HashSet},
    ffi::c_void,
    mem::{MaybeUninit},
    sync::{Arc, RwLock},
    time::Duration,
};

use derive_more::derive::Deref;
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{async_runtime, AppHandle, Manager};
use tauri_specta::Event;
use tokio::{select, sync::mpsc, task::spawn_blocking, time};
use tpower::{
    ffi::{
        core_foundation::runloop::CFRunLoopRun,
        wrapper::{Device, ServiceConnection},
        AMDeviceNotificationCallbackInfo, AMDeviceNotificationSubscribe, Action, InterfaceType,
    },
    provider::{remote::get_device_ioreg, NormalizedResource},
};

use crate::event::DeviceEvent;

#[derive(Default, Deref)]
pub struct DeviceState(RwLock<HashMap<String, (String, HashSet<InterfaceType>)>>);

#[derive(Serialize, Deserialize, Debug, Clone, Event, Type)]
#[serde(rename_all = "camelCase")]
pub struct DevicePowerTickEvent {
    pub udid: String,
    pub data: NormalizedResource,
}

#[derive(Debug)]
pub struct DeviceMessage {
    device: Device,
    action: Action,
}

pub fn start_device_listener() -> mpsc::Receiver<DeviceMessage> {
    let (tx, rx) = mpsc::channel::<DeviceMessage>(10);

    extern "C" fn callback(info: *const AMDeviceNotificationCallbackInfo, context: *mut c_void) {
        if info.is_null() || context.is_null() {
            log::error!("Received null pointer in device notification callback");
            return;
        }
        
        let tx = unsafe { &*(context as *mut mpsc::Sender<DeviceMessage>) };
        let info = unsafe { *info };
        
        if info.device.is_null() {
            log::error!("Received null device pointer in notification");
            return;
        }
        
        let device = unsafe { Device::new(info.device) };
        let tx = tx.clone();

        async_runtime::spawn(async move {
            if let Err(e) = tx.send(DeviceMessage {
                device,
                action: info.action,
            })
            .await {
                log::error!("Failed to send device message: {}", e);
            }
        });
    }

    spawn_blocking(move || {
        let boxed = Arc::new(tx);
        let mut not = MaybeUninit::uninit();
        
        unsafe {
            AMDeviceNotificationSubscribe(
                callback,
                0,
                0,
                Arc::as_ptr(&boxed) as *mut _,
                not.as_mut_ptr(),
            );
        };
        
        log::info!("Device notification subscription started");
        
        unsafe { CFRunLoopRun() };
    });

    rx
}

pub fn start_device_sender(handle: AppHandle) -> async_runtime::JoinHandle<()> {
    let mut rx = start_device_listener();
    let mut timer = time::interval(Duration::from_millis(2000));

    let mut devices: HashMap<Device, ServiceConnection> = HashMap::new();

    async_runtime::spawn(async move {
        loop {
            select! {
                _ = timer.tick() => {
                    let device_count = devices.len();
                    if device_count > 0 {
                        log::debug!("Checking {} connected devices", device_count);
                    }
                    
                    let mut errors = Vec::new();
                    let mut events = Vec::new();
                    
                    for (device, conn) in devices.iter() {
                        match get_device_ioreg(conn) {
                            Ok(res) => {
                                let event = DevicePowerTickEvent {
                                    udid: device.udid.clone(),
                                    data: NormalizedResource::from(&res),
                                };
                                events.push(event);
                            },
                            Err(err) => {
                                log::error!("Failed to get IORegistry for device {}: {}", device.udid, err);
                                errors.push(device.udid.clone());
                            }
                        }
                    }
                    
                    for event in events {
                        if let Err(e) = event.emit(&handle) {
                            log::error!("Failed to emit device power tick event: {}", e);
                        }
                    }
                }
                Some(DeviceMessage { device, action }) = rx.recv() => {
                    match action {
                        Action::Attached => {
                            log::info!("Device attached with interface: {:?}", device.interface_type);
                            
                            match device.prepare_device() {
                                Ok(_) => {
                                    let device_name = device.name();
                                    log::info!("Device prepared successfully: {}", device_name);
                                    
                                    match device.start_service("com.apple.mobile.diagnostics_relay") {
                                        conn => {
                                            let event = DeviceEvent {
                                                udid: device.udid.clone(),
                                                name: device_name,
                                                interface: device.interface_type,
                                                action,
                                            };
                                            
                                            if let Err(e) = event.emit(&handle) {
                                                log::error!("Failed to emit device event: {}", e);
                                            } else {
                                                devices.insert(device, conn);
                                            }
                                        }
                                    }
                                },
                                Err(e) => {
                                    log::error!("Failed to prepare device {}: {:?}", device.udid, e);
                                }
                            }
                        },
                        Action::Detached => {
                            log::info!("Device detached: {}", device.udid);
                            
                            let event = DeviceEvent {
                                udid: device.udid.clone(),
                                name: String::new(),
                                interface: device.interface_type,
                                action,
                            };
                            
                            if let Err(e) = event.emit(&handle) {
                                log::error!("Failed to emit device detach event: {}", e);
                            }
                            
                            devices.remove(&device);
                        },
                        _ => {
                            log::debug!("Unhandled device action: {:?} for {}", action, device.udid);
                        }
                    }
                }
            }
        }
    })
}

pub fn setup_device_listener(app: AppHandle) {
    let app_handle = app.clone();
    
    DeviceEvent::listen(&app, move |event| {
        let event = event.payload;
        
        let state_ref = app_handle.state::<DeviceState>();
        
        {
            if let Ok(mut state) = state_ref.write() {
                let entry = state.entry(event.udid.clone()).or_insert_with(|| (event.name, HashSet::new()));
                
                match event.action {
                    Action::Attached => {
                        entry.1.insert(event.interface);
                        log::debug!("Added interface {:?} for device {}", event.interface, event.udid);
                    }
                    Action::Detached => {
                        entry.1.remove(&event.interface);
                        log::debug!("Removed interface {:?} for device {}", event.interface, event.udid);
                    }
                    _ => (),
                }
            } else {
                log::error!("Failed to acquire write lock on device state");
            }
        }
    });
}
