//! CLI for agentic-comm: agent communication, channels, pub/sub.

use std::path::PathBuf;

use agentic_comm::{
    ChannelType, CollectiveDecisionMode, CommStore, CommTrustLevel, ConsentScope, FederatedZone,
    FederationPolicy, HiveRole, MessageFilter, MessageType, TemporalTarget,
};
use clap::{Parser, Subcommand};

/// Default store file path.
fn default_store_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".store.acomm")
}

/// Resolve the store path from CLI arg, env var, or defaults.
fn resolve_store_path(path: Option<PathBuf>) -> PathBuf {
    if let Some(p) = path {
        return p;
    }
    if let Ok(p) = std::env::var("ACOMM_STORE") {
        return PathBuf::from(p);
    }
    let local = PathBuf::from(".acomm/store.acomm");
    if local.exists() {
        return local;
    }
    default_store_path()
}

/// Load the store from disk or create a new one.
fn load_or_create(path: &PathBuf) -> CommStore {
    if path.exists() {
        CommStore::load(path).unwrap_or_else(|e| {
            eprintln!("Warning: could not load {}: {e}", path.display());
            CommStore::new()
        })
    } else {
        CommStore::new()
    }
}

/// acomm — Agent communication CLI
#[derive(Parser)]
#[command(name = "acomm", version, about = "Agent communication CLI for channels, messaging, and pub/sub")]
struct Cli {
    /// Path to the .acomm store file
    #[arg(long, global = true)]
    file: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Send a message to a channel
    Send {
        /// Channel ID
        channel: u64,
        /// Message content
        content: String,
        /// Message type
        #[arg(long, default_value = "text")]
        r#type: String,
        /// Sender name
        #[arg(long, default_value = "cli-user")]
        sender: String,
    },
    /// Receive messages from a channel
    Receive {
        /// Channel ID
        channel: u64,
        /// Only messages since this ISO 8601 timestamp
        #[arg(long)]
        since: Option<String>,
        /// Filter by recipient
        #[arg(long)]
        recipient: Option<String>,
    },
    /// Channel management
    Channel {
        #[command(subcommand)]
        action: ChannelAction,
    },
    /// Subscribe to a pub/sub topic
    Subscribe {
        /// Topic name
        topic: String,
        /// Subscriber name
        subscriber: String,
    },
    /// Publish a message to a pub/sub topic
    Publish {
        /// Topic name
        topic: String,
        /// Message content
        content: String,
        /// Sender name
        #[arg(long, default_value = "cli-user")]
        sender: String,
    },
    /// Query message history for a channel
    History {
        /// Channel ID
        channel: u64,
        /// Text search query
        #[arg(long)]
        query: Option<String>,
        /// Maximum results
        #[arg(long, default_value = "50")]
        limit: usize,
    },
    /// Show .acomm file statistics
    Info {
        /// Path to the .acomm file
        file: PathBuf,
    },
    /// Add a communication record (hook-compatible interface)
    Add {
        /// Path to the .acomm file
        file: PathBuf,
        /// Message type (text, command, query, etc.)
        r#type: String,
        /// Message content
        content: String,
        /// Channel name (auto-created if needed)
        #[arg(long, default_value = "default")]
        channel: String,
        /// Session identifier
        #[arg(long)]
        session: Option<String>,
        /// Confidence level (unused, for hook compatibility)
        #[arg(long)]
        confidence: Option<f64>,
    },
    /// Consent management (grant, revoke, check, list)
    Consent {
        #[command(subcommand)]
        action: ConsentAction,
    },
    /// Trust level management (set, get, list)
    Trust {
        #[command(subcommand)]
        action: TrustAction,
    },
    /// Hive mind management (form, dissolve, join, leave, list, info)
    Hive {
        #[command(subcommand)]
        action: HiveAction,
    },
    /// Federation management (configure, add-zone, remove-zone, list)
    Federation {
        #[command(subcommand)]
        action: FederationAction,
    },
    /// Temporal message scheduling (schedule, list, cancel, deliver)
    Temporal {
        #[command(subcommand)]
        action: TemporalAction,
    },
    /// Show comprehensive store statistics
    Status,
    /// Ground a claim against the communication store
    Ground {
        /// The claim to verify
        claim: String,
    },
}

