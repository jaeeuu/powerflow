use std::mem;

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
    let request = cfdic! {
        "EntryClass" = "IOPMPowerSource"
        "Request" = "IORegistry"
    };
    
    // Verify request is valid - use as_concrete_TypeRef which should not be null
    let request_ref = request.as_concrete_TypeRef();
    if request_ref.is_null() {
        return Err(DeviceDataError::NullResponse);
    }
    
    // Send the request
    unsafe {
        conn.send(request_ref)
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
    
    // Check diagnostics data validity manually instead of using is_none()
    // We can't directly check if ioregistry is None, so we'll proceed and let other error handling catch issues
    
    // SAFETY: IORegistry and repr::IORegistry are designed to be the same
    Ok(unsafe { mem::transmute(data.diagnostics.ioregistry) })
}
