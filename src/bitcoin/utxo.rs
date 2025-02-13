use super::address::{hash160_to_address, public_key_to_address, segwit_address};
use super::btc_leveldb::{decompress_value, varint128_decode, varint128_read};

#[derive(Debug, PartialEq)]
pub enum ScriptType {
    P2PKH,
    P2SH,
    P2PK,
    P2MS,
    P2WPKH,
    P2WSH,
    P2TR,
    NonStandard,
}

pub struct UtxoValue {
    pub height: u32,
    pub coinbase: bool,
    pub amount: i64,
    pub script_type: u64, // nsize
    pub script: Vec<u8>,
}

impl UtxoValue {
    pub fn parse(data: &[u8]) -> Result<Self, anyhow::Error> {
        let mut offset = 0;

        // First varint (height and coinbase)
        let (bytes_read, varint) = varint128_read(data, offset);
        offset += bytes_read;
        let varint_decoded = varint128_decode(&varint);

        // Extract height and coinbase from first varint
        let height = varint_decoded >> 1; // Remove last bit
        let coinbase = (varint_decoded & 1) == 1; // Last bit indicates coinbase

        // Second varint (amount)
        let (bytes_read, varint) = varint128_read(data, offset);
        offset += bytes_read;
        let compressed_amount = varint128_decode(&varint);
        let amount = decompress_value(compressed_amount);

        // Third varint (script type/size)
        let (bytes_read, varint) = varint128_read(data, offset);
        offset += bytes_read;
        let script_type = varint128_decode(&varint);

        // Adjust offset for certain script types (2-5 are P2PK variants)
        if (1 < script_type) && (script_type < 6) {
            offset -= 1;
        }

        // Get remaining bytes as script
        let script = data[offset..].to_vec();

        Ok(Self {
            height: height as u32,
            coinbase,
            amount,
            script_type: script_type as u64,
            script,
        })
    }

    pub fn get_script_type(&self) -> ScriptType {
        match self.script_type {
            0 => ScriptType::P2PKH,
            1 => ScriptType::P2SH,
            2..=5 => ScriptType::P2PK, // P2PK variants
            28 if self.script.len() >= 2 && self.script[0] == 0 && self.script[1] == 20 => {
                ScriptType::P2WPKH
            }
            40 if self.script.len() >= 2 && self.script[0] == 0 && self.script[1] == 32 => {
                ScriptType::P2WSH
            }
            40 if self.script.len() >= 2 && self.script[0] == 0x51 && self.script[1] == 32 => {
                ScriptType::P2TR
            }
            _ if !self.script.is_empty() && self.script[self.script.len() - 1] == 174 => {
                ScriptType::P2MS
            } // OP_CHECKMULTISIG = 174
            _ => ScriptType::NonStandard,
        }
    }

    pub fn get_address(&self, testnet: bool, p2pk_addresses: bool) -> Option<String> {
        match self.get_script_type() {
            ScriptType::P2PKH => {
                let prefix = if testnet { 0x6f } else { 0x00 };
                Some(hash160_to_address(&self.script, prefix))
            }
            ScriptType::P2SH => {
                let prefix = if testnet { 0xc4 } else { 0x05 };
                Some(hash160_to_address(&self.script, prefix))
            }
            ScriptType::P2PK if p2pk_addresses => {
                let prefix = if testnet { 0x6f } else { 0x00 };
                Some(public_key_to_address(&self.script, prefix))
            }
            ScriptType::P2WPKH | ScriptType::P2WSH | ScriptType::P2TR => {
                let version = if matches!(self.get_script_type(), ScriptType::P2TR) {
                    1
                } else {
                    0
                };
                segwit_address(version, &self.script[2..], testnet).ok()
            }
            _ => None,
        }
    }
}
