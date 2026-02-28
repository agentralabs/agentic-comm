//! Binary .acomm file format with integrity checking.
//!
//! **Version 3 (current)**: Uses Blake3 for integrity checking.
//!
//! Format v3:
//! - Magic: "ACOM" (4 bytes)
//! - Version: u16 (2 bytes, little-endian, value = 3)
//! - Flags: u16 (2 bytes, reserved)
//! - Data length: u64 (8 bytes, little-endian)
//! - Blake3 hash: [u8; 32] (32 bytes, of the data section)
//! - Data: [u8; data_length] (bincode + gzip compressed)
//!
//! Format v2 (legacy):
//! - Magic: "ACOM" (4 bytes)
//! - Version: u16 (2 bytes, little-endian, value = 2)
//! - Flags: u16 (2 bytes, reserved)
//! - Data length: u64 (8 bytes, little-endian)
//! - CRC32: u32 (4 bytes, little-endian, of the data section)
//! - Data: [u8; data_length]

use crc32fast::Hasher as Crc32Hasher;

pub const MAGIC: &[u8; 4] = b"ACOM";
/// Current format version (Blake3 integrity).
pub const FORMAT_VERSION: u16 = 3;
/// Legacy format version (CRC32 integrity).
pub const LEGACY_FORMAT_VERSION: u16 = 2;

/// Flag bit indicating Zstd compression (bit 0 of `flags`).
/// When clear (0), the payload is gzip-compressed (legacy default).
/// When set (1), the payload is Zstd-compressed.
pub const FLAG_ZSTD: u16 = 1 << 0;

/// File header for .acomm format.
///
/// Supports both v2 (CRC32, 20-byte header) and v3 (Blake3, 48-byte header).
#[derive(Debug, Clone)]
pub struct FileHeader {
    pub magic: [u8; 4],
    pub version: u16,
    pub flags: u16,
    pub data_length: u64,
    /// CRC32 checksum (used in v2 only).
    pub crc32: u32,
    /// Blake3 hash (used in v3+).
    pub blake3_hash: [u8; 32],
}

impl FileHeader {
    /// Header size for v3 (Blake3): magic(4) + version(2) + flags(2) + data_length(8) + blake3(32) = 48.
    pub const SIZE_V3: usize = 4 + 2 + 2 + 8 + 32; // 48 bytes
    /// Header size for v2 (CRC32): magic(4) + version(2) + flags(2) + data_length(8) + crc32(4) = 20.
    pub const SIZE_V2: usize = 4 + 2 + 2 + 8 + 4; // 20 bytes
    /// Default header size (current version = v3).
    pub const SIZE: usize = Self::SIZE_V3;

    /// Create a new v3 header (Blake3) for the given data.
    pub fn new(data: &[u8]) -> Self {
        Self::new_with_flags(data, 0)
    }

    /// Create a new v3 header (Blake3) with explicit flags.
    pub fn new_with_flags(data: &[u8], flags: u16) -> Self {
        let hash = blake3::hash(data);
        Self {
            magic: *MAGIC,
            version: FORMAT_VERSION,
            flags,
            data_length: data.len() as u64,
            crc32: 0,
            blake3_hash: *hash.as_bytes(),
        }
    }

    /// Create a legacy v2 header (CRC32) for the given data.
    pub fn new_legacy(data: &[u8]) -> Self {
        let mut crc = Crc32Hasher::new();
        crc.update(data);
        Self {
            magic: *MAGIC,
            version: LEGACY_FORMAT_VERSION,
            flags: 0,
            data_length: data.len() as u64,
            crc32: crc.finalize(),
            blake3_hash: [0u8; 32],
        }
    }

