use p2wsh_utxo::UTXO;
use std::collections::HashMap;
use std::env;
use std::{fs::File, io::BufWriter};

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct UtxoDump {
    txid: String,
    vout: u16,
    height: u32,
    amount: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct IterateDump {
    txid: String,
    tx_height: u16,
}

/*
#[derive(Serialize, Deserialize, Clone, Debug)]
struct ScrapeResult {
    //	block_hash: String,
    block_height: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    block_index: u16,
    transaction_index: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    amount: Option<usize>,
}
*/

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let utxo_dump_location = &args[1];
    let bitcoin_iterate_location = &args[2];
    let output_type = &args[3];
    let minimized_type = &args[4];
    let file_location = &args[5];

    let mut utxo_dump: HashMap<String, Vec<UtxoDump>> = HashMap::new();

    let mut utxo_dump_reader = csv::Reader::from_path(utxo_dump_location)?;

    println!("Dumping p2wsh utxo set...");
    for result in utxo_dump_reader.deserialize() {
        let utxo: UtxoDump = result?;

        // first see if it already exists
        match utxo_dump.get(utxo.clone().txid.as_str()) {
            Some(utxos) => {
                // if it exists, append
                let mut new_utxos = utxos.clone();
                new_utxos.push(utxo.clone());
                utxo_dump.insert(utxo.txid.to_string(), new_utxos);
            }
            None => {
                // if not exists, create as new
                utxo_dump.insert(utxo.txid.to_string(), vec![utxo]);
            }
        }
    }

    let mut bitcoin_iterate_reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(bitcoin_iterate_location)?;

    let mut scrape_results: Vec<UTXO> = vec![];

    let mut i = 0;
    let mut found = 0;
    let mut part = 1;
    println!("Analyzing all utxos...");
    for result in bitcoin_iterate_reader.deserialize() {
        let iterate: IterateDump = result?;
        i += 1;
        if i % 10_000_000 == 0 {
            println!("Analyzed {} utxos...", i);
        }

        match utxo_dump.get(&iterate.txid) {
            Some(utxos) => {
                for utxo in utxos.iter() {
                    let txid = if minimized_type == "full" {
                        Some(utxo.txid.to_string())
                    } else {
                        None
                    };
                    let amount = if minimized_type == "full" {
                        Some(utxo.amount)
                    } else {
                        None
                    };
                    let new_result = UTXO {
                        block_height: utxo.height,
                        id: txid,
                        block_index: iterate.tx_height,
                        transaction_index: utxo.vout,
                        amount,
                    };

                    scrape_results.push(new_result);
                    found += 1;

                    // println!("{}:{}", utxo.txid, utxo.vout);

                    if output_type == "json" {
                        if found % 10000 == 0 {
                            let writer = BufWriter::new(
                                File::create(format!("{}/part-{}.json", file_location, part))
                                    .unwrap(),
                            );
                            serde_json::to_writer_pretty(writer, &scrape_results).unwrap();
                            println!("Wrote 10000 transactions for part {}...", part);
                            part += 1;
                            scrape_results = vec![];
                        }
                    }
                }
            }
            None => continue,
        }
    }

    // At the very end, if there are not any cleared out results, save final file
    if output_type == "json" {
        let writer =
            BufWriter::new(File::create(format!("{}/part-{}.json", file_location, part)).unwrap());
        serde_json::to_writer_pretty(writer, &scrape_results).unwrap();
    } else if output_type == "bin" {
        // let binary_encoded = bincode::serialize(&scrape_results).unwrap();
        let writer =
            BufWriter::new(File::create(format!("{}/p2wsh-utxo.bin", file_location)).unwrap());
        bincode::serialize_into(writer, &scrape_results).unwrap();
    }

    println!(
        "Wrote {} transactions for part {}...",
        scrape_results.len(),
        part
    );
    println!("Wrote {} total transactions", found);

    return Ok(());
}
