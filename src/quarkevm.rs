

use evm::backend::sql::{MemoryAccount, MemoryBackend, MemoryVicinity};
use evm::executor::stack::{MemoryStackState, StackExecutor, StackSubstateMetadata};
use evm::executor::{Executor};
use evm::Config;
use primitive_types::{H160, U256};
use std::{collections::BTreeMap, str::FromStr};
use evm::backend::ApplyBackend;
use std::env;
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;

// Backend
const DATABASE_FILE: &str = "chain.sqlite";
const VERSION: &str = "0.0.2";


fn execute_in_vm(
	params: SendTransactionParams,
	write: bool,
	output_file: &Path
) {

	let config = Config::istanbul();

	let vicinity = MemoryVicinity {
		gas_price: U256::zero(),
		origin: H160::default(),
		block_hashes: Vec::new(),
		block_number: Default::default(),
		block_coinbase: Default::default(),
		block_timestamp: Default::default(),
		block_difficulty: Default::default(),
		block_gas_limit: Default::default(),
		chain_id: U256::one(),
		block_base_fee_per_gas: U256::zero(),
	};
	
	let mut bstate = BTreeMap::new();
	
	let db_genesis = match env::var("DB_GENESIS") {
		Ok(val) => match val.as_str() {
			"1" => true,
			_ => false
		},
		_ => false
	};

	if db_genesis {
		bstate.insert(
			H160::from_str("0xf000000000000000000000000000000000000000").unwrap(),
			MemoryAccount {
				nonce: U256::one(),
				balance: U256::from(10000000),
				storage: BTreeMap::new(),
				code: Vec::new(),
			},
		);
	}

	println!("quarkevm version {}", VERSION);

	let mut backend = MemoryBackend::new(&vicinity, bstate, DATABASE_FILE.to_string(), db_genesis);
	let metadata = StackSubstateMetadata::new(u64::MAX, &config);
	let state = MemoryStackState::new(metadata, &backend);
	let precompiles = BTreeMap::new();
	let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

	if params.to == "" {
		// Create call.
		let _reason = executor.transact_create(
			H160::from_str(&params.from).unwrap(),
			// H160::from_str(&params.value).unwrap(),
			U256::zero(), // value
			hex::decode(&params.data)
				.unwrap(),
			100000000000u64, // gas limit
			Vec::new(), // access list
		);
		
		println!("{:?}", _reason);

	} else {
		let (_reason, output) = executor.transact_call(
			H160::from_str(&params.from).unwrap(),
			H160::from_str(&params.to).unwrap(),
			U256::zero(), // value
			hex::decode(&params.data)
				.unwrap(),
			100000000000u64, // gas limit
			Vec::new(),
		);
		println!("{:?} {:?}", _reason, output);
		let mut file = File::create(output_file).unwrap();
		file.write_all(&output).unwrap();
	}

	// LEARN: 
	// Why does into_state work here, but state_mut() doesn't?
	// 	error[E0507]: cannot move out of a mutable reference
	//    --> src/quarkevm.rs:120:24
	//     |
	// 120 |     let (applies, logs) = executor.state_mut().deconstruct();
	//     |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ move occurs because value has type `MemoryStackState<'_, '_, evm::backend::sql::MemoryBackend<'_>>`, which does not implement the `Copy` trait
	let (applies, logs) = executor.into_state().deconstruct();

	// dbg!(&logs.into_iter()
    //     .map(|item| format!("{item}"))
    //     .collect::<String>());
	if write {
		println!("Applying updates");
		backend.apply(applies, logs, false);
	}

	println!("Done!");
}




#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SendTransactionParams {
	from: String,
	to: String,
	gas: Option<String>,
	
	#[serde(rename = "gasPrice")]
	gas_price: Option<String>,
	
	value: Option<String>,
	data: String
}

use std::{path::PathBuf};
use serde::{Deserialize, Serialize};
use serde_json::{Result};
use clap::{Parser, ValueHint};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(
        help = "A path to the database.",
        long,
        value_hint = ValueHint::FilePath
    )]
    pub db_path: PathBuf,

    #[clap(
        help = "A path to write the raw output.",
        long,
        value_hint = ValueHint::FilePath
    )]
    pub output_file: PathBuf,


    #[clap(
        help = "If present, will flush writes to the DB.",
        long
    )]
    pub write: bool,

    #[clap(short, long)]
    pub data: String,
}

fn run() -> Result<u8> {
	let args = Args::parse();

	// Decode args.
	let mut params: SendTransactionParams = serde_json::from_str(&args.data)?;
	params.from = params.from.strip_prefix("0x").unwrap().to_string();
	params.to = params.to.strip_prefix("0x").unwrap().to_string();
	params.data = params.data.strip_prefix("0x").unwrap().to_string();

	// Execute.
	execute_in_vm(params, args.write, &args.output_file);

	Ok(0)
}

fn main() {
    run().unwrap();
}