    /// Serialize header to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        if self.version >= 3 {
            let mut buf = Vec::with_capacity(Self::SIZE_V3);
            buf.extend_from_slice(&self.magic);
            buf.extend_from_slice(&self.version.to_le_bytes());
            buf.extend_from_slice(&self.flags.to_le_bytes());
            buf.extend_from_slice(&self.data_length.to_le_bytes());
            buf.extend_from_slice(&self.blake3_hash);
            buf
        } else {
            let mut buf = Vec::with_capacity(Self::SIZE_V2);
            buf.extend_from_slice(&self.magic);
            buf.extend_from_slice(&self.version.to_le_bytes());
            buf.extend_from_slice(&self.flags.to_le_bytes());
            buf.extend_from_slice(&self.data_length.to_le_bytes());
            buf.extend_from_slice(&self.crc32.to_le_bytes());
            buf
        }
    }

    /// Parse header from bytes, auto-detecting version for correct layout.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        // We need at least the common prefix (magic + version + flags + data_length = 16 bytes)
        // to determine the version, then read the rest.
        if bytes.len() < 16 {
            return Err(format!("Header too short: {} bytes", bytes.len()));
        }

        let magic: [u8; 4] = bytes[0..4].try_into().unwrap();
        if &magic != MAGIC {
            return Err(format!("Invalid magic: {:?}", magic));
        }

        let version = u16::from_le_bytes(bytes[4..6].try_into().unwrap());
        let flags = u16::from_le_bytes(bytes[6..8].try_into().unwrap());
        let data_length = u64::from_le_bytes(bytes[8..16].try_into().unwrap());

        if version >= 3 {
            // V3: Blake3 hash follows (32 bytes)
            if bytes.len() < Self::SIZE_V3 {
                return Err(format!(
                    "V3 header too short: {} bytes (need {})",
                    bytes.len(),
                    Self::SIZE_V3
                ));
            }
            let mut blake3_hash = [0u8; 32];
            blake3_hash.copy_from_slice(&bytes[16..48]);
            Ok(Self {
                magic,
                version,
                flags,
                data_length,
                crc32: 0,
                blake3_hash,
            })
        } else {
            // V2 (legacy): CRC32 follows (4 bytes)
            if bytes.len() < Self::SIZE_V2 {
                return Err(format!(
                    "V2 header too short: {} bytes (need {})",
                    bytes.len(),
                    Self::SIZE_V2
                ));
            }
            let crc32 = u32::from_le_bytes(bytes[16..20].try_into().unwrap());
            Ok(Self {
                magic,
                version,
                flags,
                data_length,
                crc32,
                blake3_hash: [0u8; 32],
            })
        }
    }

    /// Return the header size for this version.
    pub fn header_size(&self) -> usize {
        if self.version >= 3 {
            Self::SIZE_V3
        } else {
            Self::SIZE_V2
        }
    }

    /// Verify integrity of data against the stored hash/checksum.
    pub fn verify(&self, data: &[u8]) -> bool {
        if self.version >= 3 {
            self.verify_blake3(data)
        } else {
            self.verify_crc(data)
        }
    }

    /// Verify CRC32 of data matches header (v2 legacy).
    pub fn verify_crc(&self, data: &[u8]) -> bool {
        let mut crc = Crc32Hasher::new();
        crc.update(data);
        crc.finalize() == self.crc32
    }

    /// Verify Blake3 hash of data matches header (v3+).
    pub fn verify_blake3(&self, data: &[u8]) -> bool {
        let hash = blake3::hash(data);
        hash.as_bytes() == &self.blake3_hash
    }

    /// Returns `true` if the payload is Zstd-compressed (FLAG_ZSTD set).
    pub fn is_zstd(&self) -> bool {
        self.flags & FLAG_ZSTD != 0
    }
}

/// Write data with the binary format header (current version = v3 / Blake3).
pub fn write_with_header(data: &[u8]) -> Vec<u8> {
    write_with_header_flags(data, 0)
}

/// Write data with the binary format header and explicit flags.
pub fn write_with_header_flags(data: &[u8], flags: u16) -> Vec<u8> {
    let header = FileHeader::new_with_flags(data, flags);
    let mut output = header.to_bytes();
    output.extend_from_slice(data);
    output
}

/// Write data with a legacy v2 header (CRC32). Useful for backward-compat tests.
pub fn write_with_header_legacy(data: &[u8]) -> Vec<u8> {
    let header = FileHeader::new_legacy(data);
    let mut output = header.to_bytes();
    output.extend_from_slice(data);
    output
}

/// Read and verify data from binary format.
///
/// Auto-detects v2 (CRC32) and v3 (Blake3) headers.
pub fn read_with_header(input: &[u8]) -> Result<Vec<u8>, String> {
    let (_header, data) = read_with_header_and_meta(input)?;
    Ok(data)
}

/// Read and verify data from binary format, also returning the parsed header.
///
/// The caller can inspect `header.is_zstd()` to determine which decompressor
/// to use on the returned data.
pub fn read_with_header_and_meta(input: &[u8]) -> Result<(FileHeader, Vec<u8>), String> {
    let header = FileHeader::from_bytes(input)?;
    let hdr_size = header.header_size();
    let data_start = hdr_size;
    let data_end = data_start + header.data_length as usize;

    if input.len() < data_end {
        return Err(format!(
            "File truncated: expected {} bytes, got {}",
            data_end,
            input.len()
        ));
    }

    let data = &input[data_start..data_end];

    if !header.verify(data) {
        if header.version >= 3 {
            return Err("Blake3 hash mismatch — file may be corrupted".to_string());
        } else {
            return Err("CRC32 checksum mismatch — file may be corrupted".to_string());
        }
    }

    Ok((header, data.to_vec()))
}

