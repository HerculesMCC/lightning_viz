use anyhow::Result;
use serde::{Deserialize, Serialize};
use cln_rpc::{
    ClnRpc,
    Response,
    Request,
    model::requests::{InvoiceRequest, GetinfoRequest, ConnectRequest, FundchannelRequest, NewaddrRequest, ListfundsRequest},
    primitives::{Amount, AmountOrAny, AmountOrAll, PublicKey},
};
use std::cell::UnsafeCell;
use std::str::FromStr;
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightningConfig {
    pub network: String,
    pub lightning_dir: String,
    pub bitcoin_rpc_host: String,
    pub bitcoin_rpc_port: u16,
    pub bitcoin_rpc_user: String,
    pub bitcoin_rpc_password: String,
}

pub struct LightningNode {
    pub id: String,
    rpc_client: Option<UnsafeCell<ClnRpc>>,
    config: LightningConfig,
}

impl LightningNode {
    pub fn new(config: LightningConfig, id: String) -> Self {
        Self {
            id,
            rpc_client: None,
            config,
        }
    }

    pub async fn connect_rpc(&mut self) -> Result<()> {
        let socket_path = format!("{}/regtest/lightning-rpc", self.config.lightning_dir);
        self.rpc_client = Some(UnsafeCell::new(ClnRpc::new(socket_path).await?));
        Ok(())
    }

    pub async fn get_node_info(&self) -> Result<Value> {
        let client = self.rpc_client.as_ref().unwrap();
        unsafe {
            let response = (*client.get()).call(Request::Getinfo(GetinfoRequest {})).await?;
            Ok(serde_json::to_value(&response)?)
        }
    }

    pub async fn create_invoice(&self, amount_msat: u64, label: &str, description: &str) -> Result<Response> {
        let client = self.rpc_client.as_ref().unwrap();
        unsafe {
            Ok((*client.get()).call(Request::Invoice(InvoiceRequest {
                amount_msat: AmountOrAny::Amount(Amount::from_msat(amount_msat)),
                label: label.to_string(),
                description: description.to_string(),
                expiry: None,
                fallbacks: None,
                preimage: None,
                cltv: None,
                deschashonly: None,
                exposeprivatechannels: None,
            })).await?)
        }
    }

    pub async fn open_channel(&self, peer_id: &str, amount_sat: u64) -> Result<Response> {
        let client = self.rpc_client.as_ref().unwrap();
        let pubkey = PublicKey::from_str(peer_id)?;
        
        unsafe {
            Ok((*client.get()).call(Request::FundChannel(FundchannelRequest {
                id: pubkey,
                amount: AmountOrAll::Amount(Amount::from_sat(amount_sat)),
                push_msat: None,
                feerate: None,
                announce: None,
                minconf: None,
                close_to: None,
                request_amt: None,
                compact_lease: None,
                utxos: None,
                mindepth: None,
                reserve: None,
                channel_type: None,
            })).await?)
        }
    }

    pub async fn connect_peer(&self, node_id: &str, host: &str, port: u16) -> Result<Response> {
        let client = self.rpc_client.as_ref().unwrap();
        let pubkey = PublicKey::from_str(node_id)?;
        unsafe {
            Ok((*client.get()).call(Request::Connect(ConnectRequest {
                id: pubkey.to_string(),
                host: Some(host.to_string()),
                port: Some(port),
            })).await?)
        }
    }

    pub async fn get_new_address(&self) -> Result<String> {
        let client = self.rpc_client.as_ref().unwrap();
        unsafe {
            let response = (*client.get()).call(Request::NewAddr(NewaddrRequest {
                addresstype: None
            })).await?;
            
            println!("NewAddr response: {:?}", response);
            
            match response {
                Response::NewAddr(addr_response) => {
                    addr_response.bech32
                        .ok_or_else(|| anyhow::anyhow!("No bech32 address in response"))
                },
                _ => Err(anyhow::anyhow!("Unexpected response type"))
            }
        }
    }

    pub async fn list_funds(&self) -> Result<Value> {
        let client = self.rpc_client.as_ref().unwrap();
        unsafe {
            let response = (*client.get()).call(Request::ListFunds(ListfundsRequest {
                spent: None
            })).await?;
            Ok(serde_json::to_value(&response)?)
        }
    }
}

// Implémentation de Send et Sync pour LightningNode
unsafe impl Send for LightningNode {}
unsafe impl Sync for LightningNode {} 