use super::{Apply, ApplyBackend, Backend, Basic, Log};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use primitive_types::{H160, H256, U256};
use sqlite::{Connection};
use sqlite::State;


/// Vivinity value of a memory backend.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
	feature = "with-codec",
	derive(codec::Encode, codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MemoryVicinity {
	/// Gas price.
	pub gas_price: U256,
	/// Origin.
	pub origin: H160,
	/// Chain ID.
	pub chain_id: U256,
	/// Environmental block hashes.
	pub block_hashes: Vec<H256>,
	/// Environmental block number.
	pub block_number: U256,
	/// Environmental coinbase.
	pub block_coinbase: H160,
	/// Environmental block timestamp.
	pub block_timestamp: U256,
	/// Environmental block difficulty.
	pub block_difficulty: U256,
	/// Environmental block gas limit.
	pub block_gas_limit: U256,
	/// Environmental base fee per gas.
	pub block_base_fee_per_gas: U256,
}

/// Account information of a memory backend.
#[derive(Default, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
	feature = "with-codec",
	derive(codec::Encode, codec::Decode, scale_info::TypeInfo)
)]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MemoryAccount {
	/// Account nonce.
	pub nonce: U256,
	/// Account balance.
	pub balance: U256,
	/// Full account storage.
	pub storage: BTreeMap<H256, H256>,
	/// Account code.
	pub code: Vec<u8>,
}


/// Memory backend, storing all state values in a `BTreeMap` in memory.

pub struct MemoryBackend<'vicinity> {
	// db: Database<[u8]>,
	db: Connection,
	vicinity: &'vicinity MemoryVicinity,
	state: BTreeMap<H160, MemoryAccount>,
	logs: Vec<Log>,
}

use core::fmt::{Debug, Formatter};

impl Clone for MemoryBackend<'_> {
	fn clone(&self) -> Self { todo!() }
}

impl Debug for MemoryBackend<'_> {
	fn fmt(&self, _: &mut Formatter<'_>) -> Result<(), std::fmt::Error> { todo!() }
}

impl<'vicinity> MemoryBackend<'vicinity> {
	/// Create a new memory backend.
	pub fn new(vicinity: &'vicinity MemoryVicinity, state: BTreeMap<H160, MemoryAccount>, db_path: String, genesis: bool) -> Self {
		let connection = sqlite::open(db_path).unwrap();

		if genesis {
			connection.execute(
				"
				CREATE TABLE code (
					time INTEGER PRIMARY KEY AUTOINCREMENT, 
					address BLOB, 
					value BLOB
				);
				CREATE TABLE storage (
					time INTEGER PRIMARY KEY AUTOINCREMENT, 
					address BLOB, 
					idx BLOB, 
					value BLOB
				);
				CREATE TABLE accounts (
					time INTEGER PRIMARY KEY AUTOINCREMENT, 
					address BLOB, 
					balance BLOB, 
					nonce BLOB
				);
				"
			).unwrap();
		}

		Self {
			db: connection,
			vicinity,
			state,
			logs: Vec::new(),
		}
	}

	/// Get the underlying `BTreeMap` storing the state.
	pub fn state(&self) -> &BTreeMap<H160, MemoryAccount> {
		&self.state
	}

	/// Get a mutable reference to the underlying `BTreeMap` storing the state.
	pub fn state_mut(&mut self) -> &mut BTreeMap<H160, MemoryAccount> {
		&mut self.state
	}
}

impl<'vicinity> Backend for MemoryBackend<'vicinity> {
	fn gas_price(&self) -> U256 {
		self.vicinity.gas_price
	}
	fn origin(&self) -> H160 {
		self.vicinity.origin
	}
	fn block_hash(&self, number: U256) -> H256 {
		if number >= self.vicinity.block_number
			|| self.vicinity.block_number - number - U256::one()
				>= U256::from(self.vicinity.block_hashes.len())
		{
			H256::default()
		} else {
			let index = (self.vicinity.block_number - number - U256::one()).as_usize();
			self.vicinity.block_hashes[index]
		}
	}
	fn block_number(&self) -> U256 {
		self.vicinity.block_number
	}
	fn block_coinbase(&self) -> H160 {
		self.vicinity.block_coinbase
	}
	fn block_timestamp(&self) -> U256 {
		self.vicinity.block_timestamp
	}
	fn block_difficulty(&self) -> U256 {
		self.vicinity.block_difficulty
	}
	fn block_gas_limit(&self) -> U256 {
		self.vicinity.block_gas_limit
	}
	fn block_base_fee_per_gas(&self) -> U256 {
		self.vicinity.block_base_fee_per_gas
	}

	fn chain_id(&self) -> U256 {
		self.vicinity.chain_id
	}

	fn exists(&self, address: H160) -> bool {
		self.state.contains_key(&address)
	}

