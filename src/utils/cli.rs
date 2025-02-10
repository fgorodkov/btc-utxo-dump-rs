use clap::Parser;
use std::path::PathBuf;

pub const VALID_FIELDS: [&str; 10] = [
    "count", "txid", "vout", "height", "coinbase", "amount", "nsize", "script", "type", "address",
];

#[derive(Parser)]
#[command(
    name = "btc-utxo-dump",
    about = "Dumps Bitcoin Core's chainstate UTXO set to CSV",
    version,
    author
)]
pub struct Cli {
    #[arg(
        short,
        long,
        value_name = "DIR",
        default_value = "~/.bitcoin/chainstate/",
        help = "Bitcoin Core chainstate LevelDB directory"
    )]
    pub db: PathBuf,

    #[arg(
        short,
        long,
        value_name = "FILE",
        default_value = "utxodump.csv",
        help = "Output CSV file path"
    )]
    pub output: PathBuf,

    #[arg(
        short,
        long,
        value_name = "FIELDS",
        default_value = "count,txid,vout,amount,type,address",
        help = "Comma-separated list of fields to include in output: count,txid,vout,height,coinbase,amount,nsize,script,type,address"
    )]
    pub fields: String,

    #[arg(long, help = "Use testnet network parameters", default_value = "false")]
    pub testnet: bool,

    #[arg(
        short,
        long,
        help = "Print UTXOs to stdout while processing",
        default_value = "false"
    )]
    pub verbose: bool,

    #[arg(
        short,
        long,
        value_name = "COUNT",
        default_value = "0",
        help = "Maximum number of UTXOs to process (0 for unlimited)"
    )]
    pub max_utxos: u64,

    #[arg(short, long, help = "Quiet mode, no output", default_value = "false")]
    pub quiet: bool,
}

impl Cli {
    pub fn validate(&self) -> anyhow::Result<()> {
        for field in self.fields.split(',') {
            if !VALID_FIELDS.contains(&field.trim()) {
                anyhow::bail!("Invalid field: {}", field);
            }
        }
        Ok(())
    }
}

pub fn parse_cli() -> anyhow::Result<Cli> {
    let cli = Cli::parse();
    cli.validate()?;
    Ok(cli)
}
