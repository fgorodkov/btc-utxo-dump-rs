use bitcoin_bech32::constants::Network;
use bitcoin_bech32::{self, u5, WitnessProgram};
use bs58;
use ripemd::Ripemd160;
use sha2::{Digest, Sha256};

const TESTNET: Network = Network::Testnet;
const MAINNET: Network = Network::Bitcoin;

pub fn hash160_to_address(hash: &[u8], prefix: u8) -> String {
    // Pre-allocate with exact capacity to avoid reallocations
    let mut address = Vec::with_capacity(1 + hash.len() + 4);
    address.push(prefix);
    address.extend_from_slice(hash);
    // Calculate double SHA256
    let checksum = Sha256::digest(&Sha256::digest(&address)[..]);
    // Append checksum
    address.extend_from_slice(&checksum[..4]);
    // Convert to base58
    bs58::encode(address).into_string()
}

pub fn public_key_to_address(pubkey: &[u8], prefix: u8) -> String {
    // Perform SHA256 and RIPEMD160
    let hash160 = Ripemd160::digest(&Sha256::digest(pubkey));
    hash160_to_address(&hash160, prefix)
}

pub fn segwit_address(
    version: u8,
    program: &[u8],
    testnet: bool,
) -> Result<String, bitcoin_bech32::Error> {
    let network = if testnet { TESTNET } else { MAINNET };

    let version =
        u5::try_from_u8(version).map_err(|_| bitcoin_bech32::Error::InvalidScriptVersion)?;

    let witness_program = WitnessProgram::new(version, program.to_vec(), network)?;
    Ok(witness_program.to_address())
}