	fn basic(&self, address: H160) -> Basic {
		let address_hex = hex::encode(address);
		
		let mut statement = self.db
			.prepare(format!("
				SELECT * FROM accounts 
				WHERE address = X'{address_hex}' 
				ORDER BY time DESC LIMIT 1
			"))
			.unwrap();
		
		if let State::Row = statement.next().unwrap() {
			let balance = statement.read::<Vec<u8>>(2).unwrap();
			let nonce = statement.read::<Vec<u8>>(3).unwrap();
			return Basic{
				balance: U256::from_big_endian(&balance),
				nonce: U256::from_big_endian(&nonce),
			}
		} else {
			return Basic{
				balance: U256::zero(),
				nonce: U256::zero()
			}
		}

		// self.state
		// 	.get(&address)
		// 	.map(|a| Basic {
		// 		balance: a.balance,
		// 		nonce: a.nonce,
		// 	})
		// 	.unwrap_or_default()
	}

	fn code(&self, address: H160) -> Vec<u8> {
		let address_hex = hex::encode(address);
		let mut statement = self.db
			.prepare(format!("
				SELECT * FROM code 
				WHERE address = X'{address_hex}'
				ORDER BY time DESC LIMIT 1
			"))
			.unwrap();
		
		if let State::Row = statement.next().unwrap() {
			let code = statement.read::<Vec<u8>>(2).unwrap();
			return code
		} else {
			return vec![]
		}

		// self.state
		// 	.get(&address)
		// 	.map(|v| v.code.clone())
		// 	.unwrap_or_default()
	}

	fn storage(&self, address: H160, index: H256) -> H256 {
		let address_hex = hex::encode(address);
		let index_hex = hex::encode(index);
		let mut statement = self.db
			.prepare(format!("
				SELECT * FROM storage 
				WHERE 
					address = X'{address_hex}' 
					AND idx = X'{index_hex}'
				ORDER BY time DESC LIMIT 1
			"))
			.unwrap();
		
		if let State::Row = statement.next().unwrap() {
			let value = statement.read::<Vec<u8>>(3).unwrap();
			return H256::from_slice(&value)
		} else {
			return H256::zero();
		}

		// self.state
		// 	.get(&address)
		// 	.map(|v| v.storage.get(&index).cloned().unwrap_or_default())
		// 	.unwrap_or_default()
	}

	fn original_storage(&self, address: H160, index: H256) -> Option<H256> {
		Some(self.storage(address, index))
	}
}

impl<'vicinity> ApplyBackend for MemoryBackend<'vicinity> {
	fn apply<A, I, L>(&mut self, values: A, logs: L, delete_empty: bool)
	where
		A: IntoIterator<Item = Apply<I>>,
		I: IntoIterator<Item = (H256, H256)>,
		L: IntoIterator<Item = Log>,
	{
		for apply in values {
			match apply {
				Apply::Modify {
					address,
					basic,
					code,
					storage,
					reset_storage: _,
				} => {
					let is_empty = {

						let address_hex = hex::encode(address);

						// 1. Borrow code
						// 2. Unwrap Option<Vec<u8>> into Vec<u8>
						// Usually we borrow using the & operator
						// However we don't want to borrow the option, do we? 
						if let Some(code) = &code {
							let code_hex = hex::encode(code);
						
							// 1. Code.
							self.db.execute(format!("
								INSERT INTO code VALUES (NULL, X'{address_hex}', X'{code_hex}')
							")).unwrap();
						} else {
							// None means leaving it unchanged. 
						} 


						let account = self.state.entry(address).or_insert_with(Default::default);
						account.balance = basic.balance;
						account.nonce = basic.nonce;
						if let Some(code) = code {
							account.code = code;
						}
						// if reset_storage {
						// 	account.storage = BTreeMap::new();
						// }

						// let zeros = account
						// 	.storage
						// 	.iter()
						// 	.filter(|(_, v)| v == &&H256::default())
						// 	.map(|(k, _)| *k)
						// 	.collect::<Vec<H256>>();

						// for zero in zeros {
						// 	account.storage.remove(&zero);
						// }

						// 2. Account
						let mut balance_buf: [u8; 32] = Default::default();
						basic.balance.to_big_endian(&mut balance_buf);

						let mut nonce_buf: [u8; 32] = Default::default();
						basic.nonce.to_big_endian(&mut nonce_buf);

						let balance_hex = hex::encode(balance_buf);
						let nonce_hex = hex::encode(nonce_buf);
						self.db.execute(format!("
							INSERT INTO accounts VALUES (NULL, X'{address_hex}', X'{balance_hex}', X'{nonce_hex}')
						")).unwrap();


						// 3. Storage
						for (index, value) in storage {
							let index_hex = hex::encode(&index);
							let value_hex = hex::encode(&value);

							self.db.execute(format!("
								INSERT INTO storage VALUES (NULL, X'{address_hex}', X'{index_hex}', X'{value_hex}')
							")).unwrap();

							// if value == H256::default() {
							// 	account.storage.remove(&index);
							// } else {
							// 	account.storage.insert(index, value);
							// }
						}

						account.balance == U256::zero()
							&& account.nonce == U256::zero()
							&& account.code.is_empty()
					};

					if is_empty && delete_empty {
						self.state.remove(&address);
					}
				}
				Apply::Delete { address } => {
					// self.db.execute(format!("
					// 	INSERT INTO storage VALUES ('{address_hex}', '{index_hex}', '{value_hex}')
					// ")).unwrap();

					let address_hex = hex::encode(address);
					
					self.db.execute(format!("
						INSERT INTO code VALUES (X'{address_hex}', NULL)
					")).unwrap();

					self.db.execute(format!("
						INSERT INTO accounts VALUES (X'{address_hex}', NULL, NULL)
					")).unwrap();

					// self.state.remove(&address);
				}
			}
		}

		for log in logs {
			self.logs.push(log);
		}
	}
}
