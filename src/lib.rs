use serde::{Deserialize, Serialize};

/// Maximum vout index that can be used in a `short_channel_id`. This
/// value is based on the 2-bytes available for the vout index.
pub const MAX_SCID_VOUT_INDEX: u64 = 0xffff;

static P2WSH_BIN: &'static [u8] = include_bytes!("p2wsh-utxo.bin");

/// UTXO represents an Unspent transaction Output.
/// `id` or `amount` might not be filled in if the
/// results were generated with minimized data.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UTXO {
    pub block_height: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub block_index: u16,
    pub transaction_index: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<usize>,
}

impl UTXO {
    /// This generates what the short channel id
    /// should be if this was used inside a lightning
    /// channel.
    pub fn scid(&self) -> u64 {
        scid_from_parts(
            self.block_height as u64,
            self.block_index as u64,
            self.transaction_index as u64,
        )
    }
}

fn scid_from_parts(block: u64, tx_index: u64, vout_index: u64) -> u64 {
    (block << 40) | (tx_index << 16) | vout_index
}

/// UTXOResults holds a vector of `UTXO`s that represent P2WSH
/// bitcoin transactions that have not been spent yet.
///
/// The results may not be quite up to date. Some of the transactions
/// might have been spent and new ones may have been created.
/// Do not rely on it being exact.
pub struct UTXOResults {
    results: Vec<UTXO>,
}

impl UTXOResults {
    /// Returns a UTXOResults structure that is already initialized
    /// with unspent P2WSH transactions.
    pub fn new() -> UTXOResults {
        let decoded: Option<Vec<UTXO>> = bincode::deserialize(P2WSH_BIN).unwrap();
        UTXOResults {
            results: decoded.unwrap(),
        }
    }

    /// Returns a clone of the vector of the P2WSH UTXOs.
    pub fn results(&self) -> Vec<UTXO> {
        self.results.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scid_test() {
        let utxo1 = UTXO {
            id: None,
            block_height: 691709,
            block_index: 1160,
            transaction_index: 1,
            amount: None,
        };

        assert_eq!(760542088613330945, utxo1.scid());
    }

    #[test]
    fn returns_results() {
        let utxo1 = UTXO {
            id: None,
            block_height: 1,
            block_index: 1,
            transaction_index: 0,
            amount: None,
        };

        let results = vec![utxo1.clone()];

        let utxo_results = UTXOResults { results };
        assert_eq!(utxo_results.results()[0], utxo1.clone());
    }
}
