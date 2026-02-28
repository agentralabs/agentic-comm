//! C-compatible FFI bindings for AgenticComm.
//!
//! Provides a minimal C API for creating stores, sending messages,
//! creating channels, and saving/loading .acomm files.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::Path;

use agentic_comm::{ChannelType, CommStore, MessageType};

/// Crate version exposed for foreign runtimes.
#[no_mangle]
pub extern "C" fn acomm_version() -> *const c_char {
    // Static string — lives for the entire program lifetime.
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr() as *const c_char
}

/// Create a new empty CommStore. Returns a heap-allocated pointer.
/// The caller must free it with `acomm_store_free`.
#[no_mangle]
pub extern "C" fn acomm_store_create() -> *mut CommStore {
    Box::into_raw(Box::new(CommStore::new()))
}

/// Free a CommStore previously created by `acomm_store_create` or `acomm_load`.
///
/// # Safety
/// The pointer must have been returned by `acomm_store_create` or `acomm_load`
/// and must not have been freed already.
#[no_mangle]
pub unsafe extern "C" fn acomm_store_free(store: *mut CommStore) {
    if !store.is_null() {
        drop(Box::from_raw(store));
    }
}

/// Create a channel in the store. Returns the channel ID, or 0 on error.
///
/// `channel_type`: 0 = Direct, 1 = Group, 2 = Broadcast, 3 = PubSub
///
/// # Safety
/// `store` must be a valid pointer from `acomm_store_create`.
/// `name` must be a valid null-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn acomm_create_channel(
    store: *mut CommStore,
    name: *const c_char,
    channel_type: u32,
) -> u64 {
    if store.is_null() || name.is_null() {
        return 0;
    }

    let store = &mut *store;
    let name = match CStr::from_ptr(name).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let ct = match channel_type {
        0 => ChannelType::Direct,
        1 => ChannelType::Group,
        2 => ChannelType::Broadcast,
        3 => ChannelType::PubSub,
        _ => return 0,
    };

    match store.create_channel(name, ct, None) {
        Ok(ch) => ch.id,
        Err(_) => 0,
    }
}

/// Send a message to a channel. Returns the message ID, or 0 on error.
///
/// # Safety
/// `store` must be a valid pointer. `sender` and `content` must be valid
/// null-terminated UTF-8 strings.
#[no_mangle]
pub unsafe extern "C" fn acomm_send_message(
    store: *mut CommStore,
    channel_id: u64,
    sender: *const c_char,
    content: *const c_char,
) -> u64 {
    if store.is_null() || sender.is_null() || content.is_null() {
        return 0;
    }

    let store = &mut *store;
    let sender = match CStr::from_ptr(sender).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    let content = match CStr::from_ptr(content).to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    match store.send_message(channel_id, sender, content, MessageType::Text) {
        Ok(msg) => msg.id,
        Err(_) => 0,
    }
}

/// Receive messages from a channel as a JSON string.
/// Returns a heap-allocated null-terminated JSON string, or null on error.
/// The caller must free the returned string with `acomm_string_free`.
///
/// # Safety
/// `store` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn acomm_receive_messages(
    store: *mut CommStore,
    channel_id: u64,
) -> *mut c_char {
    if store.is_null() {
        return std::ptr::null_mut();
    }

    let store = &*store;

    match store.receive_messages(channel_id, None, None) {
        Ok(msgs) => match serde_json::to_string(&msgs) {
            Ok(json) => match CString::new(json) {
                Ok(cs) => cs.into_raw(),
                Err(_) => std::ptr::null_mut(),
            },
            Err(_) => std::ptr::null_mut(),
        },
        Err(_) => std::ptr::null_mut(),
    }
}

/// List all channels as a JSON string.
/// The caller must free the returned string with `acomm_string_free`.
///
/// # Safety
/// `store` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn acomm_list_channels(store: *mut CommStore) -> *mut c_char {
    if store.is_null() {
        return std::ptr::null_mut();
    }

    let store = &*store;
    let channels = store.list_channels();

    match serde_json::to_string(&channels) {
        Ok(json) => match CString::new(json) {
            Ok(cs) => cs.into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
        Err(_) => std::ptr::null_mut(),
    }
}

/// Save the store to a file. Returns true on success, false on error.
///
/// # Safety
/// `store` must be a valid pointer. `path` must be a valid null-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn acomm_save(store: *mut CommStore, path: *const c_char) -> bool {
    if store.is_null() || path.is_null() {
        return false;
    }

    let store = &*store;
    let path = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };

    store.save(Path::new(path)).is_ok()
}

/// Load a store from a file. Returns a heap-allocated pointer, or null on error.
/// The caller must free it with `acomm_store_free`.
///
/// # Safety
/// `path` must be a valid null-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn acomm_load(path: *const c_char) -> *mut CommStore {
    if path.is_null() {
        return std::ptr::null_mut();
    }

    let path = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    match CommStore::load(Path::new(path)) {
        Ok(store) => Box::into_raw(Box::new(store)),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Free a string previously returned by an `acomm_*` function.
///
/// # Safety
/// The pointer must have been returned by an acomm function that returns `*mut c_char`,
/// and must not have been freed already.
#[no_mangle]
pub unsafe extern "C" fn acomm_string_free(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}
