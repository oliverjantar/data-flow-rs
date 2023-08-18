use colored::*;
use dotenv::dotenv;
use ethers::core::types::Block;
use ethers::providers::{Http, Middleware, Provider, Ws};
use ethers::types::{Transaction, U64};
use futures::StreamExt;
use std::error::Error;
use tokio::sync::broadcast;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + 'static>> {
    dotenv().ok();

    let mut from_block;

    {
        let provider =
            Provider::<Http>::try_from(RPC_URL).expect("Couldn't instantiate http provider");

        from_block = provider
            .get_block_number()
            .await
            .unwrap_or_else(|err| panic!("Couldn't get latest block. Error: {err}"))
            .as_u64();
    }

    from_block -= 10;

    switch_http_to_ws(from_block).await?;

    Ok(())
}

const BUFFER_SIZE: usize = 10;
const RPC_URL_WS: &str = "";
const RPC_URL: &str = "";

async fn switch_http_to_ws(from_block: u64) -> Result<(), Box<dyn Error>> {
    let provider = Provider::<Http>::try_from(RPC_URL).expect("Couldn't instantiate http provider");

    //block download and processing
    let (sender, mut receiver) = tokio::sync::broadcast::channel::<Block<Transaction>>(BUFFER_SIZE);

    //channel for ws
    let (sender_new_block, receiver_new_block) = tokio::sync::broadcast::channel(BUFFER_SIZE);

    let latest_block = provider
        .get_block_number()
        .await
        .unwrap_or_else(|err| panic!("Couldn't get latest block. Error: {err}"))
        .as_u64();

    let processing_handle = tokio::spawn(async move {
        while let Ok(block) = receiver.recv().await {
            let number = block.number.expect("No block number").as_u64();
            let msg = format!(
                "PROCESSING Block {} with {} transactions",
                number,
                block.transactions.len()
            )
            .green();

            println!("{msg}");
        }
    });

    download_blocks_from_to(&provider, &sender, from_block, latest_block).await?;

    let provider_ws = Provider::<Ws>::connect(RPC_URL_WS)
        .await
        .expect("Couldn't connect to ws client");

    let mut receiver_ws = sender_new_block.subscribe();

    let handle = tokio::spawn(async move {
        subscribe_to_blocks(provider_ws, sender_new_block)
            .await
            .unwrap();
    });

    let first_block_ws = receiver_ws.recv().await.unwrap();
    drop(receiver_ws);

    download_blocks_from_to(&provider, &sender, latest_block + 1, first_block_ws - 1).await?;

    download_and_broadcast_block(provider, receiver_new_block, sender).await?;

    handle.await?;
    processing_handle.await?;

    Ok(())
}

#[allow(dead_code)]
async fn download_blocks_from_to(
    provider: &Provider<Http>,
    sender: &broadcast::Sender<Block<Transaction>>,
    mut from_block: u64,
    to_block: u64,
) -> Result<(), Box<dyn Error>> {
    println!("Downloading blocks from {from_block} to {to_block}");
    while from_block <= to_block {
        // download_and_broadcast_block(provider, sender, block_to_download).await?;
        //here should be error handling, for now just panic
        //error while downloading block - try the download again
        //error while broadcasting block to receivers - break
        //get_block results to Ok(None) - no block with that number

        let result = provider.get_block_with_txs(from_block).await?;

        match result {
            Some(block) => {
                println!(
                    "Downloaded block {}, tx count {}",
                    from_block,
                    block.transactions.len()
                );

                let receiver_count = sender.send(block)?;
                println!("Block {} sent to {} receivers", from_block, receiver_count);
            }
            None => {
                println!("Result for block {} is Ok(None)", from_block);
                todo!("Handle this condition")
            }
        }

        from_block += 1;
    }

    println!("Finished downloading blocks.");

    Ok(())
}

#[allow(dead_code)]
async fn subscribe_to_blocks(
    provider: Provider<Ws>,
    sender: tokio::sync::broadcast::Sender<u64>,
) -> Result<(), Box<dyn Error>> {
    let mut stream = provider.subscribe_blocks().await?;

    while let Some(block) = stream.next().await {
        let msg = format!("WS: Block number {}", block.number.unwrap_or(U64::from(0)))
            .yellow()
            .on_black();
        println!("{msg}");

        if let Some(block_number) = block.number {
            sender
                .send(block_number.as_u64())
                .expect("Error sending block number to sender");
        }
    }
    Ok(())
}

#[allow(dead_code)]
async fn download_and_broadcast_block(
    provider: Provider<Http>,
    mut receiver: tokio::sync::broadcast::Receiver<u64>,
    sender: tokio::sync::broadcast::Sender<Block<Transaction>>,
) -> Result<(), Box<dyn Error>> {
    while let Ok(block_number) = receiver.recv().await {
        let result = provider.get_block_with_txs(block_number).await?;

        match result {
            Some(block) => {
                // println!(
                //     "Downloaded full block {}, tx count {}",
                //     block_number,
                //     block.transactions.len()
                // );

                if let Err(error) = sender.send(block) {
                    println!("error while sending blocks to receivers {}", error);
                }
            }
            None => {
                println!("Result for block {} is Ok(None)", block_number);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    const RPC_URL_WS: &str = "";
    const RPC_URL: &str = "";

    #[ignore]
    #[tokio::test]
    async fn test_download_blocks_until_latest() {
        let provider =
            Provider::<Http>::try_from(RPC_URL).expect("Couldn't instantiate http provider");

        let (sender, _receiver) = broadcast::channel(10);
        let from = 12965030;
        download_blocks_from_to(&provider, &sender, from, from + 5)
            .await
            .unwrap();
    }

    #[ignore]
    #[tokio::test]
    async fn test_subscribe_to_blocks() {
        let provider = Provider::<Ws>::connect(RPC_URL_WS)
            .await
            .expect("Couldn't instantiated ws provider");

        let (sender, mut receiver) = tokio::sync::broadcast::channel(10);

        let handle = tokio::spawn(async move {
            let mut x = 0;
            while let Ok(block_number) = receiver.recv().await {
                println!("block number received: {}", block_number);
                x += 1;
                if x >= 5 {
                    break;
                }
            }
        });

        let result = subscribe_to_blocks(provider, sender).await;
        if let Err(e) = result {
            //channel closed
            println!("{}", e);
        } else {
            assert!(result.is_ok());
        }
        let receiver_result = handle.await;
        assert!(receiver_result.is_ok());
    }

    #[ignore]
    #[tokio::test]
    async fn test_subscribe_to_blocks_v2() {
        let provider = Provider::<Ws>::connect(RPC_URL_WS)
            .await
            .expect("Couldn't instantiated ws provider");

        let (sender, mut receiver) = tokio::sync::broadcast::channel(10);

        let handle = tokio::spawn(async move {
            match subscribe_to_blocks(provider, sender).await {
                Ok(()) => {}
                Err(e) => println!("error while subscribing: {}", e),
            }
        });

        let mut x = 0;
        while let Ok(block_number) = receiver.recv().await {
            println!("block number received: {}", block_number);
            x += 1;
            if x >= 1 {
                break;
            }
        }
        drop(receiver); // channel closed

        let result = handle.await;

        match result {
            Ok(_) => {}
            Err(e) => {
                println!("{}", e)
            }
        }
    }
}
