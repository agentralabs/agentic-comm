# AgenticComm Specification — Part 2

> **Version:** 0.1.0
> **Status:** Pre-Implementation Specification
> **Covers:** Message Engine, Query Engine, Indexes, Validation

---

# SPEC-05: MESSAGE ENGINE

## 5.1 Engine Overview

The Message Engine handles all message lifecycle operations: creation, encryption, transmission, receipt, and delivery.

```
MESSAGE LIFECYCLE:
══════════════════

  CREATE          ENCRYPT         SIGN            TRANSMIT
     │               │              │                │
     ▼               ▼              ▼                ▼
 ┌────────┐    ┌─────────┐    ┌─────────┐    ┌──────────┐
 │ Draft  │───▶│Encrypted│───▶│ Signed  │───▶│ In-Flight│
 └────────┘    └─────────┘    └─────────┘    └──────────┘
                                                   │
     ┌─────────────────────────────────────────────┘
     │
     ▼            VERIFY          DECRYPT         DELIVER
 ┌────────┐    ┌─────────┐    ┌─────────┐    ┌──────────┐
 │Received│───▶│Verified │───▶│Decrypted│───▶│Delivered │
 └────────┘    └─────────┘    └─────────┘    └──────────┘
                                                   │
                                                   ▼
                                             ┌──────────┐
                                             │ Receipted│
                                             └──────────┘
```

## 5.2 Core Engine

```rust
/// The message engine
pub struct MessageEngine {
    /// Channel manager
    channels: ChannelManager,
    
    /// Encryption service
    encryption: EncryptionService,
    
    /// Identity integration
    identity: IdentityBridge,
    
    /// Contract integration
    contract: ContractBridge,
    
    /// Memory integration (optional)
    memory: Option<MemoryBridge>,
    
    /// Time integration (optional)
    time: Option<TimeBridge>,
    
    /// Outbound queue
    outbound: OutboundQueue,
    
    /// Inbound queue
    inbound: InboundQueue,
    
    /// Pending temporal messages
    temporal_pending: TemporalQueue,
    
    /// Message handlers
    handlers: HandlerRegistry,
    
    /// Metrics
    metrics: EngineMetrics,
}

impl MessageEngine {
    /// Create a new message
    pub async fn create_message(
        &self,
        content: MessageContent,
        recipients: Vec<Recipient>,
        channel: ChannelId,
        options: MessageOptions,
    ) -> Result<Message, CommError> {
        // 1. Validate content against channel policy
        self.validate_content(&content, &channel).await?;
        
        // 2. Check consent for all recipients
        self.check_consent(&recipients, &content).await?;
        
        // 3. Create message structure
        let message = Message {
            id: MessageId::new(),
            content,
            sender: self.identity.current_anchor(),
            recipients,
            channel,
            metadata: self.build_metadata(&options),
            signature: Signature::empty(), // Will be filled
            receipt: None, // Will be filled
        };
        
        // 4. Generate receipt from Identity
        let receipt = self.identity.create_receipt(
            "comm.message.create",
            &message.id,
        ).await?;
        
        Ok(message.with_receipt(receipt))
    }
    
    /// Send a message
    pub async fn send(&self, message: Message) -> Result<SendResult, CommError> {
        // 1. Get channel
        let channel = self.channels.get(&message.channel)?;
        
        // 2. Encrypt message
        let encrypted = self.encrypt_for_channel(&message, &channel).await?;
        
        // 3. Sign message
        let signed = self.identity.sign_message(&encrypted).await?;
        
        // 4. Check contract policies
        self.contract.check_send_policy(&signed, &channel).await?;
        
        // 5. Handle temporal targeting
        if let Some(temporal) = message.metadata.temporal_target {
            return self.schedule_temporal(signed, temporal).await;
        }
        
        // 6. Transmit
        let result = self.transmit(&signed, &channel).await?;
        
        // 7. Store in memory (if available)
        if let Some(ref memory) = self.memory {
            memory.record_sent(&message).await?;
        }
        
        // 8. Update metrics
        self.metrics.record_send(&result);
        
        Ok(result)
    }
    
    /// Receive a message
    pub async fn receive(&self, envelope: Envelope) -> Result<Message, CommError> {
        // 1. Verify signature
        let verified = self.identity.verify_signature(&envelope).await?;
        
        // 2. Get channel
        let channel = self.channels.get(&envelope.channel)?;
        
        // 3. Decrypt
        let decrypted = self.decrypt_for_channel(&verified, &channel).await?;
        
        // 4. Parse content
        let message = self.parse_message(&decrypted)?;
        
        // 5. Check if we consent to this content type
        self.check_receive_consent(&message).await?;
        
        // 6. Generate receipt
        let receipt = self.identity.create_receipt(
            "comm.message.receive",
            &message.id,
        ).await?;
        
        // 7. Handle special message types
        self.handle_special_types(&message).await?;
        
        // 8. Store in memory
        if let Some(ref memory) = self.memory {
            memory.record_received(&message).await?;
        }
        
        // 9. Notify handlers
        self.notify_handlers(&message).await;
        
        Ok(message.with_receipt(receipt))
    }
}
```

## 5.3 Channel Manager

