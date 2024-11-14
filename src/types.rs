use ethers::{
    abi::Address,
    providers::{Http, Provider},
    types::Filter,
};
use serde::Serialize;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct MerkleTreeRoot {
    pub hash: [u8; 32],
}

#[derive(Serialize)]
pub struct Transaction {
    pub tx_hash: String,
    pub index: u64,
    pub logs: Vec<String>,
    pub from: String,
    pub to: String,
    pub block_hash: String,
    pub root: String,
    pub logs_bloom: String,
}

#[derive(Clone)]
pub struct SyncData {
    pub cur: u64,
    pub from: u64,
    pub n: u64,
    pub filters: Filter,
    pub gap: u64,
}

#[derive(Clone)]
pub struct EthereumClient {
    pub provider: Provider<Http>,
    pub chain_name: String,
    pub chain_id: u64,
    pub start_block: u64,
    pub addresses: Vec<Address>,
}

pub struct RootParam {
    pub number: u128,
    pub root: MerkleTreeRoot,
    pub tx_root: MerkleTreeRoot,
}