/// Check if data starts with the ACOM magic bytes (new format).
pub fn is_new_format(data: &[u8]) -> bool {
    data.len() >= 4 && &data[0..4] == MAGIC
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- V3 (Blake3) tests ----

    #[test]
    fn header_roundtrip_v3() {
        let data = b"hello, agentic-comm!";
        let header = FileHeader::new(data);
        let bytes = header.to_bytes();
        assert_eq!(bytes.len(), FileHeader::SIZE_V3);
        let parsed = FileHeader::from_bytes(&bytes).unwrap();

        assert_eq!(parsed.magic, *MAGIC);
        assert_eq!(parsed.version, FORMAT_VERSION);
        assert_eq!(parsed.flags, 0);
        assert_eq!(parsed.data_length, data.len() as u64);
        assert_eq!(parsed.blake3_hash, header.blake3_hash);
    }

    #[test]
    fn blake3_verification() {
        let data = b"integrity matters";
        let header = FileHeader::new(data);
        assert!(header.verify_blake3(data));
        assert!(header.verify(data));
    }

    #[test]
    fn blake3_corrupted_data_detection() {
        let data = b"original payload";
        let wrapped = write_with_header(data);

        // Corrupt one byte in the data section
        let mut corrupted = wrapped.clone();
        let last = corrupted.len() - 1;
        corrupted[last] ^= 0xFF;

        let result = read_with_header(&corrupted);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Blake3 hash mismatch"));
    }

    #[test]
    fn write_read_roundtrip_v3() {
        let data = b"roundtrip test data with some content";
        let wrapped = write_with_header(data);
        let recovered = read_with_header(&wrapped).unwrap();
        assert_eq!(recovered, data);
    }

    #[test]
    fn empty_data_v3() {
        let data: &[u8] = b"";
        let wrapped = write_with_header(data);
        let recovered = read_with_header(&wrapped).unwrap();
        assert!(recovered.is_empty());
    }

    #[test]
    fn truncated_file_detection_v3() {
        let data = b"some payload";
        let wrapped = write_with_header(data);

        // Truncate the file partway through the data section
        let truncated = &wrapped[..FileHeader::SIZE_V3 + 3];
        let result = read_with_header(truncated);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("truncated"));
    }

    #[test]
    fn large_data_blake3() {
        let data: Vec<u8> = (0..100_000).map(|i| (i % 256) as u8).collect();
        let wrapped = write_with_header(&data);
        let recovered = read_with_header(&wrapped).unwrap();
        assert_eq!(recovered, data);
    }

    // ---- V2 (CRC32 legacy) tests ----

    #[test]
    fn header_roundtrip_v2_legacy() {
        let data = b"legacy format payload";
        let header = FileHeader::new_legacy(data);
        let bytes = header.to_bytes();
        assert_eq!(bytes.len(), FileHeader::SIZE_V2);
        let parsed = FileHeader::from_bytes(&bytes).unwrap();

        assert_eq!(parsed.magic, *MAGIC);
        assert_eq!(parsed.version, LEGACY_FORMAT_VERSION);
        assert_eq!(parsed.data_length, data.len() as u64);
        assert_eq!(parsed.crc32, header.crc32);
    }

    #[test]
    fn crc_verification_legacy() {
        let data = b"integrity matters (legacy)";
        let header = FileHeader::new_legacy(data);
        assert!(header.verify_crc(data));
        assert!(header.verify(data));
    }

    #[test]
    fn write_read_roundtrip_legacy() {
        let data = b"legacy roundtrip test";
        let wrapped = write_with_header_legacy(data);
        let recovered = read_with_header(&wrapped).unwrap();
        assert_eq!(recovered, data);
    }

    #[test]
    fn corrupted_data_detection_legacy() {
        let data = b"legacy payload";
        let wrapped = write_with_header_legacy(data);

        let mut corrupted = wrapped.clone();
        let last = corrupted.len() - 1;
        corrupted[last] ^= 0xFF;

        let result = read_with_header(&corrupted);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("CRC32 checksum mismatch"));
    }

    // ---- Cross-version and detection tests ----

    #[test]
    fn legacy_format_detection() {
        // Legacy data does not start with ACOM magic
        let legacy = b"\x01\x00\x00\x00some old data";
        assert!(!is_new_format(legacy));

        // V3 format starts with ACOM
        let new_data = b"original";
        let wrapped = write_with_header(new_data);
        assert!(is_new_format(&wrapped));

        // V2 format also starts with ACOM
        let legacy_wrapped = write_with_header_legacy(new_data);
        assert!(is_new_format(&legacy_wrapped));
    }

    #[test]
    fn invalid_magic_detection() {
        let mut bad = vec![0u8; 48];
        bad[0..4].copy_from_slice(b"NOPE");
        let result = FileHeader::from_bytes(&bad);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid magic"));
    }

    #[test]
    fn header_too_short() {
        let short = vec![0u8; 10];
        let result = FileHeader::from_bytes(&short);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("too short"));
    }

    #[test]
    fn version_field_is_3() {
        let data = b"version check";
        let wrapped = write_with_header(data);
        let header = FileHeader::from_bytes(&wrapped).unwrap();
        assert_eq!(header.version, 3);
    }

    #[test]
    fn v3_header_is_48_bytes() {
        assert_eq!(FileHeader::SIZE_V3, 48);
        assert_eq!(FileHeader::SIZE_V3 - FileHeader::SIZE_V2, 28);
    }
}
