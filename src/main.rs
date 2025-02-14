pub mod bitcoin;
pub mod utils;

use csv::WriterBuilder;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use bitcoin::chainstate::{ChainStateDB, ChainStateValue};
use bitcoin::utxo::UtxoValue;
use utils::cli::parse_cli;
use utils::fields::SelectedFields;
use utils::fields::{
    FIELD_ADDRESS, FIELD_AMOUNT, FIELD_COINBASE, FIELD_COUNT, FIELD_HEIGHT, FIELD_NSIZE,
    FIELD_SCRIPT, FIELD_TXID, FIELD_TYPE, FIELD_VOUT,
};

const UTXO_PREFIX: u8 = 0x43;
const OBFUSCATION_KEY_PREFIX: u8 = 0x0E;
const REPORT_PROGRESS_INTERVAL: u64 = 100_000;

fn main() -> anyhow::Result<()> {
    let cli = parse_cli()?;
    let db = ChainStateDB::open(Path::new(&cli.db))?;

    let selected = SelectedFields::from_str(&cli.fields);

    let mut utxo_count = 0;
    let mut obfuscation_key: Vec<u8> = Vec::new();
    let mut output = HashMap::new();

    let outfile = File::create(&cli.output)?;
    let mut writer = WriterBuilder::new()
        .has_headers(true)
        .buffer_capacity(2048 * 1024)
        .from_writer(outfile);
    // Write headers
    writer.write_record(&cli.fields.split(',').collect::<Vec<_>>())?;

    for (key, value) in db.iter() {
        output.clear();
        match key.first_byte() {
            UTXO_PREFIX => {
                utxo_count += 1;

                if cli.max_utxos > 0 && utxo_count >= cli.max_utxos {
                    break;
                }

                if selected.txid {
                    output.insert(FIELD_TXID.to_string(), key.txid());
                }
                if selected.vout {
                    output.insert(FIELD_VOUT.to_string(), key.vout().to_string());
                }
                if selected.needs_utxo_parsing() {
                    let deobfuscated = ChainStateValue::new(value).deobfuscate(&obfuscation_key);
                    let needs_decompression = selected.needs_decompression(cli.include_p2pk);
                    let utxo = UtxoValue::parse(&deobfuscated, needs_decompression)?;

                    if selected.height {
                        output.insert(FIELD_HEIGHT.to_string(), utxo.height.to_string());
                    }
                    if selected.coinbase {
                        output.insert(
                            FIELD_COINBASE.to_string(),
                            if utxo.coinbase { "1" } else { "0" }.to_string(),
                        );
                    }
                    if selected.amount {
                        output.insert(FIELD_AMOUNT.to_string(), utxo.amount.to_string());
                    }
                    if selected.nsize {
                        output.insert(FIELD_NSIZE.to_string(), utxo.script_type.to_string());
                    }
                    if selected.script {
                        output.insert(FIELD_SCRIPT.to_string(), hex::encode(&utxo.script));
                    }
                    if selected.script_type || selected.address {
                        let script_type = utxo.get_script_type();
                        if selected.script_type {
                            output.insert(
                                FIELD_TYPE.to_string(),
                                format!("{:?}", script_type).to_lowercase(),
                            );
                        }
                        if selected.address {
                            if let Some(addr) = utxo.get_address_with_type(
                                script_type,
                                cli.testnet,
                                cli.include_p2pk,
                            ) {
                                output.insert(FIELD_ADDRESS.to_string(), addr);
                            }
                        }
                    }
                }
                if selected.count {
                    output.insert(FIELD_COUNT.to_string(), utxo_count.to_string());
                }

                let record: Vec<_> = cli
                    .fields
                    .split(',')
                    .map(|field| output.remove(field).unwrap_or_default())
                    .collect();
                writer.write_record(&record)?;

                if cli.verbose {
                    println!("{}", record.join(" "));
                } else if !cli.quiet && utxo_count % REPORT_PROGRESS_INTERVAL == 0 {
                    println!("{} utxos processed", utxo_count);
                }
            }
            OBFUSCATION_KEY_PREFIX => {
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
