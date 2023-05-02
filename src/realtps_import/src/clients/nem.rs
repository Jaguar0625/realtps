use crate::client::Client;
use anyhow::Result;
use async_trait::async_trait;
use realtps_common::{chain::Chain, db::Block};
use std::collections::HashMap;

pub struct NemClient {
    client: reqwest::Client,
    url: String,
}

#[allow(dead_code)]
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct NemNodeInfoMetadataResponse {
    features: u32,
    application: Option<String>,
    network_id: u32,
    version: String,
    platform: String
}

#[allow(dead_code)]
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct NemNodeInfoResponse {
    meta_data: NemNodeInfoMetadataResponse
    // endpoint
    // identity
}

#[derive(serde::Deserialize)]
struct NemChainInfoResponse {
    height: u64
}

impl NemClient {
    pub fn new(url: &str) -> Result<Self> {
        Ok(Self {
            client: reqwest::Client::new(),
            url: url.to_string(),
        })
    }

    async fn _get_block(&self, block_number: u64) -> Result<serde_json::Value> {
        let mut height_request = HashMap::new();
        height_request.insert("height", block_number);

        let url = format!("{}/block/at/public", &self.url);
        let resp = self.client.post(url).json(&height_request).send().await?;
        Ok(resp.json().await?)
    }
}

#[async_trait]
impl Client for NemClient {
    async fn client_version(&self) -> Result<String> {
        let url = format!("{}/node/info", &self.url);
        let resp = self.client.get(&url).send().await?;
        let node_info: NemNodeInfoResponse = resp.json().await?;
        Ok(node_info.meta_data.version)
    }
    async fn get_latest_block_number(&self) -> Result<u64> {
        let url = format!("{}/chain/height", &self.url);
        let resp = self.client.get(&url).send().await?;
        let chain_info: NemChainInfoResponse = resp.json().await?;

        // return the block prior to the last block because the api doesn't return the hash of the last block
        Ok(if chain_info.height > 1 { chain_info.height - 1 } else { 0 })
    }
    async fn get_block(&self, block_number: u64) -> Result<Option<Block>> {
        let block_json = self._get_block(block_number).await?;
        let next_block_json = self._get_block(block_number + 1).await?;

        Ok(Some(Block {
            chain: Chain::Nem,
            block_number,
            prev_block_number: if block_number > 0 {
                Some(block_number - 1)
            } else {
                None
            },
            timestamp: block_json["timeStamp"].as_u64().unwrap(),
            num_txs: block_json["transactions"].as_array().unwrap().len() as u64,
            hash: next_block_json["prevBlockHash"]["data"].to_string().to_uppercase(),
            parent_hash: block_json["prevBlockHash"]["data"].to_string().to_uppercase(),
        }))
    }
}

#[cfg(test)]
mod test_nem {
    use super::{Client, NemClient};

    const RPC_URL: &str = "http://san.nem.ninja:7890";

    #[tokio::test]
    async fn client_version() -> Result<(), anyhow::Error> {
        let client = NemClient::new(RPC_URL)?;
        let ver = client.client_version().await?;
        println!("client_version: {}", ver);
        assert!(!ver.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn get_latest_block_number() -> Result<(), anyhow::Error> {
        let client = NemClient::new(RPC_URL)?;
        let latest_block_number = client.get_latest_block_number().await?;
        println!("latest_block_number: {}", latest_block_number);
        assert!(latest_block_number > 0);
        Ok(())
    }

    #[tokio::test]
    async fn get_block() -> Result<(), anyhow::Error> {
        let client = NemClient::new(RPC_URL)?;
        let latest_block_number = client.get_latest_block_number().await?;
        println!("latest_block_number: {}", latest_block_number);
        let block = (client.get_block(latest_block_number).await?).unwrap();
        println!("block: {:?}", block);

        assert_eq!(block.chain, realtps_common::chain::Chain::Nem);
        assert_eq!(block.block_number, latest_block_number);
        Ok(())
    }
}