#[derive(Subcommand)]
enum ChannelAction {
    /// Create a new channel
    Create {
        /// Channel name
        name: String,
        /// Channel type: direct, group, broadcast, pubsub
        #[arg(long, default_value = "group")]
        r#type: String,
    },
    /// List all channels
    List,
    /// Show channel information
    Info {
        /// Channel ID
        channel_id: u64,
    },
}

#[derive(Subcommand)]
enum ConsentAction {
    /// Grant consent from one agent to another
    Grant {
        /// Granting agent ID
        grantor: String,
        /// Receiving agent ID
        grantee: String,
        /// Consent scope (read_messages, send_messages, join_channels, view_presence, share_content, schedule_messages, federate, hive_participation)
        scope: String,
        /// Optional reason for granting
        #[arg(long)]
        reason: Option<String>,
        /// Optional expiry time (ISO 8601)
        #[arg(long)]
        expires_at: Option<String>,
    },
    /// Revoke consent from one agent to another
    Revoke {
        /// Granting agent ID
        grantor: String,
        /// Receiving agent ID
        grantee: String,
        /// Consent scope to revoke
        scope: String,
    },
    /// Check if consent is granted between two agents
    Check {
        /// Granting agent ID
        grantor: String,
        /// Receiving agent ID
        grantee: String,
        /// Consent scope to check
        scope: String,
    },
    /// List consent gates
    List {
        /// Filter by agent ID
        #[arg(long)]
        agent: Option<String>,
    },
}

#[derive(Subcommand)]
enum TrustAction {
    /// Set trust level for an agent
    Set {
        /// Agent ID
        agent_id: String,
        /// Trust level (none, minimal, basic, standard, high, full, absolute)
        level: String,
    },
    /// Get trust level for an agent
    Get {
        /// Agent ID
        agent_id: String,
    },
    /// List all trust level overrides
    List,
}

#[derive(Subcommand)]
enum HiveAction {
    /// Form a new hive mind
    Form {
        /// Hive name
        name: String,
        /// Coordinator agent ID
        coordinator: String,
        /// Decision mode (coordinator_decides, majority, unanimous, consensus)
        #[arg(long, default_value = "coordinator_decides")]
        mode: String,
    },
    /// Dissolve a hive mind
    Dissolve {
        /// Hive ID
        hive_id: u64,
    },
    /// Join a hive mind
    Join {
        /// Hive ID
        hive_id: u64,
        /// Agent ID
        agent_id: String,
        /// Role (coordinator, member, observer)
        #[arg(long, default_value = "member")]
        role: String,
    },
    /// Leave a hive mind
    Leave {
        /// Hive ID
        hive_id: u64,
        /// Agent ID
        agent_id: String,
    },
    /// List all hive minds
    List,
    /// Show hive mind information
    Info {
        /// Hive ID
        hive_id: u64,
    },
}

#[derive(Subcommand)]
enum FederationAction {
    /// Configure federation settings
    Configure {
        /// Enable federation
        #[arg(long)]
        enable: bool,
        /// Disable federation
        #[arg(long)]
        disable: bool,
        /// Local zone identifier
        #[arg(long, default_value = "local")]
        zone: String,
        /// Default policy (allow, deny, selective)
        #[arg(long, default_value = "deny")]
        policy: String,
    },
    /// Add a federated zone
    AddZone {
        /// Zone identifier
        zone_id: String,
        /// Human-readable zone name
        #[arg(long, default_value = "")]
        name: String,
        /// Endpoint URL or address
        #[arg(long, default_value = "")]
        endpoint: String,
        /// Policy for this zone (allow, deny, selective)
        #[arg(long, default_value = "deny")]
        policy: String,
        /// Trust level for this zone (none, minimal, basic, standard, high, full, absolute)
        #[arg(long, default_value = "standard")]
        trust: String,
    },
    /// Remove a federated zone
    RemoveZone {
        /// Zone identifier
        zone_id: String,
    },
    /// List all federated zones
    List,
}