```rust
/// Manages communication channels
pub struct ChannelManager {
    /// Active channels
    channels: RwLock<HashMap<ChannelId, Channel>>,
    
    /// Channel index
    index: ChannelIndex,
    
    /// Encryption keys per channel
    keys: KeyStore,
    
    /// Channel policies
    policies: PolicyStore,
}

impl ChannelManager {
    /// Create a new channel
    pub async fn create_channel(
        &self,
        channel_type: ChannelType,
        participants: Vec<IdentityAnchor>,
        config: ChannelConfig,
    ) -> Result<Channel, CommError> {
        // 1. Generate channel ID
        let id = ChannelId::new();
        
        // 2. Request consent from all participants
        let consents = self.request_consents(&participants, &channel_type).await?;
        
        // 3. Generate encryption keys
        let encryption = self.setup_encryption(&participants, &config).await?;
        
        // 4. Create channel
        let channel = Channel {
            id,
            channel_type,
            participants: self.build_participants(participants, consents),
            state: ChannelState::Active,
            encryption,
            contract: config.contract,
            metadata: config.metadata,
            created_at: Timestamp::now(),
            last_activity: Timestamp::now(),
        };
        
        // 5. Store
        self.channels.write().insert(id, channel.clone());
        self.index.add(&channel)?;
        
        Ok(channel)
    }
    
    /// Join an existing channel
    pub async fn join_channel(
        &self,
        channel_id: ChannelId,
        agent: IdentityAnchor,
    ) -> Result<(), CommError> {
        let mut channels = self.channels.write();
        let channel = channels.get_mut(&channel_id)
            .ok_or(CommError::ChannelNotFound)?;
        
        // 1. Check if channel accepts new members
        if !channel.accepts_new_members() {
            return Err(CommError::ChannelClosed);
        }
        
        // 2. Check consent
        self.check_join_consent(&agent, channel).await?;
        
        // 3. Distribute keys to new member
        self.distribute_key(&agent, channel).await?;
        
        // 4. Add participant
        channel.participants.push(ChannelParticipant {
            agent,
            role: ParticipantRole::Member,
            permissions: channel.default_permissions(),
            joined_at: Timestamp::now(),
            last_seen: Timestamp::now(),
            contributions: 0,
        });
        
        // 5. Rotate keys (if policy requires)
        if channel.encryption.rotation_policy.on_membership_change {
            self.rotate_channel_keys(channel).await?;
        }
        
        Ok(())
    }
    
    /// Leave a channel
    pub async fn leave_channel(
        &self,
        channel_id: ChannelId,
        agent: IdentityAnchor,
    ) -> Result<(), CommError> {
        let mut channels = self.channels.write();
        let channel = channels.get_mut(&channel_id)
            .ok_or(CommError::ChannelNotFound)?;
        
        // 1. Remove participant
        channel.participants.retain(|p| p.agent != agent);
        
        // 2. Rotate keys
        if channel.encryption.rotation_policy.on_membership_change {
            self.rotate_channel_keys(channel).await?;
        }
        
        // 3. If empty, archive channel
        if channel.participants.is_empty() {
            channel.state = ChannelState::Archived;
        }
        
        Ok(())
    }
}
```

## 5.4 Encryption Service

```rust
/// Handles all encryption operations
pub struct EncryptionService {
    /// Key derivation
    kdf: KeyDerivationFunction,
    
    /// Supported schemes
    schemes: Vec<EncryptionScheme>,
    
    /// Key cache
    key_cache: LruCache<ChannelId, ChannelKey>,
}

impl EncryptionService {
    /// Encrypt message for channel
    pub async fn encrypt(
        &self,
        message: &Message,
        channel: &Channel,
    ) -> Result<EncryptedMessage, CommError> {
        // 1. Get or derive channel key
        let key = self.get_channel_key(channel).await?;
        
        // 2. Generate nonce
        let nonce = self.generate_nonce();
        
        // 3. Serialize message
        let plaintext = self.serialize_message(message)?;
        
        // 4. Encrypt
        let ciphertext = match channel.encryption.scheme {
            EncryptionScheme::ChaCha20Poly1305 => {
                self.encrypt_chacha20(&plaintext, &key, &nonce)?
            }
            EncryptionScheme::Aes256Gcm => {
                self.encrypt_aes256(&plaintext, &key, &nonce)?
            }
            EncryptionScheme::XChaCha20Poly1305 => {
                self.encrypt_xchacha20(&plaintext, &key, &nonce)?
            }
            EncryptionScheme::None => plaintext,
        };
        
        Ok(EncryptedMessage {
            ciphertext,
            nonce,
            key_epoch: channel.encryption.key_epoch,
            scheme: channel.encryption.scheme,
        })
    }
    
    /// Decrypt message from channel
    pub async fn decrypt(
        &self,
        encrypted: &EncryptedMessage,
        channel: &Channel,
    ) -> Result<Message, CommError> {
        // 1. Get channel key for epoch
        let key = self.get_channel_key_for_epoch(
            channel,
            encrypted.key_epoch,
        ).await?;
        
        // 2. Decrypt
        let plaintext = match encrypted.scheme {
            EncryptionScheme::ChaCha20Poly1305 => {
                self.decrypt_chacha20(&encrypted.ciphertext, &key, &encrypted.nonce)?
            }
            EncryptionScheme::Aes256Gcm => {
                self.decrypt_aes256(&encrypted.ciphertext, &key, &encrypted.nonce)?
            }
            EncryptionScheme::XChaCha20Poly1305 => {
                self.decrypt_xchacha20(&encrypted.ciphertext, &key, &encrypted.nonce)?
            }
            EncryptionScheme::None => encrypted.ciphertext.clone(),
        };
        
        // 3. Deserialize
        self.deserialize_message(&plaintext)
    }
    
    /// Setup end-to-end encryption for direct channel
    pub async fn setup_e2e(
        &self,
        local: &IdentityAnchor,
        remote: &IdentityAnchor,
    ) -> Result<SharedSecret, CommError> {
        // X25519 key agreement
        let local_private = self.get_private_key(local)?;
        let remote_public = self.get_public_key(remote).await?;
        
        let shared = x25519::diffie_hellman(&local_private, &remote_public);
        
        // Derive encryption key from shared secret
        let key = self.kdf.derive(&shared, b"agentic-comm-e2e")?;
        
        Ok(key)
    }
}
```

