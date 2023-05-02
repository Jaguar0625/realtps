use crate::client::Client;
use anyhow::Result;
use async_trait::async_trait;
use realtps_common::{chain::Chain, db::Block};

pub struct SymbolClient {
    client: reqwest::Client,
    url: String,
}

impl SymbolClient {
    pub fn new(url: &str) -> Result<Self> {
        Ok(Self {
            client: reqwest::Client::new(),
            url: url.to_string(),
        })
    }
}

#[allow(dead_code)]
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct SymbolNodeInfoResponse {
    version: u32,
    public_key: String,
    network_generation_hash_seed: String,
    roles: u32,
    port: u16,
    network_identifier: u8,
    host: String,
    friendly_name: String,
    node_public_key: String
}

#[allow(dead_code)]
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct SymbolChainInfoFinalizedBlock {
    finalization_epoch: u64,
    finalization_point: u64,
    height: String,
    hash: String
}

#[allow(dead_code)]
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct SymbolChainInfoResponse {
    height: String,
    score_high: String,
    score_low: String,
    latest_finalized_block: SymbolChainInfoFinalizedBlock
}

#[allow(dead_code)]
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct SymbolBlockHeaderMeta {
    hash: String,
    generation_hash: String,
    total_fee: String,
    total_transactions_count: u64,
    state_hash_sub_cache_merkle_roots: Vec<String>,
    transactions_count: u64,
    statements_count: u64
}


#[allow(dead_code)]
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct SymbolBlockHeaderData {
    size: u64,
    signature: String,
    signer_public_key: String,
    version: u64,
    network: u8,
    #[serde(rename = "type")]
    type_: u16,
    height: String,
    timestamp: String,
    previous_block_hash: String,
    beneficiary_address: String,
    fee_multiplier: u64
}

#[derive(serde::Deserialize)]
struct SymbolBlockHeaderResponse {
    meta: SymbolBlockHeaderMeta,
    block: SymbolBlockHeaderData
}

#[async_trait]
impl Client for SymbolClient {
    async fn client_version(&self) -> Result<String> {
        let url = format!("{}/node/info", &self.url);
        let resp = self.client.get(&url).send().await?;
        let node_info: SymbolNodeInfoResponse = resp.json().await?;
        Ok(format!("{:x}", node_info.version))
    }
    async fn get_latest_block_number(&self) -> Result<u64> {
        let url = format!("{}/chain/info", &self.url);
        let resp = self.client.get(&url).send().await?;
        let chain_info: SymbolChainInfoResponse = resp.json().await?;
        Ok(chain_info.height.parse().unwrap())
    }
    async fn get_block(&self, block_number: u64) -> Result<Option<Block>> {
        let url = format!("{}/blocks/{}", &self.url, block_number);
        let resp = self.client.get(url).send().await?;
        let block_header: SymbolBlockHeaderResponse = resp.json().await?;

        let timestamp = block_header.block.timestamp.parse::<u64>().unwrap() / 1000;

        Ok(Some(Block {
            chain: Chain::Symbol,
            block_number,
            prev_block_number: if block_number > 0 {
                Some(block_number - 1)
            } else {
                None
            },
            timestamp,
            num_txs: block_header.meta.total_transactions_count,
            hash: block_header.meta.hash,
            parent_hash: block_header.block.previous_block_hash,
        }))
    }
}

#[cfg(test)]
mod test_symbol {
    use super::{Client, SymbolClient};

    const RPC_URL: &str = "http://07.symbol-node.net:3000";

    #[tokio::test]
    async fn client_version() -> Result<(), anyhow::Error> {
        let client = SymbolClient::new(RPC_URL)?;
        let ver = client.client_version().await?;
        println!("client_version: {}", ver);
        assert!(!ver.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn get_latest_block_number() -> Result<(), anyhow::Error> {
        let client = SymbolClient::new(RPC_URL)?;
        let latest_block_number = client.get_latest_block_number().await?;
        println!("latest_block_number: {}", latest_block_number);
        assert!(latest_block_number > 0);
        Ok(())
    }

    #[tokio::test]
    async fn get_block() -> Result<(), anyhow::Error> {
        let client = SymbolClient::new(RPC_URL)?;
        let latest_block_number = client.get_latest_block_number().await?;
        println!("latest_block_number: {}", latest_block_number);
        let block = client.get_block(latest_block_number).await?;
        println!("block: {:?}", block);
        Ok(())
    }
}