#[derive(Subcommand)]
enum TemporalAction {
    /// Schedule a message for future delivery
    Schedule {
        /// Target channel ID
        channel: u64,
        /// Sender name
        sender: String,
        /// Message content
        content: String,
        /// Delay in seconds from now
        #[arg(long)]
        delay: Option<u64>,
        /// Absolute delivery time (ISO 8601)
        #[arg(long)]
        deliver_at: Option<String>,
    },
    /// List all scheduled (pending) temporal messages
    List,
    /// Cancel a scheduled message
    Cancel {
        /// Temporal message ID
        temporal_id: u64,
    },
    /// Deliver all pending temporal messages that are due
    Deliver,
}

/// Parse a string into a ConsentScope, exiting on failure.
fn parse_consent_scope(s: &str) -> ConsentScope {
    s.parse().unwrap_or_else(|e: String| {
        eprintln!("Invalid consent scope: {e}");
        std::process::exit(1);
    })
}

/// Parse a string into a CommTrustLevel, exiting on failure.
fn parse_trust_level(s: &str) -> CommTrustLevel {
    s.parse().unwrap_or_else(|e: String| {
        eprintln!("Invalid trust level: {e}");
        std::process::exit(1);
    })
}

/// Parse a string into a FederationPolicy, exiting on failure.
fn parse_federation_policy(s: &str) -> FederationPolicy {
    s.parse().unwrap_or_else(|e: String| {
        eprintln!("Invalid federation policy: {e}");
        std::process::exit(1);
    })
}

/// Parse a string into a HiveRole, exiting on failure.
fn parse_hive_role(s: &str) -> HiveRole {
    match s.to_lowercase().as_str() {
        "coordinator" => HiveRole::Coordinator,
        "member" => HiveRole::Member,
        "observer" => HiveRole::Observer,
        other => {
            eprintln!("Invalid hive role: {other} (expected coordinator, member, or observer)");
            std::process::exit(1);
        }
    }
}

