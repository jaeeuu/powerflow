use std::{mem, ptr::NonNull};

use core_foundation::{base::TCFType, dictionary::CFDictionary};
use thiserror::Error;

use crate::{
    cfdic,
    de::{repr, IORegistry},
    ffi::wrapper::ServiceConnection,
    util::{dict_into, DictParseError},
};

#[derive(Debug, Error)]
pub enum DeviceDataError {
    #[error("Failed to send message: {0}")]
    Send(i32),
    #[error("Failed to receive message: {0}")]
    Receive(i32),
    #[error("Failed to parse message: {0}")]
    Parse(#[from] DictParseError),
    #[error("Received null or invalid response")]
    NullResponse,
    #[error("Invalid diagnostics data")]
    InvalidDiagnostics,
}

pub fn get_device_ioreg(conn: &ServiceConnection) -> Result<IORegistry, DeviceDataError> {
    // Create the request dictionary
    let request = unsafe {
        cfdic! {
            "EntryClass" = "IOPMPowerSource"
            "Request" = "IORegistry"
        }
    };
    
    // Verify request is valid
    if request.is_null() {
        return Err(DeviceDataError::NullResponse);
    }
    
    // Send the request
    unsafe {
        conn.send(request.as_concrete_TypeRef())
            .map_err(DeviceDataError::Send)
    }?;

    // Receive the response
    let response_ptr = conn.receive().map_err(DeviceDataError::Receive)?;
    
    // Check if response is null
    if response_ptr.is_null() {
        return Err(DeviceDataError::NullResponse);
    }
    
    let response = unsafe { CFDictionary::wrap_under_create_rule(response_ptr) };

    // Parse the response
    let data = dict_into::<repr::IORegistryDiagnostic>(response)?;
    
    // Verify diagnostics data is valid
    if data.diagnostics.ioregistry.is_none() {
        return Err(DeviceDataError::InvalidDiagnostics);
    }

    // SAFETY: IORegistry and repr::IORegistry are designed to be the same
    Ok(unsafe { mem::transmute(data.diagnostics.ioregistry) })
}
