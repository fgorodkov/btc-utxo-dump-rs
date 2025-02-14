use std::collections::HashSet;

pub const FIELD_COUNT: &str = "count";
pub const FIELD_TXID: &str = "txid";
pub const FIELD_VOUT: &str = "vout";
pub const FIELD_HEIGHT: &str = "height";
pub const FIELD_COINBASE: &str = "coinbase";
pub const FIELD_AMOUNT: &str = "amount";
pub const FIELD_NSIZE: &str = "nsize";
pub const FIELD_SCRIPT: &str = "script";
pub const FIELD_TYPE: &str = "type";
pub const FIELD_ADDRESS: &str = "address";

#[derive(Debug)]
pub struct SelectedFields {
    pub txid: bool,
    pub vout: bool,
    pub height: bool,
    pub coinbase: bool,
    pub amount: bool,
    pub nsize: bool,
    pub script: bool,
    pub script_type: bool,
    pub address: bool,
    pub count: bool,
}

impl SelectedFields {
    pub fn from_str(fields: &str) -> Self {
        let fields: HashSet<_> = fields.split(',').map(str::trim).collect();
        Self {
            txid: fields.contains(FIELD_TXID),
            vout: fields.contains(FIELD_VOUT),
            height: fields.contains(FIELD_HEIGHT),
            coinbase: fields.contains(FIELD_COINBASE),
            amount: fields.contains(FIELD_AMOUNT),
            nsize: fields.contains(FIELD_NSIZE),
            script: fields.contains(FIELD_SCRIPT),
            script_type: fields.contains(FIELD_TYPE),
            address: fields.contains(FIELD_ADDRESS),
            count: fields.contains(FIELD_COUNT),
        }
    }

    pub fn needs_utxo_parsing(&self) -> bool {
        self.height
            || self.coinbase
            || self.amount
            || self.nsize
            || self.script
            || self.script_type
            || self.address
    }

    pub fn needs_decompression(&self, include_p2pk: bool) -> bool {
        self.script || (self.address && include_p2pk)
    }
}
