use anyhow::Result;
use bitcoin::{BitcoinConfig, BitcoinNode};
use lightning::{LightningConfig, LightningNode};

mod bitcoin;
mod lightning;
mod visualization;
use visualization::NetworkGraph;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Bitcoin configuration
    let bitcoin_config = BitcoinConfig {
        rpc_host: "127.0.0.1".to_string(),
        rpc_port: 18443,
        rpc_user: "bitcoinrpc".to_string(),
        rpc_password: "rpcpassword".to_string(),
        network: "regtest".to_string(),
        bitcoin_path: Some("/snap/bin/bitcoin-core.daemon".to_string()),
    };

    // Lightning configuration
    let lightning_config = LightningConfig {
        network: "regtest".to_string(),
        lightning_dir: "/home/kyllian/.lightning".to_string(),
        bitcoin_rpc_host: bitcoin_config.rpc_host.clone(),
        bitcoin_rpc_port: bitcoin_config.rpc_port,
        bitcoin_rpc_user: bitcoin_config.rpc_user.clone(),
        bitcoin_rpc_password: bitcoin_config.rpc_password.clone(),
    };

    // Create and connect to Bitcoin node
    let bitcoin_node = BitcoinNode::new(bitcoin_config.clone())?;
    println!("Bitcoin node created, getting blockchain info...");
    let blockchain_info = bitcoin_node.get_blockchain_info().await?;
    println!("Blockchain info: {}", blockchain_info);

    // Create and connect to Lightning node
    let mut lightning_node = LightningNode::new(lightning_config, "node1".to_string());
    println!("Connecting to Lightning node...");
    lightning_node.connect_rpc().await?;
    
    let node_info = lightning_node.get_node_info().await?;
    println!("Lightning node info: {:?}", node_info);

    // Create a test invoice
    let invoice = lightning_node.create_invoice(
        1000000, // 1000 sats
        &format!("test_invoice_{}", chrono::Utc::now().timestamp()),
        "Test payment"
    ).await?;
    println!("Created invoice: {:?}", invoice);

    // Créer un deuxième nœud Lightning avec une configuration différente
    let lightning_config2 = LightningConfig {
        network: "regtest".to_string(),
        lightning_dir: "/home/kyllian/.lightning2".to_string(),
        bitcoin_rpc_host: bitcoin_config.rpc_host.clone(),
        bitcoin_rpc_port: bitcoin_config.rpc_port,
        bitcoin_rpc_user: bitcoin_config.rpc_user.clone(),
        bitcoin_rpc_password: bitcoin_config.rpc_password.clone(),
    };

    let mut lightning_node2 = LightningNode::new(lightning_config2, "node2".to_string());
    println!("Connecting to second Lightning node...");
    lightning_node2.connect_rpc().await?;

    // Obtenir l'ID du deuxième nœud
    let node2_info = lightning_node2.get_node_info().await?;
    let node2_id = node2_info["result"]["id"].as_str()
        .ok_or_else(|| anyhow::anyhow!("Could not get node2 ID"))?;
    println!("Node 2 info: {:?}", node2_info);

    // Obtenir une adresse Lightning et des fonds avant d'ouvrir le canal
    println!("Demande d'une nouvelle adresse Lightning...");
    let lightning_addr = match lightning_node.get_new_address().await {
        Ok(addr) => addr,
        Err(e) => {
            println!("Erreur lors de l'obtention de l'adresse : {:?}", e);
            return Err(e);
        }
    };
    println!("Adresse Lightning obtenue : {}", lightning_addr);

    // Obtenir une adresse Bitcoin pour le minage
    let mining_addr = bitcoin_node.get_new_address().await?;
    println!("Adresse de minage : {}", mining_addr);

    // Envoyer des fonds au nœud Lightning
    let tx_id = bitcoin_node.send_to_address(&lightning_addr, 1.0).await?;
    println!("Transaction envoyée : {}", tx_id);

    // Générer des blocs pour confirmer
    let block_hashes = bitcoin_node.generate_to_address(6, &mining_addr).await?;
    println!("Blocs générés : {:?}", block_hashes);

    // Vérifier les fonds
    let funds = lightning_node.list_funds().await?;
    println!("Fonds Lightning : {:?}", funds);

    // Attendre un peu pour s'assurer que les fonds sont bien reçus
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Connecter les nœuds
    println!("Connecting nodes...");
    lightning_node.connect_peer(node2_id, "127.00.0.1", 9736).await?;

    // Vérifier les canaux existants
    println!("\nChecking existing channels...");
    let funds = lightning_node.list_funds().await?;
    if let Some(channels) = funds["result"]["channels"].as_array() {
        println!("Number of existing channels: {}", channels.len());
        println!("Active channels:");
        for channel in channels {
            if let (Some(state), Some(amount)) = (
                channel["state"].as_str(),
                channel["amount_msat"].as_str()
            ) {
                println!("- Channel state: {}, Amount: {}", state, amount);
            }
        }
    }

    // Ne pas essayer d'ouvrir de nouveaux canaux si nous en avons déjà
    println!("\nChannels are ready for payments!");

    // Attendre que tous les canaux soient prêts
    println!("Waiting for all channels to be active...");
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    // Afficher l'état final
    let final_node_info = lightning_node.get_node_info().await?;
    println!("\nFinal node state: {:?}", final_node_info);

    // Créer et mettre à jour le graphe
    println!("\nCreating network visualization...");
    let mut network = NetworkGraph::new();
    
    // Ajouter les informations du premier nœud
    network.update_from_node_info(&node_info, &funds)?;
    
    // Ajouter les informations du second nœud
    network.update_from_node_info(&node2_info, &funds)?;

    // Générer et sauvegarder le fichier DOT
    let dot_output = network.to_dot();
    println!("Generating DOT file...");
    std::fs::write("lightning_network.dot", dot_output)?;
    println!("DOT file saved. Current directory: {:?}", std::env::current_dir()?);

    // Vérifier que le fichier existe
    if std::path::Path::new("lightning_network.dot").exists() {
        println!("DOT file created successfully!");
    } else {
        println!("Warning: DOT file was not created!");
    }

    // Afficher la commande pour générer l'image
    println!("\nTo generate the visualization, run:");
    println!("dot -Tpng lightning_network.dot -o network.png");

    Ok(())
} 