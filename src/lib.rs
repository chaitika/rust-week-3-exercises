use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::Deref;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct CompactSize {
    pub value: u64,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BitcoinError {
    InsufficientBytes,
    InvalidFormat,
}

impl CompactSize {
    pub fn new(value: u64) -> Self {
        // TODO: Construct a CompactSize from a u64 value
        CompactSize { value } // same as Self { value }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: Encode according to Bitcoin's CompactSize format:         // 'x' == 4 bits
        // [0x00–0xFC] => 1 byte                                           // 0 -- 252
        // [0xFDxxxx] => 0xFD + u16 (2 bytes)                              // 253 -- 65535
        // [0xFExxxxxxxx] => 0xFE + u32 (4 bytes)                          // 65536 -- 4294967295
        // [0xFFxxxxxxxxxxxxxxxx] => 0xFF + u64 (8 bytes)

        match self.value {
            0..=252 => vec![self.value as u8],
            253..=65535 => {
                let mut v = vec![0xFD];
                v.extend_from_slice(&(self.value as u16).to_le_bytes());
                v
            }
            65536..=4294967295 => {
                let mut v = vec![0xFE];
                v.extend_from_slice(&(self.value as u32).to_le_bytes());
                v
            }
            _ => {
                let mut v = vec![0xFF];
                v.extend_from_slice(&self.value.to_le_bytes());
                v
            }
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
        // TODO: Decode CompactSize, returning value and number of bytes consumed.
        // First check if bytes is empty.
        // Check that enough bytes are available based on prefix.
        if bytes.is_empty() {
            return Err(BitcoinError::InsufficientBytes);
        }

        match bytes[0] {
            n @ 0x00..=0xFC => Ok((Self::new(n as u64), 1)),
            0xFD => {
                let val = u16::from_le_bytes([bytes[1], bytes[2]]) as u64;
                Ok((Self::new(val), 3))
            }
            0xFE => {
                let val = u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]) as u64;
                Ok((Self::new(val), 5))
            }
            0xFF => {
                let val = u64::from_le_bytes([
                    bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8],
                ]);
                Ok((Self::new(val), 9))
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Txid(pub [u8; 32]);

impl Serialize for Txid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // TODO: Serialize as a hex-encoded string (32 bytes => 64 hex characters)
        let hex = hex::encode(self.0);
        serializer.serialize_str(&hex)
    }
}

impl<'de> Deserialize<'de> for Txid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // TODO: Parse hex string into 32-byte array
        // Use `hex::decode`, validate length = 32
        let hex_str = String::deserialize(deserializer)?;
        let bytes = hex::decode(&hex_str).map_err(serde::de::Error::custom)?;
        if bytes.len() != 32 {
            return Err(serde::de::Error::custom("txid must be 32 bytes"));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Txid(arr))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct OutPoint {
    pub txid: Txid,
    pub vout: u32,
}

impl OutPoint {
    pub fn new(txid: [u8; 32], vout: u32) -> Self {
        // TODO: Create an OutPoint from raw txid bytes and output index
        OutPoint {
            txid: Txid(txid),
            vout,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: Serialize as: txid (32 bytes) + vout (4 bytes, little-endian)
        let mut bytes = self.txid.0.to_vec(); // 32 bytes
        bytes.extend(&self.vout.to_le_bytes()); // 4 bytes, little-endian
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
        // TODO: Deserialize 36 bytes: txid[0..32], vout[32..36]
        // Return error if insufficient bytes
        if bytes.len() < 36 {
            return Err(BitcoinError::InsufficientBytes);
        }
        let mut txid = [0u8; 32];
        txid.copy_from_slice(&bytes[..32]);
        let vout = u32::from_le_bytes(bytes[32..36].try_into().unwrap());
        Ok((OutPoint::new(txid, vout), 36))
    }
}

fn decode_compact_size(bytes: &[u8]) -> Result<(u64, usize), BitcoinError> {
    if bytes.is_empty() {
        return Err(BitcoinError::InsufficientBytes);
    }
    match bytes[0] {
        n @ 0x00..=0xFC => Ok((n as u64, 1)),
        0xFD => {
            if bytes.len() < 3 {
                Err(BitcoinError::InsufficientBytes)
            } else {
                let val = u16::from_le_bytes(bytes[1..3].try_into().unwrap()) as u64;
                Ok((val, 3))
            }
        }
        0xFE => {
            if bytes.len() < 5 {
                Err(BitcoinError::InsufficientBytes)
            } else {
                let val = u32::from_le_bytes(bytes[1..5].try_into().unwrap()) as u64;
                Ok((val, 5))
            }
        }
        0xFF => {
            if bytes.len() < 9 {
                Err(BitcoinError::InsufficientBytes)
            } else {
                let val = u64::from_le_bytes(bytes[1..9].try_into().unwrap());
                Ok((val, 9))
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Script {
    pub bytes: Vec<u8>,
}

fn encode_compact_size(n: u64) -> Vec<u8> {
    match n {
        0..=0xFC => vec![n as u8],
        0xFD..=0xFFFF => {
            let mut v = vec![0xFD];
            v.extend(&(n as u16).to_le_bytes());
            v
        }
        0x10000..=0xFFFF_FFFF => {
            let mut v = vec![0xFE];
            v.extend(&(n as u32).to_le_bytes());
            v
        }
        _ => {
            let mut v = vec![0xFF];
            v.extend(&n.to_le_bytes());
            v
        }
    }
}

impl Script {
    pub fn new(bytes: Vec<u8>) -> Self {
        // TODO: Simple constructor
        Script { bytes }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: Prefix with CompactSize (length), then raw bytes
        let mut result = Vec::new();
        result.extend(encode_compact_size(self.bytes.len() as u64));
        result.extend(&self.bytes);
        result
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
        // TODO: Parse CompactSize prefix, then read that many bytes
        // Return error if not enough bytes
        let (len, prefix_len) = decode_compact_size(bytes)?;
        let total_len = prefix_len + len as usize;
        if bytes.len() < total_len {
            return Err(BitcoinError::InsufficientBytes);
        }
        let script_bytes = bytes[prefix_len..total_len].to_vec();
        Ok((Script::new(script_bytes), total_len))
    }
}

impl Deref for Script {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        // TODO: Allow &Script to be used as &[u8]
        &self.bytes
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct TransactionInput {
    pub previous_output: OutPoint,
    pub script_sig: Script,
    pub sequence: u32,
}

impl TransactionInput {
    pub fn new(previous_output: OutPoint, script_sig: Script, sequence: u32) -> Self {
        TransactionInput {
            previous_output,
            script_sig,
            sequence,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: Serialize: OutPoint + Script (with CompactSize) + sequence (4 bytes LE)
        let mut result = Vec::new();
        result.extend(self.previous_output.to_bytes()); // 36 bytes
        result.extend(self.script_sig.to_bytes()); // CompactSize + script bytes
        result.extend(&self.sequence.to_le_bytes()); // 4 bytes, little-endian
        result
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
        // TODO: Deserialize in order:
        // - OutPoint (36 bytes)
        // - Script (with CompactSize)
        // - Sequence (4 bytes)
        let (outpoint, outpoint_len) = OutPoint::from_bytes(bytes)?;
        let (script_sig, script_len) = Script::from_bytes(&bytes[outpoint_len..])?;

        let seq_start = outpoint_len + script_len;
        if bytes.len() < seq_start + 4 {
            return Err(BitcoinError::InsufficientBytes);
        }
        let sequence = u32::from_le_bytes(bytes[seq_start..seq_start + 4].try_into().unwrap());

        Ok((
            TransactionInput::new(outpoint, script_sig, sequence),
            seq_start + 4,
        ))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct BitcoinTransaction {
    pub version: u32,
    pub inputs: Vec<TransactionInput>,
    pub lock_time: u32,
}

impl BitcoinTransaction {
    pub fn new(version: u32, inputs: Vec<TransactionInput>, lock_time: u32) -> Self {
        BitcoinTransaction {
            version,
            inputs,
            lock_time,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: Format:
        let mut result = Vec::new();

        // Version (4 bytes LE)
        result.extend(&self.version.to_le_bytes());

        // CompactSize for number of inputs
        result.extend(encode_compact_size(self.inputs.len() as u64));

        // Inputs
        for input in &self.inputs {
            result.extend(input.to_bytes());
        }

        // Lock time (4 bytes LE)
        result.extend(&self.lock_time.to_le_bytes());

        result
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
        if bytes.len() < 4 {
            return Err(BitcoinError::InsufficientBytes);
        }

        let version = u32::from_le_bytes(bytes[0..4].try_into().unwrap());

        // Read CompactSize for input count
        let (input_count, mut offset) = decode_compact_size(&bytes[4..])?;
        offset += 4; // because CompactSize was after the 4-byte version

        let mut inputs = Vec::new();
        for _ in 0..input_count {
            let (input, consumed) = TransactionInput::from_bytes(&bytes[offset..])?;
            inputs.push(input);
            offset += consumed;
        }

        if bytes.len() < offset + 4 {
            return Err(BitcoinError::InsufficientBytes);
        }

        let lock_time = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        offset += 4;

        Ok((BitcoinTransaction::new(version, inputs, lock_time), offset))
    }
}

impl fmt::Display for BitcoinTransaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: Format a user-friendly string showing version, inputs, lock_time
        // Display scriptSig length and bytes, and previous output info
        writeln!(f, "Version: {}", self.version)?;
        for (i, input) in self.inputs.iter().enumerate() {
            writeln!(f, "Input #{}", i)?;
            writeln!(
                f,
                "  Previous Output TXID: {}",
                hex::encode(input.previous_output.txid.0)
            )?;
            writeln!(f, "  Previous Output Vout: {}", input.previous_output.vout)?;
            writeln!(
                f,
                "  ScriptSig ({} bytes): {}",
                input.script_sig.bytes.len(),
                hex::encode(&input.script_sig.bytes)
            )?;
            writeln!(f, "  Sequence: {}", input.sequence)?;
        }
        writeln!(f, "Lock Time: {}", self.lock_time)
    }
}
