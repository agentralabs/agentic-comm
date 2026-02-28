//! Binary .acomm file format with integrity checking.
//!
//! Format:
//! - Magic: "ACOM" (4 bytes)
//! - Version: u16 (2 bytes, little-endian, currently 2)
//! - Flags: u16 (2 bytes, reserved)
//! - Data length: u64 (8 bytes, little-endian)
//! - CRC32: u32 (4 bytes, little-endian, of the data section)
//! - Data: [u8; data_length] (bincode + gzip compressed)

use crc32fast::Hasher as Crc32Hasher;

pub const MAGIC: &[u8; 4] = b"ACOM";
pub const FORMAT_VERSION: u16 = 2;

/// File header for .acomm format.
#[derive(Debug, Clone)]
pub struct FileHeader {
    pub magic: [u8; 4],
    pub version: u16,
    pub flags: u16,
    pub data_length: u64,
    pub crc32: u32,
}

impl FileHeader {
    pub const SIZE: usize = 4 + 2 + 2 + 8 + 4; // 20 bytes

    /// Create a new header for the given data.
    pub fn new(data: &[u8]) -> Self {
        let mut crc = Crc32Hasher::new();
        crc.update(data);
        Self {
            magic: *MAGIC,
            version: FORMAT_VERSION,
            flags: 0,
            data_length: data.len() as u64,
            crc32: crc.finalize(),
        }
    }

    /// Serialize header to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(Self::SIZE);
        buf.extend_from_slice(&self.magic);
        buf.extend_from_slice(&self.version.to_le_bytes());
        buf.extend_from_slice(&self.flags.to_le_bytes());
        buf.extend_from_slice(&self.data_length.to_le_bytes());
        buf.extend_from_slice(&self.crc32.to_le_bytes());
        buf
    }

    /// Parse header from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < Self::SIZE {
            return Err(format!("Header too short: {} bytes", bytes.len()));
        }

        let magic: [u8; 4] = bytes[0..4].try_into().unwrap();
        if &magic != MAGIC {
            return Err(format!("Invalid magic: {:?}", magic));
        }

        let version = u16::from_le_bytes(bytes[4..6].try_into().unwrap());
        let flags = u16::from_le_bytes(bytes[6..8].try_into().unwrap());
        let data_length = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
        let crc32 = u32::from_le_bytes(bytes[16..20].try_into().unwrap());

        Ok(Self {
            magic,
            version,
            flags,
            data_length,
            crc32,
        })
    }

    /// Verify CRC32 of data matches header.
    pub fn verify_crc(&self, data: &[u8]) -> bool {
        let mut crc = Crc32Hasher::new();
        crc.update(data);
        crc.finalize() == self.crc32
    }
}

/// Write data with the binary format header.
pub fn write_with_header(data: &[u8]) -> Vec<u8> {
    let header = FileHeader::new(data);
    let mut output = header.to_bytes();
    output.extend_from_slice(data);
    output
}

/// Read and verify data from binary format.
pub fn read_with_header(input: &[u8]) -> Result<Vec<u8>, String> {
    let header = FileHeader::from_bytes(input)?;

    let data_start = FileHeader::SIZE;
    let data_end = data_start + header.data_length as usize;

    if input.len() < data_end {
        return Err(format!(
            "File truncated: expected {} bytes, got {}",
            data_end,
            input.len()
        ));
    }

    let data = &input[data_start..data_end];

    if !header.verify_crc(data) {
        return Err("CRC32 checksum mismatch — file may be corrupted".to_string());
    }

    Ok(data.to_vec())
}

/// Check if data starts with the ACOM magic bytes (new format).
pub fn is_new_format(data: &[u8]) -> bool {
    data.len() >= 4 && &data[0..4] == MAGIC
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_roundtrip() {
        let data = b"hello, agentic-comm!";
        let header = FileHeader::new(data);
        let bytes = header.to_bytes();
        let parsed = FileHeader::from_bytes(&bytes).unwrap();

        assert_eq!(parsed.magic, *MAGIC);
        assert_eq!(parsed.version, FORMAT_VERSION);
        assert_eq!(parsed.flags, 0);
        assert_eq!(parsed.data_length, data.len() as u64);
        assert_eq!(parsed.crc32, header.crc32);
    }

    #[test]
    fn crc_verification() {
        let data = b"integrity matters";
        let header = FileHeader::new(data);
        assert!(header.verify_crc(data));
    }

    #[test]
    fn corrupted_data_detection() {
        let data = b"original payload";
        let wrapped = write_with_header(data);

        // Corrupt one byte in the data section
        let mut corrupted = wrapped.clone();
        let last = corrupted.len() - 1;
        corrupted[last] ^= 0xFF;

        let result = read_with_header(&corrupted);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("CRC32 checksum mismatch"));
    }

    #[test]
    fn write_read_roundtrip() {
        let data = b"roundtrip test data with some content";
        let wrapped = write_with_header(data);
        let recovered = read_with_header(&wrapped).unwrap();
        assert_eq!(recovered, data);
    }

    #[test]
    fn legacy_format_detection() {
        // Legacy data does not start with ACOM magic
        let legacy = b"\x01\x00\x00\x00some old data";
        assert!(!is_new_format(legacy));

        // New format starts with ACOM
        let new_data = b"original";
        let wrapped = write_with_header(new_data);
        assert!(is_new_format(&wrapped));
    }

    #[test]
    fn empty_data() {
        let data: &[u8] = b"";
        let wrapped = write_with_header(data);
        let recovered = read_with_header(&wrapped).unwrap();
        assert!(recovered.is_empty());
    }

    #[test]
    fn truncated_file_detection() {
        let data = b"some payload";
        let wrapped = write_with_header(data);

        // Truncate the file partway through the data section
        let truncated = &wrapped[..FileHeader::SIZE + 3];
        let result = read_with_header(truncated);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("truncated"));
    }

    #[test]
    fn invalid_magic_detection() {
        let mut bad = vec![0u8; 20];
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
}
