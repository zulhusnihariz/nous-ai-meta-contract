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
use types::{FinalMetadata, MetaContractResult};
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
    // let new_metadatas = metadatas.clone();
    let mut hash_map = HashMap::new();

    // Only 10 beat
    for data in metadatas {
      if data.alias == "" {
        hash_map.insert(data.public_key, data.cid);
      }
    }

    if transaction.alias == "" {
      hash_map.insert(transaction.public_key.clone(), transaction.data.clone());
    }

    let no_beats = hash_map.len() as i32;

    if no_beats > 10 {
        return MetaContractResult {
            result: false,
            metadatas: finals,
            error_string: "Can not be more than 10 beats".to_string(),
        };
    }
    
    finals.push(FinalMetadata {
        public_key: contract.public_key.clone(),
        alias: "name".to_string(),
        content: format!("Collabeat #{}", transaction.token_id),
        loose: 1,
        version: "".to_string(),
    });

    finals.push(FinalMetadata {
        public_key: contract.public_key.clone(),
        alias: "description".to_string(),
        content: "Co-Create, Collaborate and Own The Beat".to_string(),
        loose: 1,
        version: "".to_string(),
    });

    finals.push(FinalMetadata {
        public_key: contract.public_key.clone(),
        alias: "image".to_string(),
        content: "ipfs://".to_string(),
        loose: 1,
        version: "".to_string(),
    });

    finals.push(FinalMetadata {
        public_key: contract.public_key.clone(),
        alias: "external_url".to_string(),
        content: format!("{}{}", DEFAULT_COLLABEAT_URL, transaction.data_key),
        loose: 1,
        version: "".to_string(),
    });

    finals.push(FinalMetadata {
        public_key: contract.public_key.clone(),
        alias: "animation_url".to_string(),
        content: format!("{}{}", DEFAULT_COLLABEAT_URL, transaction.data_key),
        loose: 1,
        version: "".to_string(),
    });

    let attr = vec![{
      OpenSeaAttributes { display_type: "number".to_string(), trait_type: "No of Beats".to_string(), value: no_beats}
    }];

    finals.push(FinalMetadata {
        public_key: contract.public_key.clone(),
        alias: "attributes".to_string(),
        content: serde_json::to_string(&attr).unwrap(),
        loose: 1,
        version: "".to_string(),
    });
    
    // let serde_metadata: Result<SerdeMetadata, serde_json::Error> = serde_json::from_str(&transaction.mcdata.clone());
    // let mut loose;

    // match serde_metadata {
    //   Ok(sm) => loose = sm.loose,
    //   _ => loose = 1,
    // }
    // finals.push(FinalMetadata {
    //     public_key: transaction.public_key,
    //     alias: transaction.alias,
    //     content: transaction.data,
    //     loose,
    // });

    MetaContractResult {
        result: true,
        metadatas: finals,
        error_string: "".to_string(),
    }
}

#[marine]
pub fn on_clone() -> bool {
    return true;
}

#[marine]
pub fn on_mint(contract: MetaContract, data_key: String, token_id: String, data: String) -> MetaContractResult {
    let mut name = format!("Collabeat #{}", token_id);
    let mut error: Option<String> = None;
    let mut finals: Vec<FinalMetadata> = vec![];
    let mut no_beats = 0;
    let mut hash_map: HashMap<String, String> = HashMap::new();

    finals.push(FinalMetadata {
        public_key: contract.public_key.clone(),
        alias: "description".to_string(),
        content: "Co-Create, Collaborate and Own The Beat".to_string(),
        loose: 1,
        version: "".to_string(),
    });

    finals.push(FinalMetadata {
        public_key: contract.public_key.clone(),
        alias: "image".to_string(),
        content: "ipfs://".to_string(),
        loose: 1,
        version: "".to_string(),
    });

    // extract out data
    if data.len() > 0 {

        let data_bytes = &hex::decode(&data);

        match data_bytes {
          Ok(decoded) => {
            let param_types = vec![
              ParamType::String,
              ParamType::String,
              ParamType::String,
            ];

            let results = decode(&param_types, decoded);

            match results {
              Ok(result) => {
                if result.len() == 3 {
                  let new_name = result[0].clone().to_string();

                  if new_name.len() > 0 {
                    name = format!("{}", new_name.clone());
                  }
                  let ipfs_multiaddr = result[1].clone().to_string();
                  let cid = result[2].clone().to_string();
                  
                  let datasets = get(cid, ipfs_multiaddr, 0);
                  let result: Result<Vec<DataStructFork>, serde_json::Error> =
                      serde_json::from_str(&datasets);
    
                  match result {
                      Ok(datas) => {
                          no_beats = datas.clone().len() as i32;
                      }
                      Err(e) => error = Some(format!("Invalid data structure: {}", e.to_string())),
                  }
                }
              },
              Err(e) => error = Some(format!("Invalid data structure: {}", e.to_string())),
            }
          },
          Err(e) => error = Some(format!("Invalid data structure: {}", e.to_string())),
        }
    }

    if !error.is_none() {
      return MetaContractResult {
        result: false,
        metadatas: Vec::new(),
        error_string: error.unwrap().to_string(),
      };
    }
    // adds attributes
    let attr = vec![{
      OpenSeaAttributes { display_type: "number".to_string(), trait_type: "No of Beats".to_string(), value: no_beats}
    }];
    finals.push(FinalMetadata {
        public_key: contract.public_key.clone(),
        alias: "attributes".to_string(),
        content: serde_json::to_string(&attr).unwrap(),
        loose: 1,
        version: "".to_string(), 
    });

    finals.push(FinalMetadata {
        public_key: contract.public_key.clone(),
        alias: "name".to_string(),
        content: name,
        loose: 1,
        version: "".to_string(),
    });

    finals.push(FinalMetadata {
        public_key: contract.public_key.clone(),
        alias: "animation_url".to_string(),
        content: format!("{}{}", DEFAULT_COLLABEAT_URL, data_key),
        loose: 1,
        version: "".to_string(),
    });

    finals.push(FinalMetadata {
        public_key: contract.public_key.clone(),
        alias: "external_url".to_string(),
        content: format!("{}{}", DEFAULT_COLLABEAT_URL, data_key),
        loose: 1,
        version: "".to_string(),
    });

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
