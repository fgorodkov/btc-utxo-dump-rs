pub mod bitcoin;
pub mod utils;

use std::path::Path;

use bitcoin::chainstate::ChainStateDB;
use utils::cli::parse_cli;

fn main() -> anyhow::Result<()> {
    let cli = parse_cli()?;

    println!("Database path: {}", cli.db.display());
    println!("Output file: {}", cli.output.display());
    println!("Fields: {}", cli.fields);
    println!("Testnet: {}", cli.testnet);
    println!("Verbose: {}", cli.verbose);
    println!("Max UTXOs: {}", cli.max_utxos);
    println!("Quiet: {}", cli.quiet);

    let fields: std::collections::HashSet<_> = cli.fields.split(',')
        .map(str::trim)
        .collect();

    let mut utxo_count = 0;
    let mut obfuscation_key: Vec<u8> = Vec::new();

    let db = ChainStateDB::open(Path::new(&cli.db))?;

    for (key, value) in db.iter() {
        // Check the first byte of the key
        match key.first_byte() {
            // 0x43 ('C') prefix represents a UTXO entry
            0x43 => {
                if fields.contains("txid") {
                    // TODO: Implement txid field
                }
                if fields.contains("vout") {
                    // TODO: Implement vout field
                }
                if fields.contains("height") {
                    // TODO: Implement height field
                }
                if fields.contains("coinbase") {
                    // TODO: Implement coinbase field
                }
                if fields.contains("amount") {
                    // TODO: Implement amount field
                }
                if fields.contains("nsize") {
                    // TODO: Implement nsize field
                }
                if fields.contains("script") {
                    // TODO: Implement script field
                }
                if fields.contains("type") {
                    // TODO: Implement type field
                }
                if fields.contains("address") {
                    // TODO: Implement address field
                }
                utxo_count += 1;
                if utxo_count >= cli.max_utxos {
                    break;
                }
            }
            // 0x0E prefix represents an obfuscation key
            0x0E => {
                // Skip the first byte when storing the obfuscation key, it is the length of the key
                obfuscation_key = value[1..].to_vec();
            }
            _ => continue,
        }
    }

    Ok(())
}