## 5.5 Semantic Processor

```rust
/// Processes semantic message content
pub struct SemanticProcessor {
    /// Embedding model
    embedder: EmbeddingModel,
    
    /// Graph operations
    graph_ops: GraphOperations,
    
    /// Conflict detector
    conflict_detector: ConflictDetector,
}

impl SemanticProcessor {
    /// Extract semantic fragment from cognitive state
    pub fn extract_fragment(
        &self,
        focus: &[NodeId],
        context_depth: usize,
        perspective: &Perspective,
    ) -> Result<SemanticFragment, CommError> {
        // 1. Get focused nodes
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        
        for node_id in focus {
            let node = self.graph_ops.get_node(node_id)?;
            nodes.push(node.clone());
            
            // 2. Expand context
            let context = self.graph_ops.get_context(node_id, context_depth)?;
            nodes.extend(context.nodes);
            edges.extend(context.edges);
        }
        
        // 3. Deduplicate
        nodes.sort_by_key(|n| n.id);
        nodes.dedup_by_key(|n| n.id);
        
        // 4. Compute graft points
        let graft_points = self.compute_graft_points(&nodes)?;
        
        // 5. Build context anchors
        let context_anchors = self.build_context_anchors(&nodes)?;
        
        Ok(SemanticFragment {
            nodes,
            edges,
            graft_points,
            context: context_anchors,
            perspective: perspective.clone(),
        })
    }
    
    /// Graft received fragment onto local graph
    pub fn graft_fragment(
        &self,
        fragment: &SemanticFragment,
        local_graph: &mut CognitiveGraph,
    ) -> Result<GraftResult, CommError> {
        let mut grafted_nodes = 0;
        let mut new_connections = 0;
        let mut conflicts = Vec::new();
        
        // 1. For each graft point, find matching local nodes
        for graft_point in &fragment.graft_points {
            let matches = self.find_graft_matches(
                &graft_point,
                local_graph,
            )?;
            
            for local_match in matches {
                // 2. Check for conflicts
                if let Some(conflict) = self.conflict_detector.check(
                    &fragment.nodes,
                    &local_match,
                )? {
                    conflicts.push(conflict);
                    continue;
                }
                
                // 3. Create connection
                local_graph.add_edge(CognitiveEdge {
                    id: EdgeId::new(),
                    from: graft_point.fragment_node,
                    to: local_match.node_id,
                    edge_type: graft_point.connection_type,
                    strength: local_match.similarity,
                    metadata: EdgeMetadata::graft(),
                })?;
                
                new_connections += 1;
            }
        }
        
        // 4. Add fragment nodes to local graph
        for node in &fragment.nodes {
            if local_graph.add_node(node.clone())? {
                grafted_nodes += 1;
            }
        }
        
        // 5. Add fragment edges
        for edge in &fragment.edges {
            local_graph.add_edge(edge.clone())?;
        }
        
        Ok(GraftResult {
            nodes_grafted: grafted_nodes,
            connections_formed: new_connections,
            conflicts,
            new_understanding: self.identify_new_insights(local_graph)?,
            merged_perspective: self.merge_perspectives(
                &local_graph.perspective,
                &fragment.perspective,
            )?,
        })
    }
}
```

## 5.6 Affect Processor

```rust
/// Processes affect transmission
pub struct AffectProcessor {
    /// Current affect state
    current_state: RwLock<AffectState>,
    
    /// Affect history
    history: AffectHistory,
    
    /// Contagion model
    contagion_model: ContagionModel,
    
    /// Resistance factors
    resistance: ResistanceFactors,
}

impl AffectProcessor {
    /// Encode current affect for transmission
    pub fn encode_affect(&self, strength: f64) -> AffectPayload {
        let state = self.current_state.read();
        
        AffectPayload {
            affect: state.clone(),
            contagion_strength: strength,
            resistable: true,
        }
    }
    
    /// Process received affect
    pub fn receive_affect(
        &self,
        payload: &AffectPayload,
        sender_trust: f64,
    ) -> Result<ContagionResult, CommError> {
        let mut current = self.current_state.write();
        
        // 1. Calculate effective contagion strength
        let effective_strength = payload.contagion_strength 
            * sender_trust 
            * (1.0 - self.resistance.current());
        
        // 2. Apply contagion
        let delta = self.contagion_model.apply(
            &current,
            &payload.affect,
            effective_strength,
        );
        
        // 3. Update state
        let new_state = current.apply_delta(&delta);
        *current = new_state.clone();
        
        // 4. Record in history
        self.history.record(AffectEvent {
            received: payload.affect.clone(),
            delta: delta.clone(),
            new_state: new_state.clone(),
            timestamp: Timestamp::now(),
        });
        
        Ok(ContagionResult {
            received: payload.affect.clone(),
            affect_delta: delta,
            new_state,
            contagion_strength: effective_strength,
            resistance: self.resistance.current(),
        })
    }
    
    /// Resist incoming affect
    pub fn resist(&self) -> f64 {
        self.resistance.increase();
        self.resistance.current()
    }
    
    /// Lower resistance
    pub fn open(&self) -> f64 {
        self.resistance.decrease();
        self.resistance.current()
    }
}
```

## 5.7 Temporal Scheduler

