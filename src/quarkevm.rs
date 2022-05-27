



use evm::backend::{MemoryAccount, MemoryBackend, MemoryVicinity};
use evm::executor::stack::{MemoryStackState, StackExecutor, StackSubstateMetadata};
use evm::executor::{Executor};
use evm::Config;
use primitive_types::{H160, U256};
use std::{collections::BTreeMap, str::FromStr};
use std::path::Path;

use std::sync::{Arc, Mutex};

use sqlite;



// Backend
const DATABASE_FILE: &str = "chain.sqlite";
const VERSION: &str = "0.0.1";


/*
mock state 


let mut state = BTreeMap::new();
	state.insert(
		H160::from_str("0x1000000000000000000000000000000000000000").unwrap(),
		MemoryAccount {
			nonce: U256::one(),
			balance: U256::from(10000000),
			storage: BTreeMap::new(),
			code: hex::decode("608060405234801561001057600080fd5b50600436106100365760003560e01c8063a41368621461003b578063cfae321714610057575b600080fd5b6100556004803603810190610050919061022c565b610075565b005b61005f61008f565b60405161006c91906102a6565b60405180910390f35b806000908051906020019061008b929190610121565b5050565b60606000805461009e9061037c565b80601f01602080910402602001604051908101604052809291908181526020018280546100ca9061037c565b80156101175780601f106100ec57610100808354040283529160200191610117565b820191906000526020600020905b8154815290600101906020018083116100fa57829003601f168201915b5050505050905090565b82805461012d9061037c565b90600052602060002090601f01602090048101928261014f5760008555610196565b82601f1061016857805160ff1916838001178555610196565b82800160010185558215610196579182015b8281111561019557825182559160200191906001019061017a565b5b5090506101a391906101a7565b5090565b5b808211156101c05760008160009055506001016101a8565b5090565b60006101d76101d2846102ed565b6102c8565b9050828152602081018484840111156101ef57600080fd5b6101fa84828561033a565b509392505050565b600082601f83011261021357600080fd5b81356102238482602086016101c4565b91505092915050565b60006020828403121561023e57600080fd5b600082013567ffffffffffffffff81111561025857600080fd5b61026484828501610202565b91505092915050565b60006102788261031e565b6102828185610329565b9350610292818560208601610349565b61029b8161043d565b840191505092915050565b600060208201905081810360008301526102c0818461026d565b905092915050565b60006102d26102e3565b90506102de82826103ae565b919050565b6000604051905090565b600067ffffffffffffffff8211156103085761030761040e565b5b6103118261043d565b9050602081019050919050565b600081519050919050565b600082825260208201905092915050565b82818337600083830152505050565b60005b8381101561036757808201518184015260208101905061034c565b83811115610376576000848401525b50505050565b6000600282049050600182168061039457607f821691505b602082108114156103a8576103a76103df565b5b50919050565b6103b78261043d565b810181811067ffffffffffffffff821117156103d6576103d561040e565b5b80604052505050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052602260045260246000fd5b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b6000601f19601f830116905091905056fea264697066735822122070d157c4efbb3fba4a1bde43cbba5b92b69f2fc455a650c0dfb61e9ed3d4bd6364736f6c63430008040033").unwrap(),
		}
	);
	state.insert(
		H160::from_str("0xf000000000000000000000000000000000000000").unwrap(),
		MemoryAccount {
			nonce: U256::one(),
			balance: U256::from(10000000),
			storage: BTreeMap::new(),
			code: Vec::new(),
		},
	);

*/

fn create_schema() {
	// connection
    // .execute(
    //     "
    //     CREATE TABLE users (name TEXT, age INTEGER);
    //     INSERT INTO users VALUES ('Alice', 42);
    //     INSERT INTO users VALUES ('Bob', 69);
    //     ",
    // )
    // .unwrap();
}

