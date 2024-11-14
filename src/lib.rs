use std::thread::sleep;
use std::time::Duration;

use ethers::prelude::*;
use ethers::providers::{Http, Provider};
use ethers::types::{Block, Log, Transaction as EtherTransaction, TxHash};
use serde::Serialize;
use types::{EthereumClient, SyncData, Transaction};

pub mod merkle;
pub mod types;
pub use ethers::*;

impl EthereumClient {
    pub async fn new(
        rpc: &str,
        chain_name: &str,
        chain_id: u64,
        start_block: u64,
        addresses: Vec<Address>,
    ) -> Self {
        let provider = Provider::<Http>::try_from(rpc).expect("Invalid provider");

        Self {
            chain_name: chain_name.to_owned(),
            chain_id,
            provider,
            start_block,
            addresses,
        }
    }

    pub async fn new_sync(&self, from: u64, events: &[&str]) -> anyhow::Result<SyncData> {
        let cur = self.provider.get_block_number().await?.as_u64() - 3;

        Ok(SyncData {
            cur,
            from,
            filters: Filter::new()
                .address(self.addresses.clone())
                .events(events)
                .from_block(from)
                .to_block(cur),
            n: 50000,
            gap: 3,
        })
    }

    pub async fn fetch_event(
        &mut self,
        sync_data: &mut SyncData,
    ) -> anyhow::Result<(Vec<Log>, u64)> {
        let gap = sync_data.cur - sync_data.from;
        let limit = if gap > sync_data.n {
            sync_data.from + sync_data.n - 1
        } else {
            sleep(Duration::from_secs(10));
            sync_data.cur
        };

        sync_data.filters = sync_data
            .filters
            .clone()
            .from_block(sync_data.from)
            .to_block(U64([limit]));

        let mut number = limit + 1;
        if gap > sync_data.n {
            sync_data.from = limit + 1;
        } else {
            sync_data.from = limit;
            number = self.provider.get_block_number().await?.as_u64() - sync_data.gap;
            sync_data.cur = number;
        }
        let logs = self.provider.get_logs(&sync_data.filters).await?;
        Ok((logs, number))
    }

    pub async fn get_block_count(&self) -> anyhow::Result<u64> {
        Ok(self.provider.get_block_number().await?.as_u64())
    }

    pub async fn get_block(&self, block_number: u64) -> anyhow::Result<Option<Block<TxHash>>> {
        Ok(self.provider.get_block(block_number).await?)
    }

    // 获取区块中的交易列表
    pub async fn get_block_transactions(
        &self,
        block_number: u64,
    ) -> anyhow::Result<Vec<EtherTransaction>> {
        let block = self.get_block(block_number).await?;
        if let Some(block) = block {
            let mut transactions = Vec::new();
            for tx_hash in block.transactions {
                if let Some(tx) = self.get_transaction(tx_hash).await? {
                    transactions.push(tx);
                }
            }
            Ok(transactions)
        } else {
            Ok(vec![])
        }
    }

    pub async fn get_transaction(
        &self,
        tx_hash: TxHash,
    ) -> anyhow::Result<Option<EtherTransaction>> {
        Ok(self.provider.get_transaction(tx_hash).await?)
    }

    pub async fn get_transaction_receipt(
        &self,
        tx_hash: TxHash,
    ) -> anyhow::Result<Option<TransactionReceipt>> {
        Ok(self.provider.get_transaction_receipt(tx_hash).await?)
    }

    pub async fn get_logs(&self, start_block: u64, end_block: u64) -> anyhow::Result<Vec<Log>> {
        Ok(self
            .provider
            .get_logs(&Filter::new().from_block(start_block).to_block(end_block))
            .await?)
    }

    fn data_slice<T>(datas: &Vec<T>) -> Vec<Vec<u8>>
    where
        T: Serialize,
    {
        datas
            .iter()
            .map(|f| string_to_crypto_hash(&serde_json::to_string(f).unwrap()).to_vec())
            .collect()
    }

    pub async fn get_transaction_merkle(&self, block: &Block<H256>) -> anyhow::Result<MerkleTree> {
        let mut txs = Vec::new();
        for x in block.transactions.clone() {
            if let Some(receipt) = self.get_transaction_receipt(x).await? {
                txs.push(serde_json::to_vec(&Transaction {
                    tx_hash: serde_json::to_string(&receipt.transaction_hash).unwrap(),
                    index: receipt.transaction_index.as_u64(),
                    logs: receipt
                        .logs
                        .iter()
                        .map(|f| serde_json::to_string(f).unwrap())
                        .collect(),
                    from: format!("{:?}", receipt.from),
                    to: format!("{:?}", receipt.to),
                    block_hash: format!("{:?}", receipt.block_hash),
                    root: receipt.root.unwrap_or_default().to_string(),
                    logs_bloom: receipt.logs_bloom.to_string(),
                })?);
            }
        }
        Ok(MerkleTree::build(&txs))
    }

    pub async fn get_root_merkle(
        &self,
        block: &Block<H256>,
        index: Option<u64>,
    ) -> anyhow::Result<(MerkleTreeRoot, MerkleTreeProof, Vec<u8>)> {
        let mut items = Vec::new();
        let mut i = 0;
        let mut count = 0;
        for tx_hash in block.transactions.clone() {
            if let Some(receipt) = self.get_transaction_receipt(tx_hash).await? {
                items.push(serde_json::to_vec(&Transaction {
                    tx_hash: serde_json::to_string(&receipt.transaction_hash).unwrap(),
                    index: receipt.transaction_index.as_u64(),
                    logs: receipt
                        .logs
                        .iter()
                        .map(|f| serde_json::to_string(f).unwrap())
                        .collect(),
                    from: format!("{:?}", receipt.from),
                    to: format!("{:?}", receipt.to),
                    block_hash: format!("{:?}", receipt.block_hash),
                    root: receipt.root.unwrap_or_default().to_string(),
                    logs_bloom: receipt.logs_bloom.to_string(),
                })?);
                if let Some(c) = index {
                    if receipt.transaction_index.as_u64() == c {
                        i = count;
                    }
                }
                count += 1;
            }
        }
        let merkle = MerkleTree::build(&items);

        Ok((merkle.root, merkle.proofs[i].clone(), items[i].clone()))
    }

    pub fn get_hash_merkle(
        block: &Block<H256>,
        transaction_hash: Option<H256>,
    ) -> (MerkleTreeRoot, MerkleTreeProof) {
        let mut tx_hashs = block.transactions.clone();
        tx_hashs.push(block.transactions_root);

        let index = if let Some(hx) = transaction_hash {
            tx_hashs
                .iter()
                .position(|h| h == &hx)
                .expect("Transaction hash not found")
        } else {
            0
        };
        let hash_items = Self::data_slice(&tx_hashs);
        let merkle = MerkleTree::build(&hash_items);
        (merkle.root, merkle.proofs[index].clone())
    }
}
