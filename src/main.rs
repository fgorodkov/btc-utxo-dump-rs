pub mod bitcoin;
pub mod utils;

use csv::WriterBuilder;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use bitcoin::chainstate::{ChainStateDB, ChainStateValue};
use bitcoin::utxo::UtxoValue;
use utils::cli::parse_cli;

fn main() -> anyhow::Result<()> {
    let cli = parse_cli()?;
    let db = ChainStateDB::open(Path::new(&cli.db))?;

    let fields: std::collections::HashSet<_> = cli.fields.split(',').map(str::trim).collect();

    let mut utxo_count = 0;
    let mut obfuscation_key: Vec<u8> = Vec::new();
    let mut output = HashMap::new();

    let outfile = File::create(&cli.output)?;
    let mut writer = WriterBuilder::new().has_headers(true).from_writer(outfile);
    // Write headers
    writer.write_record(&cli.fields.split(',').collect::<Vec<_>>())?;

    for (key, value) in db.iter() {
        output.clear();
        match key.first_byte() {
            // 0x43 ('C') prefix represents a UTXO entry
            0x43 => {
                if fields.contains("txid") {
                    output.insert("txid".to_string(), key.txid());
                }
                if fields.contains("vout") {
                    output.insert("vout".to_string(), key.vout().to_string());
                }
                if fields.contains("height")
                    || fields.contains("coinbase")
                    || fields.contains("amount")
                    || fields.contains("nsize")
                    || fields.contains("script")
                    || fields.contains("type")
                    || fields.contains("address")
                {
                    let deobfuscated = ChainStateValue::new(value).deobfuscate(&obfuscation_key);
                    let utxo = UtxoValue::parse(&deobfuscated)?;

                    if fields.contains("height") {
                        output.insert("height".to_string(), utxo.height.to_string());
                    }
                    if fields.contains("coinbase") {
                        output.insert("coinbase".to_string(), if utxo.coinbase { "1" } else { "0" }.to_string());
                    }
                    if fields.contains("amount") {
                        output.insert("amount".to_string(), utxo.amount.to_string());
                    }
                    if fields.contains("nsize") {
                        output.insert("nsize".to_string(), utxo.script_type.to_string());
                    }
                    if fields.contains("script") {
                        output.insert("script".to_string(), hex::encode(&utxo.script));
                    }
                    if fields.contains("type") {
                        output.insert(
                            "type".to_string(),
                            format!("{:?}", utxo.get_script_type()).to_lowercase(),
                        );
                    }
                    if fields.contains("address") {
                        if let Some(addr) = utxo.get_address(cli.testnet, cli.include_p2pk) {
                            output.insert("address".to_string(), addr);
                        }
                    }
                }

                utxo_count += 1;

                if fields.contains("count") {
                    output.insert("count".to_string(), utxo_count.to_string());
                }

                let record: Vec<_> = cli
                    .fields
                    .split(',')
                    .map(|field| output.remove(field).unwrap_or_default())
                    .collect();
                writer.write_record(&record)?;

                if cli.verbose {
                    println!("{}", record.join(" "));
                } else if !cli.quiet && utxo_count % 100_000 == 0 {
                    println!("{} utxos processed", utxo_count);
                }
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

    writer.flush()?;

    if !cli.quiet {
        println!("\nTotal UTXOs: {}", utxo_count);
    }

    Ok(())
}