fn execute_in_vm(
	params: SendTransactionParams
) {
	// Load database.
	let connection = sqlite::open(
		Path::new(DATABASE_FILE)
	).unwrap();

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
	
	let mut state = BTreeMap::new();

	println!("quarkevm version {}", VERSION);

	let backend = MemoryBackend::new(&vicinity, state);
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
		let _reason = executor.transact_call(
			H160::from_str(&params.from).unwrap(),
			H160::from_str(&params.to).unwrap(),
			U256::zero(), // value
			hex::decode(&params.data)
				.unwrap(),
			100000000000u64, // gas limit
			Vec::new(),
		);
		println!("{:?}", _reason);
	}

	println!("Done!");
}




#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SendTransactionParams {
	from: String,
	to: String,
	gas: String,
	gasPrice: String,
	value: String,
	data: String
}

use std::{path::PathBuf};
use serde::{Deserialize, Serialize};
use serde_json::{Result, Number, Value};
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
	execute_in_vm(params);

	Ok(0)
}

fn main() {
    run().unwrap();
}




// use tide::Request;
// use tide::prelude::*;

// #[derive(Debug, Deserialize)]
// struct JSONRpcRequest {
//     id: String,
// 	jsonrpc: String,
// 	method: String,
// 	params: 
//     : u8,
// }

// #[async_std::main]
// async fn run_rpc_server() -> tide::Result<()> {
//     let mut app = tide::new();
//     app.at("/").post(handle_rpc_request);
//     app.listen("127.0.0.1:8569").await?;
//     Ok(())
// }

// async fn handle_rpc_request(mut req: Request<()>) -> tide::Result {
//     let Animal { name, legs } = req.body_json().await?;
//     Ok(format!("Hello, {}! I've put in an order for {} shoes", name, legs).into())
// }


// Persistent storage in sqlite.

// use jsonrpc_http_server::jsonrpc_core::{IoHandler, Value, Params};
// use jsonrpc_http_server::ServerBuilder;
// use serde::de::DeserializeOwned;
// use serde::{self, Serialize, Deserialize}; // 1.0.104


// use serde_json::{json, Error};
// use serde_json::map::Map;


// use jsonrpc_core::Result;
// use jsonrpc_derive::rpc;

// pub struct RpcImpl {

// }

// impl RpcImpl {
// 	fn eth_call(&self) -> Result<u64> {
// 		Ok(0)
// 	}

// 	fn eth_send_transaction(&self, params: SendTransactionParams) -> Result<String> {
// 		// let mut parsed = params.parse::<SendTransactionParams>().unwrap();
// 		let mut parsed = params.clone();
		
// 		parsed.from = params.from.strip_prefix("0x").unwrap().to_string();
// 		parsed.data = params.data.strip_prefix("0x").unwrap().to_string();

// 		// TODO validate parsed.from signature.

// 		// now execute the transaction.
// 		// let executor = executor_ref.lock().unwrap();

		// let _reason = executor.transact_call(
		// 	H160::from_str(&parsed.from).unwrap(),
		// 	H160::from_str(&parsed.to).unwrap(),
		// 	U256::zero(), // value
		// 	hex::decode(&parsed.data)
		// 		.unwrap(),
		// 	100000000000u64, // gas limit
		// 	Vec::new(),
		// );

// 		// println!("{:?}", _reason);

// 		Ok(hex::encode("hello").to_owned())
// 	}
// }


// fn run_rpc_server() {
// 	// let executor_ref = Arc::new(Mutex::new(executor));
	
// 	let mut rpc_server = &RpcImpl{};
// 	let mut io = jsonrpc_core::IoHandler::new();
	
// 	io.add_method("eth_call", |_params: Params| async {
// 		Ok(Value::String(hex::encode("hello").to_owned()))
// 	});
	
// 	io.add_method("eth_sendTransaction", |_params: Params| async {
// 		let mut parsed = _params.parse::<SendTransactionParams>().unwrap();
// 		let res = rpc_server.eth_send_transaction(parsed).unwrap();
// 		Ok(Value::String(res))
// 	});

// 	let server = ServerBuilder::new(io)
// 		.threads(1)
// 		.start_http(&"127.0.0.1:8549".parse().unwrap())
// 		.unwrap();

// 	server.wait();
// }