```rust
/// Schedules temporal messages
pub struct TemporalScheduler {
    /// Pending messages by target time
    pending: BTreeMap<Timestamp, Vec<ScheduledMessage>>,
    
    /// Conditional messages
    conditional: Vec<ConditionalMessage>,
    
    /// Time integration
    time: TimeBridge,
    
    /// Check interval
    check_interval: Duration,
}

impl TemporalScheduler {
    /// Schedule a message for future delivery
    pub async fn schedule(
        &self,
        message: Message,
        target: TemporalTarget,
    ) -> Result<ScheduleId, CommError> {
        let schedule_id = ScheduleId::new();
        
        match target {
            TemporalTarget::FutureAbsolute(timestamp) => {
                self.pending.entry(timestamp)
                    .or_default()
                    .push(ScheduledMessage {
                        id: schedule_id,
                        message,
                        scheduled_at: Timestamp::now(),
                    });
            }
            
            TemporalTarget::FutureRelative(duration) => {
                let timestamp = Timestamp::now() + duration;
                self.pending.entry(timestamp)
                    .or_default()
                    .push(ScheduledMessage {
                        id: schedule_id,
                        message,
                        scheduled_at: Timestamp::now(),
                    });
            }
            
            TemporalTarget::Conditional(condition) => {
                self.conditional.push(ConditionalMessage {
                    id: schedule_id,
                    message,
                    condition,
                    scheduled_at: Timestamp::now(),
                });
            }
            
            TemporalTarget::Retroactive(past_time) => {
                // Deliver to memory as historical event
                self.deliver_retroactive(message, past_time).await?;
            }
            
            TemporalTarget::Optimal(optimality) => {
                self.schedule_optimal(message, optimality).await?;
            }
            
            TemporalTarget::Eternal => {
                self.store_eternal(message).await?;
            }
            
            TemporalTarget::Immediate => {
                // Don't schedule, just return
                return Err(CommError::NotTemporal);
            }
        }
        
        Ok(schedule_id)
    }
    
    /// Check and deliver due messages
    pub async fn check_and_deliver(&self) -> Vec<DeliveryResult> {
        let now = Timestamp::now();
        let mut results = Vec::new();
        
        // 1. Check time-based messages
        let due: Vec<_> = self.pending
            .range(..=now)
            .flat_map(|(_, msgs)| msgs.iter())
            .cloned()
            .collect();
        
        for scheduled in due {
            let result = self.deliver(scheduled.message).await;
            results.push(result);
        }
        
        // Remove delivered
        self.pending.retain(|t, _| *t > now);
        
        // 2. Check conditional messages
        let mut delivered_conditions = Vec::new();
        for (idx, conditional) in self.conditional.iter().enumerate() {
            if self.evaluate_condition(&conditional.condition).await {
                let result = self.deliver(conditional.message.clone()).await;
                results.push(result);
                delivered_conditions.push(idx);
            }
        }
        
        // Remove delivered conditionals
        for idx in delivered_conditions.into_iter().rev() {
            self.conditional.remove(idx);
        }
        
        results
    }
}
```

---

# SPEC-06: QUERY ENGINE

## 6.1 Query Overview

```rust
/// The communication query engine
pub struct CommQueryEngine {
    /// Message index
    message_index: MessageIndex,
    
    /// Channel index
    channel_index: ChannelIndex,
    
    /// Relationship index
    relationship_index: RelationshipIndex,
    
    /// Semantic search
    semantic_search: SemanticSearchEngine,
    
    /// Temporal queries
    temporal_queries: TemporalQueryEngine,
}

impl CommQueryEngine {
    /// Query messages
    pub async fn query_messages(
        &self,
        query: MessageQuery,
    ) -> Result<Vec<Message>, CommError> {
        match query {
            MessageQuery::ById(id) => {
                self.message_index.get(&id)
            }
            
            MessageQuery::ByChannel(channel_id) => {
                self.message_index.by_channel(&channel_id)
            }
            
            MessageQuery::BySender(sender) => {
                self.message_index.by_sender(&sender)
            }
            
            MessageQuery::ByTimeRange(start, end) => {
                self.message_index.by_time_range(start, end)
            }
            
            MessageQuery::Semantic(text, limit) => {
                self.semantic_search.search(&text, limit).await
            }
            
            MessageQuery::ByAffect(affect_filter) => {
                self.message_index.by_affect(&affect_filter)
            }
            
            MessageQuery::Thread(thread_id) => {
                self.message_index.by_thread(&thread_id)
            }
            
            MessageQuery::Compound(queries) => {
                self.execute_compound(queries).await
            }
        }
    }
    
    /// Query channels
    pub async fn query_channels(
        &self,
        query: ChannelQuery,
    ) -> Result<Vec<Channel>, CommError> {
        match query {
            ChannelQuery::ById(id) => {
                self.channel_index.get(&id)
            }
            
            ChannelQuery::ByParticipant(agent) => {
                self.channel_index.by_participant(&agent)
            }
            
            ChannelQuery::ByType(channel_type) => {
                self.channel_index.by_type(channel_type)
            }
            
            ChannelQuery::Active => {
                self.channel_index.active()
            }
            
            ChannelQuery::WithAgent(agent) => {
                self.channel_index.with_agent(&agent)
            }
        }
    }
    
    /// Query relationships
    pub async fn query_relationships(
        &self,
        query: RelationshipQuery,
    ) -> Result<Vec<Relationship>, CommError> {
        match query {
            RelationshipQuery::Between(a, b) => {
                self.relationship_index.between(&a, &b)
            }
            
            RelationshipQuery::TrustLevel(agent, min_trust) => {
                self.relationship_index.by_trust(&agent, min_trust)
            }
            
            RelationshipQuery::RecentContacts(agent, limit) => {
                self.relationship_index.recent_contacts(&agent, limit)
            }
            
            RelationshipQuery::MostActive(agent, limit) => {
                self.relationship_index.most_active(&agent, limit)
            }
        }
    }
}
```

## 6.2 Query Types

