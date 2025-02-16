pub mod bitcoin;
pub mod utils;

use csv::WriterBuilder;
use std::fmt::Write;
use std::fs::File;
use std::path::Path;

use bitcoin::chainstate::{ChainStateDB, ChainStateValue};
use bitcoin::utxo::UtxoValue;
use utils::cli::parse_cli;
use utils::fields::FieldIndices;

const UTXO_PREFIX: u8 = 0x43;
const OBFUSCATION_KEY_PREFIX: u8 = 0x0E;
const REPORT_PROGRESS_INTERVAL: u64 = 100_000;

fn main() -> anyhow::Result<()> {
    let cli = parse_cli()?;
    let db = ChainStateDB::open(Path::new(&cli.db))?;

    let indices = FieldIndices::from_str(&cli.fields);

    let mut utxo_count = 0;
    let mut obfuscation_key: Vec<u8> = Vec::new();

    let field_count = cli.fields.split(',').count();
    let mut record = vec![String::new(); field_count];

    let outfile = File::create(&cli.output)?;
    let mut writer = WriterBuilder::new()
        .has_headers(true)
        .buffer_capacity(2048 * 1024)
        .from_writer(outfile);

    // Write headers
    writer.write_record(&cli.fields.split(',').collect::<Vec<_>>())?;

    for (key, value) in db.iter() {
        for string in record.iter_mut() {
            string.clear();
        }
        match key.first_byte() {
            UTXO_PREFIX => {
                utxo_count += 1;

                if cli.max_utxos > 0 && utxo_count >= cli.max_utxos {
                    break;
                }
                if let Some(txid_pos) = indices.txid {
                    record[txid_pos].push_str(&key.txid());
                }
                if let Some(vout_pos) = indices.vout {
                    write!(&mut record[vout_pos], "{}", key.vout()).unwrap();
                }
                if indices.needs_utxo_parsing() {
                    let deobfuscated = ChainStateValue::new(value).deobfuscate(&obfuscation_key);
                    let needs_decompression = indices.needs_decompression(cli.include_p2pk);
                    let utxo = UtxoValue::parse(&deobfuscated, needs_decompression)?;

                    if let Some(height_pos) = indices.height {
                        write!(&mut record[height_pos], "{}", utxo.height).unwrap();
                    }
                    if let Some(coinbase_pos) = indices.coinbase {
                        record[coinbase_pos].push(if utxo.coinbase { '1' } else { '0' });
                    }
                    if let Some(amount_pos) = indices.amount {
                        write!(&mut record[amount_pos], "{}", utxo.amount).unwrap();
                    }
                    if let Some(nsize_pos) = indices.nsize {
                        write!(&mut record[nsize_pos], "{}", utxo.script_type).unwrap();
                    }
                    if let Some(script_pos) = indices.script {
                        record[script_pos].push_str(&hex::encode(&utxo.script));
                    }
                    if indices.script_type.is_some() || indices.address.is_some() {
                        let script_type = utxo.get_script_type();
                        if let Some(type_pos) = indices.script_type {
                            write!(&mut record[type_pos], "{}", script_type).unwrap();
                        }
                        if let Some(address_pos) = indices.address {
                            if let Some(addr) = utxo.get_address_with_type(
                                script_type,
                                cli.testnet,
                                cli.include_p2pk,
                            ) {
                                record[address_pos].push_str(&addr);
                            }
                        }
                    }
                }
                if let Some(count_pos) = indices.count {
                    write!(&mut record[count_pos], "{}", utxo_count).unwrap();
                }

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
