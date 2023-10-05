#![allow(improper_ctypes)]

mod data;
mod defaults;
mod types;

use std::collections::HashMap;
use data::{DataStructFork, OpenSeaAttributes};
use defaults::{DEFAULT_IPFS_MULTIADDR, DEFAULT_TIMEOUT_SEC, DEFAULT_COLLABEAT_URL};
use marine_rs_sdk::marine;
use marine_rs_sdk::module_manifest;
use marine_rs_sdk::MountedBinaryResult;
use marine_rs_sdk::WasmLoggerBuilder;
use types::MetaContract;
use types::Metadata;
use types::SerdeMetadata;
use types::Transaction;
use types::{ FinalMetadata, MetaContractResult, NousAiMetadata };
use ethabi::{decode, ParamType};

module_manifest!();

pub fn main() {
    WasmLoggerBuilder::new()
        .with_log_level(log::LevelFilter::Info)
        .build()
        .unwrap();
}

#[marine]
pub fn on_execute(
    contract: MetaContract,
    metadatas: Vec<Metadata>,
    transaction: Transaction,
) -> MetaContractResult {
    let mut finals: Vec<FinalMetadata> = vec![];
    let serde_metadata: Result<NousAiMetadata, serde_json::Error> = serde_json::from_str(&transaction.data.clone());

    match serde_metadata{
        Ok(metadata) => {
            if metadata.id.is_empty() {
                return MetaContractResult {
                    result:false,
                    metadatas: Vec::new(),
                    error_string: "id cannot be empty".to_string()
                }
            }
        }
        Err(_) => {
            return MetaContractResult {
                result: false,
                metadatas: Vec::new(),
                error_string: "Data does not follow the required JSON schema".to_string()
            }
        }

    }

    finals.push(FinalMetadata { 
        public_key: transaction.public_key,
        alias: transaction.alias,
        content: transaction.data,
        loose: 0,
        version: transaction.version
    });

    MetaContractResult {
        result: true,
        metadatas: finals,
        error_string: "".to_string(),
    }
}

#[marine]
pub fn on_clone() -> bool {
    return false;
}

#[marine]
pub fn on_mint(contract: MetaContract, data_key: String, token_id: String, data: String) -> MetaContractResult {
    let mut error: Option<String> = None;
    let mut finals: Vec<FinalMetadata> = vec![];
    // extract out data

    MetaContractResult {
        result: true,
        metadatas: finals,
        error_string: "".to_string(),
    }
}


/**
 * Get data from ipfs
 */
fn get(hash: String, api_multiaddr: String, timeout_sec: u64) -> String {
    let address: String;
    let t;

    if api_multiaddr.is_empty() {
        address = DEFAULT_IPFS_MULTIADDR.to_string();
    } else {
        address = api_multiaddr;
    }

    if timeout_sec == 0 {
        t = DEFAULT_TIMEOUT_SEC;
    } else {
        t = timeout_sec;
    }

    let args = vec![String::from("dag"), String::from("get"), hash];

    let cmd = make_cmd_args(args, address, t);

    let result = ipfs(cmd);

    String::from_utf8(result.stdout).unwrap()
}

pub fn make_cmd_args(args: Vec<String>, api_multiaddr: String, timeout_sec: u64) -> Vec<String> {
    args.into_iter()
        .chain(vec![
            String::from("--timeout"),
            get_timeout_string(timeout_sec),
            String::from("--api"),
            api_multiaddr,
        ])
        .collect()
}

#[inline]
pub fn get_timeout_string(timeout: u64) -> String {
    format!("{}s", timeout)
}

// Service
// - curl

#[marine]
#[link(wasm_import_module = "host")]
extern "C" {
    pub fn ipfs(cmd: Vec<String>) -> MountedBinaryResult;
}
