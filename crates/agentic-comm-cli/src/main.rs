//! CLI for agentic-comm: agent communication, channels, pub/sub.

use std::path::PathBuf;

use agentic_comm::{ChannelType, CommStore, MessageFilter, MessageType};
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
    }
}
