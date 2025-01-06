use anyhow::Result;
use bitcoincore_rpc::{Auth, Client, RpcApi};
use bitcoincore_rpc::bitcoin::{Address, Amount as BitcoinAmount, Network};
use serde::{Deserialize, Serialize};
use std::process::{Command, Child};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BitcoinConfig {
    pub rpc_host: String,
    pub rpc_port: u16,
    pub rpc_user: String,
    pub rpc_password: String,
    pub network: String,
    pub bitcoin_path: Option<String>,
}

pub struct BitcoinNode {
    client: Client,
    process: Option<Child>,
}

impl BitcoinNode {
    pub fn new(config: BitcoinConfig) -> Result<Self> {
        let rpc_url = format!("http://{}:{}", config.rpc_host, config.rpc_port);
        let auth = Auth::UserPass(config.rpc_user, config.rpc_password);
        let client = Client::new(&rpc_url, auth)?;
        
        Ok(Self { 
            client,
            process: None 
        })
    }

    pub async fn get_blockchain_info(&self) -> Result<String> {
        let info = self.client.get_blockchain_info()?;
        Ok(serde_json::to_string_pretty(&info)?)
    }

    pub async fn generate_blocks(&self, count: u64) -> Result<Vec<String>> {
        // Vérifier que le portefeuille est chargé
        self.ensure_wallet().await?;
        
        let address = self.client.get_new_address(None, None)?
            .require_network(Network::Regtest)?;
        
        println!("Generating {} blocks to address: {}", count, address);
        
        let block_hashes = self.client.generate_to_address(count, &address)?;
        
        println!("Generated blocks successfully!");
        
        Ok(block_hashes.iter().map(|h| h.to_string()).collect())
    }

    pub async fn ensure_wallet(&self) -> Result<()> {
        for _ in 0..5 {  // Essayer 5 fois
            match self.client.create_wallet("default", None, None, None, None) {
                Ok(_) => {
                    println!("Created new wallet 'default'");
                }
                Err(e) => {
                    if !e.to_string().contains("Database already exists") {
                        println!("Warning: {}", e);
                    }
                    println!("Wallet 'default' already exists");
                }
            }

            // Essayer de charger le portefeuille
            match self.client.load_wallet("default") {
                Ok(_) => {
                    println!("Loaded wallet 'default'");
                    return Ok(());
                }
                Err(e) => {
                    if e.to_string().contains("already loaded") {
                        println!("Wallet was already loaded");
                        return Ok(());
                    }
                    println!("Waiting for Bitcoin Core to be ready...");
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                }
            }
        }
        Err(anyhow::anyhow!("Failed to setup wallet after multiple attempts"))
    }

    pub async fn start_daemon(&mut self, config: &BitcoinConfig) -> Result<()> {
        match self.client.get_blockchain_info() {
            Ok(_) => {
                println!("Bitcoin Core est déjà en cours d'exécution");
            }
            Err(_) => {
                let bitcoin_path = if let Some(path) = &config.bitcoin_path {
                    PathBuf::from(path)
                } else {
                    PathBuf::from(r"C:\Program Files\Bitcoin\daemon\bitcoind.exe")
                };

                if !bitcoin_path.exists() {
                    return Err(anyhow::anyhow!("Bitcoin executable not found at: {:?}", bitcoin_path));
                }

                println!("Starting Bitcoin Core from: {:?}", bitcoin_path);
                
                let process = Command::new(bitcoin_path)
                    .arg("-regtest")
                    .spawn()?;

                self.process = Some(process);
                
                // Attendre plus longtemps pour l'initialisation
                println!("Waiting for Bitcoin Core to initialize...");
                for _ in 0..30 {  // Attendre jusqu'à 30 secondes
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    if self.client.get_blockchain_info().is_ok() {
                        println!("Bitcoin Core is ready!");
                        break;
                    }
                }
            }
        }
        
        // Attendre encore un peu avant de créer le portefeuille
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        match self.ensure_wallet().await {
            Ok(_) => println!("Wallet setup completed"),
            Err(e) => println!("Warning: Wallet setup failed: {}", e),
        }
        
        Ok(())
    }

    pub async fn stop_daemon(&mut self) -> Result<()> {
        if let Some(mut process) = self.process.take() {
            process.kill()?;
            process.wait()?;
        }
        Ok(())
    }

    pub async fn send_to_address(&self, address: &str, amount: f64) -> Result<String> {
        // Convertir l'adresse string en Address Bitcoin
        let bitcoin_address = Address::from_str(address)?.require_network(Network::Regtest)?;
        // Convertir le montant f64 en Amount Bitcoin (en BTC)
        let bitcoin_amount = BitcoinAmount::from_btc(amount)?;
        
        let txid = self.client.send_to_address(
            &bitcoin_address,
            bitcoin_amount,
            None,
            None,
            None,
            None,
            None,
            None
        )?;
        
        Ok(txid.to_string())
    }

    pub async fn generate_to_address(&self, blocks: u64, address: &str) -> Result<Vec<String>> {
        // Convertir l'adresse string en Address Bitcoin
        let bitcoin_address = Address::from_str(address)?.require_network(Network::Regtest)?;
        
        let block_hashes = self.client.generate_to_address(blocks, &bitcoin_address)?;
        
        // Convertir les BlockHash en String
        Ok(block_hashes.iter().map(|h| h.to_string()).collect())
    }

    pub async fn get_new_address(&self) -> Result<String> {
        let address = self.client.get_new_address(None, None)?
            .require_network(Network::Regtest)?;
        Ok(address.to_string())
    }
}

impl Drop for BitcoinNode {
    fn drop(&mut self) {
        if let Some(mut process) = self.process.take() {
            let _ = process.kill();
            let _ = process.wait();
        }
    }
} 