```rust
/// Message query types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageQuery {
    /// By message ID
    ById(MessageId),
    
    /// All messages in channel
    ByChannel(ChannelId),
    
    /// All messages from sender
    BySender(IdentityAnchor),
    
    /// Messages in time range
    ByTimeRange(Timestamp, Timestamp),
    
    /// Semantic search
    Semantic(String, usize),
    
    /// By affect criteria
    ByAffect(AffectFilter),
    
    /// By thread
    Thread(ThreadId),
    
    /// Compound query
    Compound(Vec<MessageQuery>),
}

/// Channel query types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChannelQuery {
    /// By channel ID
    ById(ChannelId),
    
    /// By participant
    ByParticipant(IdentityAnchor),
    
    /// By type
    ByType(ChannelType),
    
    /// All active
    Active,
    
    /// Channels with specific agent
    WithAgent(IdentityAnchor),
}

/// Relationship query types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipQuery {
    /// Between two agents
    Between(IdentityAnchor, IdentityAnchor),
    
    /// By trust level
    TrustLevel(IdentityAnchor, f64),
    
    /// Recent contacts
    RecentContacts(IdentityAnchor, usize),
    
    /// Most active relationships
    MostActive(IdentityAnchor, usize),
}

/// Affect filter for queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectFilter {
    /// Minimum valence
    pub min_valence: Option<f64>,
    
    /// Maximum valence
    pub max_valence: Option<f64>,
    
    /// Minimum arousal
    pub min_arousal: Option<f64>,
    
    /// Required emotions
    pub emotions: Vec<Emotion>,
    
    /// Minimum urgency
    pub min_urgency: Option<UrgencyLevel>,
}
```

## 6.3 Semantic Search

```rust
/// Semantic search engine for communications
pub struct SemanticSearchEngine {
    /// Embedding model
    embedder: EmbeddingModel,
    
    /// Vector index
    vector_index: VectorIndex,
    
    /// Hybrid search (keyword + semantic)
    hybrid: HybridSearcher,
}

impl SemanticSearchEngine {
    /// Search messages semantically
    pub async fn search(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<Message>, CommError> {
        // 1. Generate query embedding
        let query_embedding = self.embedder.embed(query).await?;
        
        // 2. Search vector index
        let vector_results = self.vector_index.search(
            &query_embedding,
            limit * 2, // Get more for re-ranking
        ).await?;
        
        // 3. Keyword search for hybrid
        let keyword_results = self.hybrid.keyword_search(query, limit).await?;
        
        // 4. Merge and re-rank
        let merged = self.hybrid.merge_results(
            vector_results,
            keyword_results,
            limit,
        );
        
        Ok(merged)
    }
    
    /// Find similar messages
    pub async fn find_similar(
        &self,
        message: &Message,
        limit: usize,
    ) -> Result<Vec<Message>, CommError> {
        // 1. Get message embedding
        let embedding = self.get_or_compute_embedding(message).await?;
        
        // 2. Search
        let results = self.vector_index.search(&embedding, limit + 1).await?;
        
        // 3. Filter out the query message itself
        let filtered: Vec<_> = results
            .into_iter()
            .filter(|m| m.id != message.id)
            .take(limit)
            .collect();
        
        Ok(filtered)
    }
}
```

## 6.4 Temporal Queries

```rust
/// Temporal query engine
pub struct TemporalQueryEngine {
    /// Time integration
    time: TimeBridge,
    
    /// Message index
    message_index: MessageIndex,
    
    /// Conversation timeline
    timeline: ConversationTimeline,
}

impl TemporalQueryEngine {
    /// Query conversation at point in time
    pub async fn at_time(
        &self,
        channel: ChannelId,
        timestamp: Timestamp,
    ) -> Result<ConversationSnapshot, CommError> {
        // Get all messages up to timestamp
        let messages = self.message_index.by_channel_before(&channel, timestamp)?;
        
        // Build snapshot
        let snapshot = ConversationSnapshot {
            channel,
            timestamp,
            messages,
            participant_states: self.get_participant_states_at(&channel, timestamp).await?,
        };
        
        Ok(snapshot)
    }
    
    /// Query conversation changes over time range
    pub async fn changes_in_range(
        &self,
        channel: ChannelId,
        start: Timestamp,
        end: Timestamp,
    ) -> Result<ConversationChanges, CommError> {
        let before = self.at_time(channel, start).await?;
        let after = self.at_time(channel, end).await?;
        
        Ok(ConversationChanges {
            channel,
            start,
            end,
            new_messages: after.messages.len() - before.messages.len(),
            participant_changes: self.diff_participants(&before, &after),
            affect_trajectory: self.compute_affect_trajectory(&channel, start, end).await?,
        })
    }
    
    /// Find message echoes (how message propagated)
    pub async fn message_echoes(
        &self,
        message_id: MessageId,
    ) -> Result<EchoChain, CommError> {
        // Find all messages that reference or were influenced by this one
        let echoes = self.trace_message_influence(message_id).await?;
        
        Ok(EchoChain {
            original: message_id,
            echoes,
        })
    }
}
```

---

# SPEC-07: INDEX STRUCTURES

## 7.1 Message Index

