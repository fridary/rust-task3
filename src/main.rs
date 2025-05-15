use serde::Deserialize;
use std::{collections::HashMap, fs::File, error::Error, sync::Arc};
use tonic::transport::Channel;
use futures::stream;
use tokio_stream::StreamExt;

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    signature::{read_keypair_file, Keypair, Signer},
    system_instruction,
    transaction::Transaction,
    pubkey::Pubkey,
    commitment_config::CommitmentConfig,
};

mod proto_out;

use proto_out::geyser::geyser_client::GeyserClient;
use proto_out::geyser::{SubscribeRequest, SubscribeRequestFilterBlocks};

#[derive(Debug, Deserialize)]
struct Config {
    geyser_url: String,
    rpc_url: String,
    sender_keypair_path: String,
    recipient_address: String,
    amount_sol: f64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config: Config = serde_yaml::from_reader(File::open("config.yaml")?)?;

    let rpc_client = Arc::new(RpcClient::new_with_commitment(
        config.rpc_url.clone(),
        CommitmentConfig::confirmed(),
    ));
    let sender = Arc::new(read_keypair_file(&config.sender_keypair_path)?);
    let recipient: Pubkey = config.recipient_address.parse()?;
    let lamports = (config.amount_sol * 1_000_000_000.0) as u64;

    let mut grpc_client = GeyserClient::connect(config.geyser_url.clone()).await?;

    let mut blocks = HashMap::new();
    blocks.insert("default".to_string(), SubscribeRequestFilterBlocks {
        include_entries: Some(true),
        include_transactions: Some(true),
        ..Default::default()
    });

    let request = SubscribeRequest {
        blocks,
        ..Default::default()
    };

    let request_stream = stream::once(async { request });
    let mut stream = grpc_client.subscribe(request_stream).await?.into_inner();

    println!("Подписка активна. Ожидание новых блоков...");

    while let Some(update) = stream.next().await {
        match update {
            Ok(_) => {
                println!("Обнаружен новый блок. Отправка SOL...");
                match send_sol(
                    rpc_client.clone(),
                    sender.clone(),
                    recipient,
                    lamports,
                ).await {
                    Ok(sig) => println!("Транзакция отправлена. Подпись: {}", sig),
                    Err(e) => eprintln!("Ошибка при отправке: {}", e),
                }
            }
            Err(e) => eprintln!("Ошибка при получении блока: {}", e),
        }
    }

    Ok(())
}

async fn send_sol(
    client: Arc<RpcClient>,
    from: Arc<Keypair>,
    to: Pubkey,
    lamports: u64,
) -> Result<String, Box<dyn Error>> {
    let blockhash = client.get_latest_blockhash().await?;
    let tx = Transaction::new_signed_with_payer(
        &[system_instruction::transfer(&from.pubkey(), &to, lamports)],
        Some(&from.pubkey()),
        &[&*from],
        blockhash,
    );
    let sig = client.send_and_confirm_transaction(&tx).await?;
    Ok(sig.to_string())
}
