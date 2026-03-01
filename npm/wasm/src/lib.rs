//! WebAssembly bindings for agentic-comm.
//!
//! Provides a JavaScript-friendly API wrapping the core CommStore
//! for use in Node.js and browser environments.

use wasm_bindgen::prelude::*;
use serde::Serialize;

/// Serializable channel representation for JSON export.
#[derive(Serialize)]
struct ChannelView {
    name: String,
    message_count: u64,
    created_at: u64,
}

/// Serializable message representation for JSON export.
#[derive(Serialize)]
struct MessageView {
    id: u64,
    channel: String,
    content: String,
    sender: String,
    timestamp: u64,
}

/// JavaScript-facing wrapper around the agentic-comm store.
///
/// Usage from Node.js:
/// ```js
/// const { WasmCommStore } = require('@agentic/comm');
/// const store = new WasmCommStore();
/// store.create_channel("tasks");
/// store.send("tasks", "Build the frontend", "agent-1");
/// console.log(store.channel_count());
/// ```
#[wasm_bindgen]
pub struct WasmCommStore {
    channels: Vec<String>,
    messages: Vec<(String, String, String, u64)>, // (channel, content, sender, timestamp)
    next_id: u64,
}

#[wasm_bindgen]
impl WasmCommStore {
    /// Create a new empty comm store.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            channels: Vec::new(),
            messages: Vec::new(),
            next_id: 1,
        }
    }

    /// Create a new communication channel.
    pub fn create_channel(&mut self, name: &str) -> Result<(), JsValue> {
        if self.channels.contains(&name.to_string()) {
            return Err(JsValue::from_str(&format!("channel already exists: {}", name)));
        }
        self.channels.push(name.to_string());
        Ok(())
    }

    /// Send a message to a channel.
    ///
    /// Returns the assigned message ID.
    pub fn send(
        &mut self,
        channel: &str,
        content: &str,
        sender: &str,
    ) -> Result<u64, JsValue> {
        if !self.channels.contains(&channel.to_string()) {
            return Err(JsValue::from_str(&format!("channel not found: {}", channel)));
        }
        let id = self.next_id;
        self.next_id += 1;
        let timestamp = js_sys::Date::now() as u64;
        self.messages.push((
            channel.to_string(),
            content.to_string(),
            sender.to_string(),
            timestamp,
        ));
        Ok(id)
    }

    /// Get messages from a channel as a JSON array string.
    pub fn receive(&self, channel: &str, limit: usize) -> Result<String, JsValue> {
        let views: Vec<MessageView> = self.messages
            .iter()
            .enumerate()
            .filter(|(_, (ch, _, _, _))| ch == channel)
            .rev()
            .take(limit)
            .map(|(i, (ch, content, sender, ts))| MessageView {
                id: (i + 1) as u64,
                channel: ch.clone(),
                content: content.clone(),
                sender: sender.clone(),
                timestamp: *ts,
            })
            .collect();
        serde_json::to_string(&views)
            .map_err(|e| JsValue::from_str(&format!("serialization error: {}", e)))
    }

    /// Get the number of channels.
    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }

    /// Get the total number of messages.
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// List all channels as a JSON array string.
    pub fn list_channels(&self) -> Result<String, JsValue> {
        let views: Vec<ChannelView> = self.channels
            .iter()
            .map(|name| {
                let count = self.messages.iter().filter(|(ch, _, _, _)| ch == name).count();
                ChannelView {
                    name: name.clone(),
                    message_count: count as u64,
                    created_at: 0,
                }
            })
            .collect();
        serde_json::to_string(&views)
            .map_err(|e| JsValue::from_str(&format!("serialization error: {}", e)))
    }

    /// Search messages by content substring. Returns a JSON array string.
    pub fn search(&self, query: &str, limit: usize) -> Result<String, JsValue> {
        let query_lower = query.to_lowercase();
        let views: Vec<MessageView> = self.messages
            .iter()
            .enumerate()
            .filter(|(_, (_, content, _, _))| content.to_lowercase().contains(&query_lower))
            .take(limit)
            .map(|(i, (ch, content, sender, ts))| MessageView {
                id: (i + 1) as u64,
                channel: ch.clone(),
                content: content.clone(),
                sender: sender.clone(),
                timestamp: *ts,
            })
            .collect();
        serde_json::to_string(&views)
            .map_err(|e| JsValue::from_str(&format!("serialization error: {}", e)))
    }

    /// Export the entire store as a JSON string.
    pub fn to_json(&self) -> Result<String, JsValue> {
        #[derive(Serialize)]
        struct StoreExport {
            channel_count: usize,
            message_count: usize,
            channels: Vec<ChannelView>,
            messages: Vec<MessageView>,
        }

        let channels: Vec<ChannelView> = self.channels
            .iter()
            .map(|name| {
                let count = self.messages.iter().filter(|(ch, _, _, _)| ch == name).count();
                ChannelView {
                    name: name.clone(),
                    message_count: count as u64,
                    created_at: 0,
                }
            })
            .collect();

        let messages: Vec<MessageView> = self.messages
            .iter()
            .enumerate()
            .map(|(i, (ch, content, sender, ts))| MessageView {
                id: (i + 1) as u64,
                channel: ch.clone(),
                content: content.clone(),
                sender: sender.clone(),
                timestamp: *ts,
            })
            .collect();

        let export = StoreExport {
            channel_count: channels.len(),
            message_count: messages.len(),
            channels,
            messages,
        };
        serde_json::to_string(&export)
            .map_err(|e| JsValue::from_str(&format!("serialization error: {}", e)))
    }
}
