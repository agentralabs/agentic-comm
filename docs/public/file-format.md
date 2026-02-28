---
status: draft
---

# .acomm File Format Specification

This document describes the binary layout of `.acomm` files -- the on-disk format for AgenticComm communication data. The format is designed for fast sequential reads, compact storage, and forward compatibility.

## Design Goals

- **Compact:** Messages are serialized with bincode and compressed with flate2 (gzip). Typical agent conversations compress at 3:1 or better.
- **Atomic writes:** The checksum footer is written last. A crash during write leaves the previous valid state intact. Readers verify the checksum and fall back to the last known-good state on mismatch.
- **Forward compatible:** Unknown sections are skipped using the section table offsets. Older readers can safely open files written by newer versions.
- **Indexed:** Channel-to-message and timestamp indexes are stored inline for fast lookup without full deserialization.
- **Bounded:** Maximum file size is 2 GB. Maximum message count is 10 million. These limits prevent unbounded growth and ensure predictable performance.

## File Layout

An `.acomm` file consists of seven contiguous sections:

```
+-------------------------+
|   Magic + Header        |  96 bytes
+-------------------------+
|   Section Table         |  variable (6 entries * 24 bytes = 144 bytes)
+-------------------------+
|   Channel Section       |  variable (bincode serialized)
+-------------------------+
|   Message Section       |  variable (bincode + flate2 compressed)
+-------------------------+
|   Subscription Section  |  variable (bincode serialized)
+-------------------------+
|   Index Section         |  variable (bincode serialized)
+-------------------------+
|   Footer                |  40 bytes (checksum + metadata)
+-------------------------+
```

## Section 1: Header (96 bytes)

The header occupies the first 96 bytes of the file.

| Offset | Size | Type | Field | Description |
|--------|------|------|-------|-------------|
| 0 | 8 | `[u8; 8]` | `magic` | Magic bytes: `0x41 0x43 0x4F 0x4D 0x4D 0x30 0x30 0x31` (ASCII "ACOMM001"). |
| 8 | 2 | `u16` | `version` | Format version. Current: `1`. |
| 10 | 4 | `u32` | `flags` | Bitfield. See flags table below. |
| 14 | 2 | `u16` | `section_count` | Number of sections in the section table. Current: `6`. |
| 16 | 8 | `u64` | `channel_count` | Total number of channels. |
| 24 | 8 | `u64` | `message_count` | Total number of messages (active + archived). |
| 32 | 8 | `u64` | `subscription_count` | Total number of subscriptions. |
| 40 | 8 | `u64` | `dead_letter_count` | Number of messages in the dead letter queue. |
| 48 | 8 | `u64` | `created_at` | Store creation timestamp (Unix seconds, UTC). |
| 56 | 8 | `u64` | `modified_at` | Last modification timestamp (Unix seconds, UTC). |
| 64 | 8 | `u64` | `total_size` | Total file size in bytes. Used for quick validation. |
| 72 | 24 | `[u8; 24]` | `reserved` | Reserved for future use. Must be zero. |

### Flags Bitfield

| Bit | Name | Description |
|-----|------|-------------|
| 0 | `COMPRESSED` | Message section is flate2 compressed. |
| 1 | `INDEXED` | Index section is present. |
| 2 | `HAS_DEAD_LETTERS` | Dead letter messages are included in the message section. |
| 3 | `HAS_SIGNATURES` | At least one message carries a cryptographic signature. |
| 4 | `HAS_METADATA` | At least one message carries metadata. |
| 5 | `ENCRYPTED` | Message content is encrypted at rest. |
| 6-31 | -- | Reserved. Must be zero. |

### Validation Rules

- `magic` must be exactly `ACOMM001` (8 bytes).
- `version` must be `<= 1` for this specification. Readers encountering a higher version should either upgrade or refuse to open the file with a clear error message.
- `section_count` must match the number of entries in the section table.
- `channel_count` and `message_count` must be consistent with their respective sections.
- `total_size` must match the actual file size.
- Reserved bytes must be zero. Readers must ignore reserved bytes (do not reject files with non-zero reserved bytes, for forward compatibility).

## Section 2: Section Table

Immediately following the header. Each entry is 24 bytes. The table has `section_count` entries (currently 6).

| Offset | Size | Type | Field | Description |
|--------|------|------|-------|-------------|
| 0 | 4 | `u32` | `section_type` | Section type identifier. See table below. |
| 4 | 4 | `u32` | `flags` | Section-specific flags. Currently reserved (must be zero). |
| 8 | 8 | `u64` | `offset` | Byte offset from file start to the beginning of this section. |
| 16 | 8 | `u64` | `length` | Length of this section in bytes. |

### Section Types

| Value | Name | Description |
|-------|------|-------------|
| 1 | `CHANNELS` | Channel definitions and configurations. |
| 2 | `MESSAGES` | Message records (compressed). |
| 3 | `SUBSCRIPTIONS` | Pub/sub subscription records. |
| 4 | `INDEXES` | Lookup indexes (channel-message, timestamp, topic, sender). |
| 5 | `DEAD_LETTERS` | Failed messages that exhausted retries. |
| 6 | `ARCHIVE` | Archived messages (retained but past retention policy). |
| 7-255 | -- | Reserved for future sections. Unknown section types must be skipped. |