/// Parse a string into a CollectiveDecisionMode, exiting on failure.
fn parse_decision_mode(s: &str) -> CollectiveDecisionMode {
    match s.to_lowercase().as_str() {
        "coordinator_decides" => CollectiveDecisionMode::CoordinatorDecides,
        "majority" => CollectiveDecisionMode::Majority,
        "unanimous" => CollectiveDecisionMode::Unanimous,
        "consensus" => CollectiveDecisionMode::Consensus,
        other => {
            eprintln!("Invalid decision mode: {other} (expected coordinator_decides, majority, unanimous, or consensus)");
            std::process::exit(1);
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let store_path = resolve_store_path(cli.file);

    match cli.command {
        Commands::Send {
            channel,
            content,
            r#type,
            sender,
        } => {
            let msg_type: MessageType = r#type
                .parse()
                .unwrap_or_else(|e| {
                    eprintln!("Invalid message type: {e}");
                    std::process::exit(1);
                });
            let mut store = load_or_create(&store_path);
            match store.send_message(channel, &sender, &content, msg_type) {
                Ok(msg) => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "sent",
                            "message_id": msg.id,
                            "channel_id": msg.channel_id,
                            "timestamp": msg.timestamp.to_rfc3339(),
                        }))
                        .unwrap()
                    );
                    if let Err(e) = store.save(&store_path) {
                        eprintln!("Warning: failed to save store: {e}");
                    }
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::Receive {
            channel,
            since,
            recipient,
        } => {
            let store = load_or_create(&store_path);
            let since_dt = since.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|dt| dt.with_timezone(&chrono::Utc))
            });
            match store.receive_messages(channel, recipient.as_deref(), since_dt) {
                Ok(msgs) => {
                    println!("{}", serde_json::to_string_pretty(&msgs).unwrap());
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::Channel { action } => match action {
            ChannelAction::Create { name, r#type } => {
                let ch_type: ChannelType = r#type
                    .parse()
                    .unwrap_or_else(|e| {
                        eprintln!("Invalid channel type: {e}");
                        std::process::exit(1);
                    });
                let mut store = load_or_create(&store_path);
                match store.create_channel(&name, ch_type, None) {
                    Ok(ch) => {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "status": "created",
                                "channel_id": ch.id,
                                "name": ch.name,
                                "type": ch.channel_type.to_string(),
                            }))
                            .unwrap()
                        );
                        if let Err(e) = store.save(&store_path) {
                            eprintln!("Warning: failed to save store: {e}");
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {e}");
                        std::process::exit(1);
                    }
                }
            }
            ChannelAction::List => {
                let store = load_or_create(&store_path);
                let channels = store.list_channels();
                println!("{}", serde_json::to_string_pretty(&channels).unwrap());
            }
            ChannelAction::Info { channel_id } => {
                let store = load_or_create(&store_path);
                match store.get_channel(channel_id) {
                    Some(ch) => {
                        println!("{}", serde_json::to_string_pretty(&ch).unwrap());
                    }
                    None => {
                        eprintln!("Channel {channel_id} not found");
                        std::process::exit(1);
                    }
                }
            }
        },

        Commands::Subscribe { topic, subscriber } => {
            let mut store = load_or_create(&store_path);
            match store.subscribe(&topic, &subscriber) {
                Ok(sub) => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "subscribed",
                            "subscription_id": sub.id,
                            "topic": sub.topic,
                            "subscriber": sub.subscriber,
                        }))
                        .unwrap()
                    );
                    if let Err(e) = store.save(&store_path) {
                        eprintln!("Warning: failed to save store: {e}");
                    }
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::Publish {
            topic,
            content,
            sender,
        } => {
            let mut store = load_or_create(&store_path);
            match store.publish(&topic, &sender, &content) {
                Ok(msgs) => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "published",
                            "delivered_count": msgs.len(),
                            "topic": topic,
                        }))
                        .unwrap()
                    );
                    if let Err(e) = store.save(&store_path) {
                        eprintln!("Warning: failed to save store: {e}");
                    }
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::History {
            channel,
            query,
            limit,
        } => {
            let store = load_or_create(&store_path);
            if let Some(q) = query {
                let results = store.search_messages(&q, limit);
                println!("{}", serde_json::to_string_pretty(&results).unwrap());
            } else {
                let filter = MessageFilter {
                    limit: Some(limit),
                    ..Default::default()
                };
                let results = store.query_history(channel, &filter);
                println!("{}", serde_json::to_string_pretty(&results).unwrap());
            }
        }

        Commands::Info { file } => {
            match CommStore::load(&file) {
                Ok(store) => {
                    let stats = store.stats();
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "file": file.display().to_string(),
                            "channels": stats.channel_count,
                            "messages": stats.message_count,
                            "subscriptions": stats.subscription_count,
                            "total_participants": stats.total_participants,
                        }))
                        .unwrap()
                    );
                }
                Err(e) => {
                    eprintln!("Error reading {}: {e}", file.display());
                    std::process::exit(1);
                }
            }
        }

        Commands::Add {
            file,
            r#type,
            content,
            channel,
            session: _session,
            confidence: _confidence,
        } => {
            let msg_type: MessageType = r#type
                .parse()
                .unwrap_or_else(|e| {
                    eprintln!("Invalid message type: {e}");
                    std::process::exit(1);
                });
            let mut store = if file.exists() {
                CommStore::load(&file).unwrap_or_else(|_| CommStore::new())
            } else {
                CommStore::new()
            };

            // Auto-create channel if needed
            let channel_id = store
                .list_channels()
                .iter()
                .find(|c| c.name == channel)
                .map(|c| c.id)
                .unwrap_or_else(|| {
                    store
                        .create_channel(&channel, ChannelType::Group, None)
                        .expect("Failed to create channel")
                        .id
                });

            match store.send_message(channel_id, "cli-hook", &content, msg_type) {
                Ok(msg) => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "added",
                            "message_id": msg.id,
                            "channel": channel,
                            "file": file.display().to_string(),
                        }))
                        .unwrap()
                    );
                    if let Err(e) = store.save(&file) {
                        eprintln!("Warning: failed to save: {e}");
                    }
                }
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }

        // -----------------------------------------------------------------
        // Consent management
        // -----------------------------------------------------------------
        Commands::Consent { action } => match action {
            ConsentAction::Grant {
                grantor,
                grantee,
                scope,
                reason,
                expires_at,
            } => {
                let scope = parse_consent_scope(&scope);
                let mut store = load_or_create(&store_path);
                match store.grant_consent(&grantor, &grantee, scope, reason, expires_at) {
                    Ok(entry) => {
                        println!("{}", serde_json::to_string_pretty(&entry).unwrap());
                        if let Err(e) = store.save(&store_path) {
                            eprintln!("Warning: failed to save store: {e}");
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {e}");
                        std::process::exit(1);
                    }
                }
            }
            ConsentAction::Revoke {
                grantor,
                grantee,
                scope,
            } => {
                let scope = parse_consent_scope(&scope);
                let mut store = load_or_create(&store_path);
                match store.revoke_consent(&grantor, &grantee, &scope) {
                    Ok(()) => {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "status": "revoked",
                                "grantor": grantor,
                                "grantee": grantee,
                                "scope": scope.to_string(),
                            }))
                            .unwrap()
                        );
                        if let Err(e) = store.save(&store_path) {
                            eprintln!("Warning: failed to save store: {e}");
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {e}");
                        std::process::exit(1);
                    }
                }
            }
            ConsentAction::Check {
                grantor,
                grantee,
                scope,
            } => {
                let scope = parse_consent_scope(&scope);
                let store = load_or_create(&store_path);
                let granted = store.check_consent(&grantor, &grantee, &scope);
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "grantor": grantor,
                        "grantee": grantee,
                        "scope": scope.to_string(),
                        "granted": granted,
                    }))
                    .unwrap()
                );
            }
            ConsentAction::List { agent } => {
                let store = load_or_create(&store_path);
                let gates = store.list_consent_gates(agent.as_deref());
                println!("{}", serde_json::to_string_pretty(&gates).unwrap());
            }
        },

        // -----------------------------------------------------------------
        // Trust management
        // -----------------------------------------------------------------
        Commands::Trust { action } => match action {
            TrustAction::Set { agent_id, level } => {
                let level = parse_trust_level(&level);
                let mut store = load_or_create(&store_path);
                match store.set_trust_level(&agent_id, level) {
                    Ok(()) => {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "status": "set",
                                "agent_id": agent_id,
                                "level": level.to_string(),
                            }))
                            .unwrap()
                        );
                        if let Err(e) = store.save(&store_path) {
                            eprintln!("Warning: failed to save store: {e}");
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {e}");
                        std::process::exit(1);
                    }
                }
            }
            TrustAction::Get { agent_id } => {
                let store = load_or_create(&store_path);
                let level = store.get_trust_level(&agent_id);
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "agent_id": agent_id,
                        "level": level.to_string(),
                    }))
                    .unwrap()
                );
            }
            TrustAction::List => {
                let store = load_or_create(&store_path);
                let levels = store.list_trust_levels();
                let display: std::collections::HashMap<&String, String> =
                    levels.iter().map(|(k, v)| (k, v.to_string())).collect();
                println!("{}", serde_json::to_string_pretty(&display).unwrap());
            }
        },

        // -----------------------------------------------------------------
        // Hive mind management
        // -----------------------------------------------------------------
        Commands::Hive { action } => match action {
            HiveAction::Form {
                name,
                coordinator,
                mode,
            } => {
                let decision_mode = parse_decision_mode(&mode);
                let mut store = load_or_create(&store_path);
                match store.form_hive(&name, &coordinator, decision_mode) {
                    Ok(hive) => {
                        println!("{}", serde_json::to_string_pretty(&hive).unwrap());
                        if let Err(e) = store.save(&store_path) {
                            eprintln!("Warning: failed to save store: {e}");
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {e}");
                        std::process::exit(1);
                    }
                }
            }
            HiveAction::Dissolve { hive_id } => {
                let mut store = load_or_create(&store_path);
                match store.dissolve_hive(hive_id) {
                    Ok(()) => {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "status": "dissolved",
                                "hive_id": hive_id,
                            }))
                            .unwrap()
                        );
                        if let Err(e) = store.save(&store_path) {
                            eprintln!("Warning: failed to save store: {e}");
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {e}");
                        std::process::exit(1);
                    }
                }
            }
            HiveAction::Join {
                hive_id,
                agent_id,
                role,
            } => {
                let role = parse_hive_role(&role);
                let mut store = load_or_create(&store_path);
                match store.join_hive(hive_id, &agent_id, role) {
                    Ok(()) => {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "status": "joined",
                                "hive_id": hive_id,
                                "agent_id": agent_id,
                                "role": role.to_string(),
                            }))
                            .unwrap()
                        );
                        if let Err(e) = store.save(&store_path) {
                            eprintln!("Warning: failed to save store: {e}");
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {e}");
                        std::process::exit(1);
                    }
                }
            }
            HiveAction::Leave { hive_id, agent_id } => {
                let mut store = load_or_create(&store_path);
                match store.leave_hive(hive_id, &agent_id) {
                    Ok(()) => {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "status": "left",
                                "hive_id": hive_id,
                                "agent_id": agent_id,
                            }))
                            .unwrap()
                        );
                        if let Err(e) = store.save(&store_path) {
                            eprintln!("Warning: failed to save store: {e}");
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {e}");
                        std::process::exit(1);
                    }
                }
            }
            HiveAction::List => {
                let store = load_or_create(&store_path);
                let hives = store.list_hives();
                println!("{}", serde_json::to_string_pretty(&hives).unwrap());
            }
            HiveAction::Info { hive_id } => {
                let store = load_or_create(&store_path);
                match store.get_hive(hive_id) {
                    Some(hive) => {
                        println!("{}", serde_json::to_string_pretty(&hive).unwrap());
                    }
                    None => {
                        eprintln!("Hive {hive_id} not found");
                        std::process::exit(1);
                    }
                }
            }
        },

        // -----------------------------------------------------------------
        // Federation management
        // -----------------------------------------------------------------
        Commands::Federation { action } => match action {
            FederationAction::Configure {
                enable,
                disable,
                zone,
                policy,
            } => {
                let enabled = if disable { false } else { enable };
                let policy = parse_federation_policy(&policy);
                let mut store = load_or_create(&store_path);
                match store.configure_federation(enabled, &zone, policy) {
                    Ok(()) => {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "status": "configured",
                                "enabled": enabled,
                                "local_zone": zone,
                                "default_policy": policy.to_string(),
                            }))
                            .unwrap()
                        );
                        if let Err(e) = store.save(&store_path) {
                            eprintln!("Warning: failed to save store: {e}");
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {e}");
                        std::process::exit(1);
                    }
                }
            }
            FederationAction::AddZone {
                zone_id,
                name,
                endpoint,
                policy,
                trust,
            } => {
                let policy = parse_federation_policy(&policy);
                let trust_level = parse_trust_level(&trust);
                let zone = FederatedZone {
                    zone_id: zone_id.clone(),
                    name,
                    endpoint,
                    policy,
                    trust_level,
                };
                let mut store = load_or_create(&store_path);
                match store.add_federated_zone(zone) {
                    Ok(()) => {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "status": "added",
                                "zone_id": zone_id,
                            }))
                            .unwrap()
                        );
                        if let Err(e) = store.save(&store_path) {
                            eprintln!("Warning: failed to save store: {e}");
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {e}");
                        std::process::exit(1);
                    }
                }
            }
            FederationAction::RemoveZone { zone_id } => {
                let mut store = load_or_create(&store_path);
                match store.remove_federated_zone(&zone_id) {
                    Ok(()) => {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "status": "removed",
                                "zone_id": zone_id,
                            }))
                            .unwrap()
                        );
                        if let Err(e) = store.save(&store_path) {
                            eprintln!("Warning: failed to save store: {e}");
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {e}");
                        std::process::exit(1);
                    }
                }
            }
            FederationAction::List => {
                let store = load_or_create(&store_path);
                let config = store.get_federation_config();
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "enabled": config.enabled,
                        "local_zone": config.local_zone,
                        "default_policy": config.default_policy.to_string(),
                        "zones": config.zones,
                    }))
                    .unwrap()
                );
            }
        },

        // -----------------------------------------------------------------
        // Temporal message scheduling
        // -----------------------------------------------------------------
        Commands::Temporal { action } => match action {
            TemporalAction::Schedule {
                channel,
                sender,
                content,
                delay,
                deliver_at,
            } => {
                let target = if let Some(secs) = delay {
                    TemporalTarget::FutureRelative {
                        delay_seconds: secs,
                    }
                } else if let Some(dt) = deliver_at {
                    TemporalTarget::FutureAbsolute { deliver_at: dt }
                } else {
                    TemporalTarget::Immediate
                };
                let mut store = load_or_create(&store_path);
                match store.schedule_message(channel, &sender, &content, target, None) {
                    Ok(msg) => {
                        println!("{}", serde_json::to_string_pretty(&msg).unwrap());
                        if let Err(e) = store.save(&store_path) {
                            eprintln!("Warning: failed to save store: {e}");
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {e}");
                        std::process::exit(1);
                    }
                }
            }
            TemporalAction::List => {
                let store = load_or_create(&store_path);
                let scheduled = store.list_scheduled();
                println!("{}", serde_json::to_string_pretty(&scheduled).unwrap());
            }
            TemporalAction::Cancel { temporal_id } => {
                let mut store = load_or_create(&store_path);
                match store.cancel_scheduled(temporal_id) {
                    Ok(()) => {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "status": "cancelled",
                                "temporal_id": temporal_id,
                            }))
                            .unwrap()
                        );
                        if let Err(e) = store.save(&store_path) {
                            eprintln!("Warning: failed to save store: {e}");
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {e}");
                        std::process::exit(1);
                    }
                }
            }
            TemporalAction::Deliver => {
                let mut store = load_or_create(&store_path);
                let delivered = store.deliver_pending_temporal();
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "status": "delivered",
                        "count": delivered,
                    }))
                    .unwrap()
                );
                if let Err(e) = store.save(&store_path) {
                    eprintln!("Warning: failed to save store: {e}");
                }
            }
        },

        // -----------------------------------------------------------------
        // Status — comprehensive stats
        // -----------------------------------------------------------------
        Commands::Status => {
            let store = load_or_create(&store_path);
            let stats = store.stats();
            println!("{}", serde_json::to_string_pretty(&stats).unwrap());
        }

        // -----------------------------------------------------------------
        // Ground — claim verification
        // -----------------------------------------------------------------
        Commands::Ground { claim } => {
            let store = load_or_create(&store_path);
            let result = store.ground_claim(&claim);
            println!("{}", serde_json::to_string_pretty(&result).unwrap());
        }
    }
}
