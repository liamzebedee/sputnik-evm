

use evm::backend::sql::{MemoryAccount, MemoryBackend, MemoryVicinity};
// use evm::backend::memory::{MemoryAccount, MemoryBackend, MemoryVicinity};
use std::fs;
use evm::executor::stack::{MemoryStackState, StackExecutor, StackSubstateMetadata, StackState};
use evm::executor::{Executor};
use evm::Config;
use primitive_types::{H160, U256};
use std::fmt::Debug;
use std::{collections::BTreeMap, str::FromStr};
use evm::backend::ApplyBackend;
use std::env;
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;

// Backend
const DATABASE_FILE: &str = "file:chain.sqlite?cache=shared";
const VERSION: &str = "0.0.2";


fn execute_in_vm(
	params: SendTransactionParams,
	write: bool,
	output_file: &Path,
	db_path: &Path,
	state_leaves_file: &Path,
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

	let mut backend = MemoryBackend::new(&vicinity, bstate, db_path.to_str().unwrap().to_string(), db_genesis);
	// let mut backend = MemoryBackend::new(&vicinity, bstate);
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
		let mut file = File::create(output_file).unwrap();
		file.write_all(&output).unwrap();
	}

	let state = executor.into_state();
	
	let accessed = state.metadata().accessed().as_ref().unwrap();
	
	let ad = accessed
		.accessed_addresses
		.iter()
		.map(|a| format!("address {:?}", H160(a.0)))
		.collect::<Vec<String>>()
		.join("\n");
	
	let st = accessed
		.accessed_storage
		.iter()
		.map(|a| format!("storage {:?} {:?}", a.0, a.1))
		.collect::<Vec<String>>()
		.join("\n");
	
	println!("{}", ad);
	println!("{}", st);
	let state_leaves_content = format!("{}\n{}", ad, st);
	fs::write(state_leaves_file, state_leaves_content).expect("Unable to write file");

	dbg!("{}", &accessed.accessed_addresses, &accessed.accessed_storage);

	// let mut file = File::create(state_leaves_file).unwrap();
	// file.write_all(&output).unwrap();

	let (applies, logs) = state.deconstruct();

	// let mut file = File::create(output_file).unwrap();
	// file.write_all(&output).unwrap();

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
        help = "A path to write the state leaves accessed during a tx.",
        long,
        value_hint = ValueHint::FilePath
    )]
    pub state_leaves_file: PathBuf,

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
	execute_in_vm(params, args.write, &args.output_file, &args.db_path.into_boxed_path(), &args.state_leaves_file);

	Ok(0)
}

fn main() {
    run().unwrap();
}