### Forward Compatibility

When a reader encounters an unknown section type:
1. Log a warning (not an error).
2. Skip the section using the `offset` and `length` fields.
3. Continue processing remaining sections.

This allows older readers to open files written by newer versions that introduce additional sections.

## Section 3: Channel Section

Contains all channel records, serialized sequentially with bincode.

### Channel Record Layout

Each channel is serialized as a bincode struct. The section is a length-prefixed sequence:

```
[channel_count: u64]
[channel_0: bincode bytes]
[channel_1: bincode bytes]
...
[channel_N: bincode bytes]
```

Each channel record includes:

| Field | Type | Description |
|-------|------|-------------|
| `id` | `u64` | Channel ID. |
| `name_len` | `u32` | Length of channel name. |
| `name` | `[u8; name_len]` | Channel name (UTF-8). |
| `channel_type` | `u8` | Channel type enum value. |
| `owner_len` | `u32` | Length of owner participant ID. |
| `owner` | `[u8; owner_len]` | Owner participant ID (UTF-8). |
| `participant_count` | `u32` | Number of participants. |
| `participants` | `[Participant; count]` | Participant records (see below). |
| `config` | `ChannelConfig` | Serialized channel configuration. |
| `state` | `u8` | Channel state enum value. |
| `created_at` | `u64` | Creation timestamp. |
| `modified_at` | `u64` | Last modification timestamp. |
| `message_count` | `u64` | Number of messages in this channel. |
| `description_len` | `u32` | Length of description (0 if none). |
| `description` | `[u8; desc_len]` | Description (UTF-8), omitted if length is 0. |
| `tag_count` | `u32` | Number of tags. |
| `tags` | `[Tag; count]` | Tags (length-prefixed strings). |

### Participant Sub-record

| Field | Type | Description |
|-------|------|-------------|
| `id_len` | `u32` | Length of participant ID. |
| `id` | `[u8; id_len]` | Participant ID (UTF-8). |
| `role` | `u8` | Role enum value (0=Owner, 1=Member, 2=Observer). |
| `joined_at` | `u64` | Join timestamp. |
| `has_identity` | `u8` | 1 if identity_id is present, 0 otherwise. |
| `identity_id_len` | `u32` | Length of identity ID (only if has_identity=1). |
| `identity_id` | `[u8; len]` | Identity ID string (only if has_identity=1). |

## Section 4: Message Section

Contains all active messages, serialized with bincode and optionally compressed with flate2.

### Compression Envelope

When the `COMPRESSED` flag is set in the header:

```
[uncompressed_size: u64]  -- 8 bytes, original size before compression
[compressed_data: bytes]  -- flate2 gzip compressed bincode data
```

When the `COMPRESSED` flag is not set:

```
[raw_data: bytes]  -- uncompressed bincode data
```

### Message Record Sequence

After decompression (or directly if uncompressed), the message section contains:

```
[message_count: u64]
[message_0: bincode bytes]
[message_1: bincode bytes]
...
[message_N: bincode bytes]
```

Messages are stored in creation order (ascending by `created_at`). This enables efficient temporal range queries on the raw data.

### Message Record Layout

Each message is a bincode-serialized struct with the fields defined in [Data Structures](data-structures.md). Key considerations:

- `content` is stored as a length-prefixed byte array (UTF-8). The content is part of the compressed block, so individual message content is not independently accessible without decompressing the entire section.
- `signature`, if present, is stored as a length-prefixed byte array.
- `metadata`, if present, is serialized as a bincode map.
- Optional fields use bincode's `Option` encoding (1 byte tag: 0=None, 1=Some followed by value).

## Section 5: Subscription Section

Contains all pub/sub subscriptions.

```
[subscription_count: u64]
[subscription_0: bincode bytes]
[subscription_1: bincode bytes]
...
[subscription_N: bincode bytes]
```

Each subscription record follows the Subscription struct layout from [Data Structures](data-structures.md).

## Section 6: Index Section

Contains lookup indexes for fast queries without full message scanning.

### Index Types

The index section contains multiple sub-indexes, each prefixed with a type tag and length:

```
[index_count: u32]
[index_type_0: u32] [index_length_0: u64] [index_data_0: bytes]
[index_type_1: u32] [index_length_1: u64] [index_data_1: bytes]
...
```

### Channel-Message Index (type = 1)

Maps channel IDs to the message IDs they contain.

```
[entry_count: u64]
For each entry:
  [channel_id: u64]
  [message_id_count: u64]
  [message_ids: [u64; count]]   -- sorted ascending
```

### Timestamp Index (type = 2)

Sorted array of (timestamp, message_id) pairs for binary search.

```
[entry_count: u64]
For each entry:
  [timestamp: u64]
  [message_id: u64]
```