```rust
/// Index for fast message retrieval
pub struct MessageIndex {
    /// Primary index by ID
    by_id: HashMap<MessageId, MessageLocation>,
    
    /// By channel
    by_channel: HashMap<ChannelId, Vec<MessageId>>,
    
    /// By sender
    by_sender: HashMap<IdentityAnchor, Vec<MessageId>>,
    
    /// By timestamp (B-tree for range queries)
    by_time: BTreeMap<Timestamp, Vec<MessageId>>,
    
    /// By thread
    by_thread: HashMap<ThreadId, Vec<MessageId>>,
    
    /// By content type
    by_content_type: HashMap<MessageContentType, Vec<MessageId>>,
    
    /// Full-text search index
    text_index: TantivyIndex,
    
    /// Vector embeddings index
    vector_index: HnswIndex,
}

impl MessageIndex {
    /// Add message to index
    pub fn add(&mut self, message: &Message, location: MessageLocation) -> Result<(), CommError> {
        let id = message.id;
        
        // Primary index
        self.by_id.insert(id, location);
        
        // Channel index
        self.by_channel.entry(message.channel)
            .or_default()
            .push(id);
        
        // Sender index
        self.by_sender.entry(message.sender.clone())
            .or_default()
            .push(id);
        
        // Time index
        self.by_time.entry(message.metadata.created_at)
            .or_default()
            .push(id);
        
        // Thread index
        if let Some(thread) = message.metadata.thread {
            self.by_thread.entry(thread)
                .or_default()
                .push(id);
        }
        
        // Content type index
        self.by_content_type.entry(message.content.content_type())
            .or_default()
            .push(id);
        
        // Text index
        if let Some(text) = message.content.as_text() {
            self.text_index.add(id, text)?;
        }
        
        // Vector index (async, may be computed later)
        // self.vector_index.add(id, embedding)?;
        
        Ok(())
    }
    
    /// Get by ID
    pub fn get(&self, id: &MessageId) -> Result<Option<MessageLocation>, CommError> {
        Ok(self.by_id.get(id).copied())
    }
    
    /// Get by channel
    pub fn by_channel(&self, channel: &ChannelId) -> Result<Vec<MessageId>, CommError> {
        Ok(self.by_channel.get(channel).cloned().unwrap_or_default())
    }
    
    /// Get by time range
    pub fn by_time_range(
        &self,
        start: Timestamp,
        end: Timestamp,
    ) -> Result<Vec<MessageId>, CommError> {
        let messages: Vec<_> = self.by_time
            .range(start..=end)
            .flat_map(|(_, ids)| ids.iter())
            .copied()
            .collect();
        
        Ok(messages)
    }
}
```

## 7.2 Channel Index

```rust
/// Index for channel retrieval
pub struct ChannelIndex {
    /// By ID
    by_id: HashMap<ChannelId, ChannelMetadata>,
    
    /// By participant
    by_participant: HashMap<IdentityAnchor, Vec<ChannelId>>,
    
    /// By type
    by_type: HashMap<ChannelType, Vec<ChannelId>>,
    
    /// By state
    by_state: HashMap<ChannelState, Vec<ChannelId>>,
    
    /// By activity (sorted)
    by_activity: BTreeMap<Timestamp, ChannelId>,
}

impl ChannelIndex {
    /// Add channel
    pub fn add(&mut self, channel: &Channel) -> Result<(), CommError> {
        let id = channel.id;
        
        // Primary
        self.by_id.insert(id, ChannelMetadata::from(channel));
        
        // By participant
        for participant in &channel.participants {
            self.by_participant.entry(participant.agent.clone())
                .or_default()
                .push(id);
        }
        
        // By type
        self.by_type.entry(channel.channel_type)
            .or_default()
            .push(id);
        
        // By state
        self.by_state.entry(channel.state)
            .or_default()
            .push(id);
        
        // By activity
        self.by_activity.insert(channel.last_activity, id);
        
        Ok(())
    }
    
    /// Get active channels for agent
    pub fn active_for_agent(&self, agent: &IdentityAnchor) -> Vec<ChannelId> {
        let agent_channels = self.by_participant.get(agent)
            .cloned()
            .unwrap_or_default();
        
        let active = self.by_state.get(&ChannelState::Active)
            .cloned()
            .unwrap_or_default();
        
        // Intersection
        agent_channels.into_iter()
            .filter(|c| active.contains(c))
            .collect()
    }
}
```

## 7.3 Relationship Index

```rust
/// Index for agent relationships
pub struct RelationshipIndex {
    /// Relationships by agent pair
    relationships: HashMap<(IdentityAnchor, IdentityAnchor), Relationship>,
    
    /// Trust levels
    trust_levels: HashMap<IdentityAnchor, HashMap<IdentityAnchor, f64>>,
    
    /// Communication frequency
    frequency: HashMap<IdentityAnchor, HashMap<IdentityAnchor, u64>>,
    
    /// Last contact
    last_contact: HashMap<IdentityAnchor, HashMap<IdentityAnchor, Timestamp>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    /// The two agents
    pub agents: (IdentityAnchor, IdentityAnchor),
    
    /// Trust level (mutual)
    pub trust: f64,
    
    /// Communication count
    pub message_count: u64,
    
    /// First contact
    pub first_contact: Timestamp,
    
    /// Last contact
    pub last_contact: Timestamp,
    
    /// Shared channels
    pub shared_channels: Vec<ChannelId>,
    
    /// Relationship type
    pub relationship_type: RelationshipType,
    
    /// Affect history summary
    pub affect_summary: AffectSummary,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RelationshipType {
    /// Never communicated
    None,
    
    /// Single interaction
    OneTime,
    
    /// Occasional
    Occasional,
    
    /// Regular
    Regular,
    
    /// Frequent
    Frequent,
    
    /// Hive/meld history
    Deep,
}

impl RelationshipIndex {
    /// Update relationship on communication
    pub fn record_communication(
        &mut self,
        from: &IdentityAnchor,
        to: &IdentityAnchor,
        message: &Message,
    ) {
        let key = self.normalize_key(from, to);
        
        let relationship = self.relationships.entry(key.clone())
            .or_insert_with(|| Relationship::new(from, to));
        
        relationship.message_count += 1;
        relationship.last_contact = Timestamp::now();
        
        // Update frequency
        self.frequency.entry(from.clone())
            .or_default()
            .entry(to.clone())
            .and_modify(|c| *c += 1)
            .or_insert(1);
        
        // Update last contact
        self.last_contact.entry(from.clone())
            .or_default()
            .insert(to.clone(), Timestamp::now());
        
        // Update relationship type based on frequency
        relationship.relationship_type = self.compute_relationship_type(relationship);
    }
    
    /// Get relationship between agents
    pub fn between(
        &self,
        a: &IdentityAnchor,
        b: &IdentityAnchor,
    ) -> Option<&Relationship> {
        let key = self.normalize_key(a, b);
        self.relationships.get(&key)
    }
    
    fn normalize_key(
        &self,
        a: &IdentityAnchor,
        b: &IdentityAnchor,
    ) -> (IdentityAnchor, IdentityAnchor) {
        if a < b { (a.clone(), b.clone()) } else { (b.clone(), a.clone()) }
    }
}
```

