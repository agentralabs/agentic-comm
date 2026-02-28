//! ChannelManager — manages channel lifecycle independently.
//!
//! A standalone component for creating, joining, leaving, closing, and
//! configuring communication channels.  It can be used alongside `CommStore`
//! as a re-usable building block.

use crate::{
    Channel, ChannelConfig, ChannelState, ChannelType, CommError, CommResult,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Manages channel lifecycle, membership, and state transitions.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ChannelManager {
    channels: HashMap<u64, Channel>,
    next_id: u64,
}

impl ChannelManager {
    /// Create a new, empty channel manager.
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
            next_id: 1,
        }
    }

    /// Create a new channel and return a reference to it.
    pub fn create(
        &mut self,
        name: &str,
        channel_type: ChannelType,
        config: Option<ChannelConfig>,
    ) -> CommResult<&Channel> {
        // Validate name: 1-128 chars, alphanumeric / hyphen / underscore
        if name.is_empty() || name.len() > 128 {
            return Err(CommError::InvalidChannelName(
                "Channel name must be 1-128 characters".to_string(),
            ));
        }
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(CommError::InvalidChannelName(
                "Channel name must contain only alphanumeric characters, hyphens, or underscores"
                    .to_string(),
            ));
        }

        let id = self.next_id;
        self.next_id += 1;

        let channel = Channel {
            id,
            name: name.to_string(),
            channel_type,
            created_at: Utc::now(),
            participants: Vec::new(),
            config: config.unwrap_or_default(),
            state: ChannelState::Active,
            comm_id: None,
            contract_ref: None,
        };

        self.channels.insert(id, channel);
        Ok(self.channels.get(&id).unwrap())
    }

    /// Get an immutable reference to a channel by id.
    pub fn get(&self, id: u64) -> Option<&Channel> {
        self.channels.get(&id)
    }

    /// Get a mutable reference to a channel by id.
    pub fn get_mut(&mut self, id: u64) -> Option<&mut Channel> {
        self.channels.get_mut(&id)
    }

    /// List all channels.
    pub fn list(&self) -> Vec<&Channel> {
        self.channels.values().collect()
    }

    /// Add a participant to a channel.
    pub fn join(&mut self, channel_id: u64, agent_id: &str) -> CommResult<()> {
        let channel = self
            .channels
            .get_mut(&channel_id)
            .ok_or(CommError::ChannelNotFound(channel_id))?;

        if channel.participants.contains(&agent_id.to_string()) {
            return Err(CommError::AlreadyInChannel(
                agent_id.to_string(),
                channel_id,
            ));
        }

        if channel.config.max_participants > 0
            && channel.participants.len() >= channel.config.max_participants as usize
        {
            return Err(CommError::ChannelFull(channel_id));
        }

        channel.participants.push(agent_id.to_string());
        Ok(())
    }

    /// Remove a participant from a channel.
    pub fn leave(&mut self, channel_id: u64, agent_id: &str) -> CommResult<()> {
        let channel = self
            .channels
            .get_mut(&channel_id)
            .ok_or(CommError::ChannelNotFound(channel_id))?;

        let pos = channel
            .participants
            .iter()
            .position(|p| p == agent_id)
            .ok_or_else(|| CommError::NotInChannel(agent_id.to_string(), channel_id))?;

        channel.participants.remove(pos);
        Ok(())
    }

    /// Close a channel (sets state to [`ChannelState::Closed`]).
    pub fn close(&mut self, channel_id: u64) -> CommResult<()> {
        let channel = self
            .channels
            .get_mut(&channel_id)
            .ok_or(CommError::ChannelNotFound(channel_id))?;
        channel.state = ChannelState::Closed;
        Ok(())
    }

    /// Transition a channel to a new state.
    pub fn set_state(&mut self, channel_id: u64, state: ChannelState) -> CommResult<()> {
        let channel = self
            .channels
            .get_mut(&channel_id)
            .ok_or(CommError::ChannelNotFound(channel_id))?;
        channel.state = state;
        Ok(())
    }

    /// Update the configuration of a channel.
    pub fn set_config(&mut self, channel_id: u64, config: ChannelConfig) -> CommResult<()> {
        let channel = self
            .channels
            .get_mut(&channel_id)
            .ok_or(CommError::ChannelNotFound(channel_id))?;
        channel.config = config;
        Ok(())
    }

    /// Return the number of channels.
    pub fn count(&self) -> usize {
        self.channels.len()
    }

    /// Immutable access to the underlying channel map.
    pub fn channels(&self) -> &HashMap<u64, Channel> {
        &self.channels
    }

    /// Mutable access to the underlying channel map.
    pub fn channels_mut(&mut self) -> &mut HashMap<u64, Channel> {
        &mut self.channels
    }

    /// Advance and return the next channel ID.
    pub fn next_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_channel() {
        let mut mgr = ChannelManager::new();
        let ch = mgr.create("general", ChannelType::Group, None).unwrap();
        assert_eq!(ch.name, "general");
        assert_eq!(ch.channel_type, ChannelType::Group);
        assert_eq!(ch.state, ChannelState::Active);
        assert_eq!(mgr.count(), 1);
    }

    #[test]
    fn create_channel_invalid_name() {
        let mut mgr = ChannelManager::new();
        let result = mgr.create("", ChannelType::Direct, None);
        assert!(result.is_err());

        let result = mgr.create("has spaces", ChannelType::Direct, None);
        assert!(result.is_err());
    }

    #[test]
    fn join_and_leave() {
        let mut mgr = ChannelManager::new();
        let ch = mgr.create("team", ChannelType::Group, None).unwrap();
        let id = ch.id;

        mgr.join(id, "alice").unwrap();
        mgr.join(id, "bob").unwrap();
        assert_eq!(mgr.get(id).unwrap().participants.len(), 2);

        // Duplicate join should fail
        let result = mgr.join(id, "alice");
        assert!(result.is_err());

        mgr.leave(id, "alice").unwrap();
        assert_eq!(mgr.get(id).unwrap().participants.len(), 1);
        assert_eq!(mgr.get(id).unwrap().participants[0], "bob");

        // Leaving when not in channel should fail
        let result = mgr.leave(id, "alice");
        assert!(result.is_err());
    }

    #[test]
    fn join_nonexistent_channel() {
        let mut mgr = ChannelManager::new();
        let result = mgr.join(999, "alice");
        assert!(result.is_err());
    }

    #[test]
    fn channel_full() {
        let mut mgr = ChannelManager::new();
        let config = ChannelConfig {
            max_participants: 1,
            ..Default::default()
        };
        let ch = mgr
            .create("tiny", ChannelType::Direct, Some(config))
            .unwrap();
        let id = ch.id;

        mgr.join(id, "alice").unwrap();
        let result = mgr.join(id, "bob");
        assert!(result.is_err());
    }

    #[test]
    fn close_channel() {
        let mut mgr = ChannelManager::new();
        let ch = mgr.create("temp", ChannelType::Group, None).unwrap();
        let id = ch.id;

        mgr.close(id).unwrap();
        assert_eq!(mgr.get(id).unwrap().state, ChannelState::Closed);
    }

    #[test]
    fn state_transitions() {
        let mut mgr = ChannelManager::new();
        let ch = mgr.create("workflow", ChannelType::Group, None).unwrap();
        let id = ch.id;

        assert_eq!(mgr.get(id).unwrap().state, ChannelState::Active);

        mgr.set_state(id, ChannelState::Paused).unwrap();
        assert_eq!(mgr.get(id).unwrap().state, ChannelState::Paused);

        mgr.set_state(id, ChannelState::Draining).unwrap();
        assert_eq!(mgr.get(id).unwrap().state, ChannelState::Draining);

        mgr.set_state(id, ChannelState::Archived).unwrap();
        assert_eq!(mgr.get(id).unwrap().state, ChannelState::Archived);

        // Setting state on nonexistent channel should fail
        let result = mgr.set_state(999, ChannelState::Active);
        assert!(result.is_err());
    }

    #[test]
    fn config_update() {
        let mut mgr = ChannelManager::new();
        let ch = mgr.create("conf-test", ChannelType::PubSub, None).unwrap();
        let id = ch.id;

        assert!(!mgr.get(id).unwrap().config.encryption_required);

        let new_config = ChannelConfig {
            max_participants: 50,
            ttl_seconds: 3600,
            persistence: true,
            encryption_required: true,
            ..Default::default()
        };
        mgr.set_config(id, new_config).unwrap();

        let ch = mgr.get(id).unwrap();
        assert_eq!(ch.config.max_participants, 50);
        assert_eq!(ch.config.ttl_seconds, 3600);
        assert!(ch.config.encryption_required);
    }

    #[test]
    fn list_channels() {
        let mut mgr = ChannelManager::new();
        mgr.create("alpha", ChannelType::Group, None).unwrap();
        mgr.create("beta", ChannelType::Direct, None).unwrap();
        mgr.create("gamma", ChannelType::Broadcast, None).unwrap();

        let list = mgr.list();
        assert_eq!(list.len(), 3);
    }

    #[test]
    fn next_id_advances() {
        let mut mgr = ChannelManager::new();
        assert_eq!(mgr.next_id(), 1);
        assert_eq!(mgr.next_id(), 2);
        assert_eq!(mgr.next_id(), 3);
    }

    #[test]
    fn channels_accessor() {
        let mut mgr = ChannelManager::new();
        mgr.create("test", ChannelType::Group, None).unwrap();
        assert_eq!(mgr.channels().len(), 1);
        assert_eq!(mgr.channels_mut().len(), 1);
    }
}
