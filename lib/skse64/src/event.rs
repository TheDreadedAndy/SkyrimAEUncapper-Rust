//!
//! @file event.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief Allows a plugin to register to listen to SKSE events.
//! @bug No known bugs.
//!

use core::ffi::c_char;

use racy_cell::RacyCell;

use crate::plugin_api::{Message, SkseMessagingInterface, SkseInterface, InterfaceId};
use crate::plugin_api;

const VEC_INIT: Vec<fn(&Message)> = Vec::new();
static SKSE_HANDLERS: RacyCell<[Vec<fn(&Message)>; Message::SKSE_MAX]>
    = RacyCell::new([VEC_INIT; Message::SKSE_MAX]);

/// Registers our listener wrapper to the SKSE message sender.
pub (in crate) fn init_listener(
    skse: &SkseInterface
) {
    unsafe {
        // SAFETY: The SkseInterface structure is provided by SKSE and is valid.
        let msg_if = (skse.query_interface)(InterfaceId::Messaging) as *mut SkseMessagingInterface;
        ((*msg_if).register_listener)(
            plugin_api::handle(),
            "SKSE\0".as_bytes().as_ptr() as *const c_char,
            skse_listener
        );
    }
}

/// Registers a new listener for a skse message.
pub fn register_listener(
    msg_type: u32,
    callback: fn(&Message)
) {
    assert!(msg_type < Message::SKSE_MAX as u32);
    unsafe {
        (*SKSE_HANDLERS.get())[msg_type as usize].push(callback);
    }
}

/// Handles a message from the skse plugin by forwarding it to the registered listener.
unsafe extern "system" fn skse_listener(
    msg: *mut Message
) {
    let msg = msg.as_ref().unwrap();

    // Only handle messages we understand.
    if msg.msg_type >= Message::SKSE_MAX as u32 { return; }

    for callback in (*SKSE_HANDLERS.get())[msg.msg_type as usize].iter() {
        callback(msg);
    }
}