## 7.4 Vector Index

```rust
/// HNSW vector index for semantic search
pub struct VectorIndex {
    /// The HNSW graph
    hnsw: HnswGraph<f32>,
    
    /// Message ID to vector ID mapping
    id_map: BiMap<MessageId, usize>,
    
    /// Embedding dimension
    dimension: usize,
    
    /// Index parameters
    params: HnswParams,
}

#[derive(Debug, Clone)]
pub struct HnswParams {
    /// Max connections per node
    pub m: usize,
    
    /// Construction search depth
    pub ef_construction: usize,
    
    /// Search depth
    pub ef_search: usize,
}

impl Default for HnswParams {
    fn default() -> Self {
        Self {
            m: 16,
            ef_construction: 200,
            ef_search: 100,
        }
    }
}

impl VectorIndex {
    /// Add embedding
    pub fn add(&mut self, message_id: MessageId, embedding: &[f32]) -> Result<(), CommError> {
        let vector_id = self.hnsw.insert(embedding)?;
        self.id_map.insert(message_id, vector_id);
        Ok(())
    }
    
    /// Search nearest neighbors
    pub fn search(&self, query: &[f32], k: usize) -> Result<Vec<(MessageId, f32)>, CommError> {
        let results = self.hnsw.search(query, k)?;
        
        let mapped: Vec<_> = results.into_iter()
            .filter_map(|(vector_id, distance)| {
                self.id_map.get_by_right(&vector_id)
                    .map(|msg_id| (*msg_id, distance))
            })
            .collect();
        
        Ok(mapped)
    }
}
```

---

# SPEC-08: VALIDATION

## 8.1 Message Validation

```rust
/// Message validator
pub struct MessageValidator {
    /// Content validators
    content_validators: Vec<Box<dyn ContentValidator>>,
    
    /// Policy checker
    policy_checker: PolicyChecker,
    
    /// Rate limiter
    rate_limiter: RateLimiter,
}

impl MessageValidator {
    /// Validate outgoing message
    pub fn validate_outgoing(
        &self,
        message: &Message,
        channel: &Channel,
    ) -> Result<ValidationResult, CommError> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // 1. Content validation
        for validator in &self.content_validators {
            match validator.validate(&message.content) {
                Ok(()) => {}
                Err(ValidationError::Error(e)) => errors.push(e),
                Err(ValidationError::Warning(w)) => warnings.push(w),
            }
        }
        
        // 2. Policy validation
        if let Err(e) = self.policy_checker.check_send(message, channel) {
            errors.push(e.to_string());
        }
        
        // 3. Rate limiting
        if let Err(e) = self.rate_limiter.check(&message.sender) {
            errors.push(e.to_string());
        }
        
        // 4. Recipient validation
        for recipient in &message.recipients {
            if let Err(e) = self.validate_recipient(recipient, channel) {
                errors.push(e.to_string());
            }
        }
        
        // 5. Signature validation
        if message.signature.is_empty() {
            errors.push("Message must be signed".into());
        }
        
        if errors.is_empty() {
            Ok(ValidationResult::Valid { warnings })
        } else {
            Ok(ValidationResult::Invalid { errors, warnings })
        }
    }
    
    /// Validate incoming message
    pub fn validate_incoming(
        &self,
        message: &Message,
        channel: &Channel,
    ) -> Result<ValidationResult, CommError> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // 1. Signature verification (already done in engine, but double-check)
        if message.signature.is_empty() {
            errors.push("Message has no signature".into());
        }
        
        // 2. Sender is participant
        if !channel.has_participant(&message.sender) {
            errors.push("Sender is not channel participant".into());
        }
        
        // 3. Content type allowed
        if !channel.allows_content_type(&message.content) {
            errors.push("Content type not allowed in channel".into());
        }
        
        // 4. Timestamp sanity
        let now = Timestamp::now();
        if message.metadata.created_at > now + Duration::from_secs(60) {
            warnings.push("Message timestamp is in the future".into());
        }
        
        // 5. Not expired
        if let Some(expires) = message.metadata.expires_at {
            if expires < now {
                errors.push("Message has expired".into());
            }
        }
        
        if errors.is_empty() {
            Ok(ValidationResult::Valid { warnings })
        } else {
            Ok(ValidationResult::Invalid { errors, warnings })
        }
    }
}

#[derive(Debug)]
pub enum ValidationResult {
    Valid { warnings: Vec<String> },
    Invalid { errors: Vec<String>, warnings: Vec<String> },
}
```

## 8.2 Channel Validation