Entries are sorted by timestamp ascending. Ties are broken by message_id ascending.

### Topic Index (type = 3)

Maps exact topic strings to message IDs.

```
[entry_count: u64]
For each entry:
  [topic_len: u32]
  [topic: [u8; topic_len]]
  [message_id_count: u64]
  [message_ids: [u64; count]]   -- sorted ascending
```

### Sender Index (type = 4)

Maps sender participant IDs to their message IDs.

```
[entry_count: u64]
For each entry:
  [sender_len: u32]
  [sender: [u8; sender_len]]
  [message_id_count: u64]
  [message_ids: [u64; count]]   -- sorted ascending
```

### Correlation Index (type = 5)

Maps correlation IDs to all messages in the thread.

```
[entry_count: u64]
For each entry:
  [correlation_id_len: u32]
  [correlation_id: [u8; len]]
  [message_id_count: u64]
  [message_ids: [u64; count]]   -- sorted ascending
```

## Section 7: Footer (40 bytes)

The footer is the last 40 bytes of the file. It is written last to ensure atomic updates.

| Offset | Size | Type | Field | Description |
|--------|------|------|-------|-------------|
| 0 | 32 | `[u8; 32]` | `checksum` | SHA-256 hash of all preceding bytes (header through index section). |
| 32 | 8 | `[u8; 8]` | `footer_magic` | Footer magic bytes: `0x41 0x43 0x45 0x4E 0x44 0x30 0x30 0x31` (ASCII "ACEND001"). |

### Checksum Verification

On file load:
1. Read the last 40 bytes to extract checksum and footer magic.
2. Verify footer magic is `ACEND001`.
3. Compute SHA-256 of bytes 0 through (file_size - 40).
4. Compare computed hash with stored checksum.
5. If mismatch: reject the file with an integrity error. Do not silently load corrupt data.

### Atomic Write Protocol

When writing a `.acomm` file:
1. Write to a temporary file (`<path>.acomm.tmp`).
2. Write header, section table, all sections.
3. Compute SHA-256 of everything written so far.
4. Write footer (checksum + magic).
5. Flush and sync the temporary file.
6. Atomically rename the temporary file to the target path.

This ensures that readers never see a partially-written file. If the process crashes during write, the temporary file is left behind and can be cleaned up on the next successful write.

## Size Limits

| Limit | Value | Rationale |
|-------|-------|-----------|
| Maximum file size | 2 GB (2,147,483,648 bytes) | Keeps memory-mapped operations predictable. |
| Maximum message count | 10,000,000 | Prevents excessive decompression time. |
| Maximum channel count | 100,000 | Keeps channel index manageable. |
| Maximum subscription count | 1,000,000 | Prevents subscription matching overhead. |
| Maximum message content | 1 MB (1,048,576 bytes) | Prevents single messages from dominating the file. |
| Maximum channel name | 128 bytes | Keeps indexes compact. |
| Maximum topic length | 256 bytes | Keeps topic matching fast. |

When a limit is reached, the store returns an error with a descriptive message. The store does not silently truncate or drop data.

## Versioning

The `version` field in the header enables format evolution:

| Version | Status | Changes |
|---------|--------|---------|
| 1 | Current | Initial format. All sections as described above. |

### Version Upgrade Protocol

When a reader encounters `version > 1`:
1. If the reader supports the version, proceed normally.
2. If the reader does not support the version, check if the section table contains only known section types.
3. If all section types are known, attempt to read the file (the new version may only add new sections that can be skipped).
4. If unknown section types are present, skip them and read known sections.
5. If the header structure itself has changed (different header size), reject with a version error.

### Version Downgrade

Files written by version N can be read by readers supporting version N or higher. Downgrade (opening a version-2 file with a version-1 reader) is supported only if the version-2 file uses no features beyond version-1 and all unknown sections can be safely skipped.

## File Locking

When multiple processes access the same `.acomm` file:

1. Writers acquire an exclusive lock on `<path>.acomm.lock` (advisory file lock).
2. Readers do not require a lock (they read the last atomically-written version).
3. The lock file contains the PID of the holding process and a timestamp.
4. Stale locks (process PID no longer running) are automatically recovered after 60 seconds.

### Lock File Format

```
PID: <process_id>
STARTED: <unix_timestamp>
HOSTNAME: <hostname>
```

Plain text, human-readable. This allows manual inspection and recovery.

## Example File Sizes

| Scenario | Channels | Messages | Uncompressed | Compressed | Ratio |
|----------|----------|----------|--------------|------------|-------|
| Small session | 3 | 100 | 48 KB | 18 KB | 2.7:1 |
| Day of work | 10 | 5,000 | 2.1 MB | 680 KB | 3.1:1 |
| Week of work | 25 | 50,000 | 19 MB | 5.8 MB | 3.3:1 |
| Large project | 100 | 500,000 | 180 MB | 52 MB | 3.5:1 |
| Maximum capacity | 100,000 | 10,000,000 | ~1.8 GB | ~500 MB | 3.6:1 |
