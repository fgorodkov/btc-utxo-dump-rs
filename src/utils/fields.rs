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

pub struct FieldIndices {
    pub count: Option<usize>,
    pub txid: Option<usize>,
    pub vout: Option<usize>,
    pub height: Option<usize>,
    pub coinbase: Option<usize>,
    pub amount: Option<usize>,
    pub nsize: Option<usize>,
    pub script: Option<usize>,
    pub script_type: Option<usize>,
    pub address: Option<usize>,
}

impl FieldIndices {
    pub fn from_str(fields: &str) -> Self {
        let fields: Vec<_> = fields.split(',').collect();
        Self {
            count: fields.iter().position(|&f| f == FIELD_COUNT),
            txid: fields.iter().position(|&f| f == FIELD_TXID),
            vout: fields.iter().position(|&f| f == FIELD_VOUT),
            height: fields.iter().position(|&f| f == FIELD_HEIGHT),
            coinbase: fields.iter().position(|&f| f == FIELD_COINBASE),
            amount: fields.iter().position(|&f| f == FIELD_AMOUNT),
            nsize: fields.iter().position(|&f| f == FIELD_NSIZE),
            script: fields.iter().position(|&f| f == FIELD_SCRIPT),
            script_type: fields.iter().position(|&f| f == FIELD_TYPE),
            address: fields.iter().position(|&f| f == FIELD_ADDRESS),
        }
    }
    pub fn needs_utxo_parsing(&self) -> bool {
        self.height.is_some()
            || self.coinbase.is_some()
            || self.amount.is_some()
            || self.nsize.is_some()
            || self.script.is_some()
            || self.script_type.is_some()
            || self.address.is_some()
    }
    pub fn needs_decompression(&self, include_p2pk: bool) -> bool {
        self.script.is_some() || (self.address.is_some() && include_p2pk)
    }
}