```rust
/// Channel validator
pub struct ChannelValidator {
    /// Participant limits
    participant_limits: ParticipantLimits,
    
    /// Channel type rules
    type_rules: HashMap<ChannelType, ChannelTypeRules>,
}

impl ChannelValidator {
    /// Validate channel creation
    pub fn validate_creation(
        &self,
        channel_type: ChannelType,
        participants: &[IdentityAnchor],
        config: &ChannelConfig,
    ) -> Result<(), CommError> {
        let rules = self.type_rules.get(&channel_type)
            .ok_or(CommError::UnsupportedChannelType)?;
        
        // Check participant count
        if participants.len() < rules.min_participants {
            return Err(CommError::TooFewParticipants);
        }
        
        if participants.len() > rules.max_participants {
            return Err(CommError::TooManyParticipants);
        }
        
        // Check for duplicates
        let unique: HashSet<_> = participants.iter().collect();
        if unique.len() != participants.len() {
            return Err(CommError::DuplicateParticipants);
        }
        
        // Type-specific validation
        match channel_type {
            ChannelType::Direct => {
                if participants.len() != 2 {
                    return Err(CommError::DirectChannelMustHaveTwoParticipants);
                }
            }
            ChannelType::Hive => {
                if participants.len() < 2 {
                    return Err(CommError::HiveRequiresMultipleAgents);
                }
            }
            _ => {}
        }
        
        Ok(())
    }
}
```

## 8.3 Consent Validation

```rust
/// Consent validator
pub struct ConsentValidator {
    /// Consent store
    consent_store: ConsentStore,
    
    /// Contract integration
    contract: ContractBridge,
}

impl ConsentValidator {
    /// Check if agent has consented to message type
    pub async fn check_consent(
        &self,
        recipient: &IdentityAnchor,
        content_type: &MessageContentType,
        sender: &IdentityAnchor,
    ) -> Result<ConsentStatus, CommError> {
        // 1. Check explicit consent
        let explicit = self.consent_store.get_consent(
            recipient,
            sender,
            content_type,
        )?;
        
        if let Some(consent) = explicit {
            return Ok(consent.status);
        }
        
        // 2. Check contract policies
        let policy_consent = self.contract.check_communication_consent(
            recipient,
            sender,
            content_type,
        ).await?;
        
        if policy_consent != ConsentStatus::Pending {
            return Ok(policy_consent);
        }
        
        // 3. Check default settings
        let defaults = self.consent_store.get_defaults(recipient)?;
        
        Ok(defaults.status_for(content_type))
    }
    
    /// Validate consent for special content types
    pub async fn validate_special_consent(
        &self,
        recipient: &IdentityAnchor,
        content: &MessageContent,
        sender: &IdentityAnchor,
    ) -> Result<(), CommError> {
        match content {
            MessageContent::Semantic(_) => {
                self.require_consent(recipient, sender, ConsentScope::ReceiveSemantic).await?;
            }
            MessageContent::Affect(_) => {
                self.require_consent(recipient, sender, ConsentScope::ReceiveAffect).await?;
            }
            MessageContent::Unspeakable(_) => {
                // Unspeakable requires explicit high-trust consent
                self.require_high_trust_consent(recipient, sender).await?;
            }
            _ => {}
        }
        
        Ok(())
    }
}
```

## 8.4 Input Validation (MCP Hardening)

```rust
/// MCP input validator - strict validation, no silent fallbacks
pub struct McpInputValidator;

impl McpInputValidator {
    /// Validate channel_create parameters
    pub fn validate_channel_create(params: &Value) -> Result<ChannelCreateParams, CommError> {
        let channel_type = params.get("channel_type")
            .and_then(|v| v.as_str())
            .ok_or(CommError::MissingParameter("channel_type"))?;
        
        let channel_type = ChannelType::from_str(channel_type)
            .map_err(|_| CommError::InvalidParameter("channel_type", "unknown channel type"))?;
        
        let participants = params.get("participants")
            .and_then(|v| v.as_array())
            .ok_or(CommError::MissingParameter("participants"))?;
        
        if participants.is_empty() {
            return Err(CommError::InvalidParameter("participants", "cannot be empty"));
        }
        
        let participants: Result<Vec<_>, _> = participants.iter()
            .map(|p| {
                p.as_str()
                    .ok_or(CommError::InvalidParameter("participants", "must be strings"))
                    .and_then(|s| IdentityAnchor::from_str(s)
                        .map_err(|_| CommError::InvalidParameter("participants", "invalid identity")))
            })
            .collect();
        
        Ok(ChannelCreateParams {
            channel_type,
            participants: participants?,
            config: Self::parse_config(params.get("config"))?,
        })
    }
    
    /// Validate message_send parameters
    pub fn validate_message_send(params: &Value) -> Result<MessageSendParams, CommError> {
        let channel_id = params.get("channel_id")
            .and_then(|v| v.as_str())
            .ok_or(CommError::MissingParameter("channel_id"))?;
        
        let channel_id = ChannelId::from_str(channel_id)
            .map_err(|_| CommError::InvalidParameter("channel_id", "invalid UUID"))?;
        
        let content = params.get("content")
            .ok_or(CommError::MissingParameter("content"))?;
        
        let content = Self::parse_content(content)?;
        
        Ok(MessageSendParams {
            channel_id,
            content,
            options: Self::parse_message_options(params.get("options"))?,
        })
    }
    
    // NO SILENT FALLBACKS - all errors are explicit
    fn parse_content(value: &Value) -> Result<MessageContent, CommError> {
        let content_type = value.get("type")
            .and_then(|v| v.as_str())
            .ok_or(CommError::MissingParameter("content.type"))?;
        
        match content_type {
            "text" => {
                let text = value.get("text")
                    .and_then(|v| v.as_str())
                    .ok_or(CommError::MissingParameter("content.text"))?;
                
                Ok(MessageContent::Text(TextMessage {
                    text: text.to_string(),
                    language: value.get("language").and_then(|v| v.as_str()).map(String::from),
                    formatting: None,
                }))
            }
            "semantic" => {
                // Strict parsing for semantic content
                Self::parse_semantic_content(value)
            }
            "affect" => {
                Self::parse_affect_content(value)
            }
            _ => Err(CommError::InvalidParameter("content.type", "unknown content type")),
        }
    }
}
```

---

*End of Part 2. Continued in Part 3: CLI, MCP Server, Sister Integration, Tests*
