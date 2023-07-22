use ethers::core::types::Block;
use ethers::providers::{Http, Middleware, Provider, Ws};
use ethers::types::{Transaction, U64};
use futures::StreamExt;
use std::error::Error;
use tokio::sync::broadcast;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + 'static>> {
    // run_evm_pipeline().await?;

    Ok(())
}

#[allow(dead_code)]
async fn download_blocks_until_latest(
    provider: &Provider<Http>,
    sender: &broadcast::Sender<Block<Transaction>>,
    from_block: u64,
) -> Result<(), Box<dyn Error>> {
    let latest_block = provider.get_block_number().await.unwrap_or_else(|err| {
        panic!("Couldn't get latest block. Error: {err}");
    });

    let mut block_to_download = from_block;

    while U64::from(block_to_download) <= latest_block {
        // download_and_broadcast_block(provider, sender, block_to_download).await?;
        //here should be error handling, for now just panic
        //error while downloading block - try the download again
        //error while broadcasting block to receivers - break
        //get_block results to Ok(None) - no block with that number

        let result = provider.get_block_with_txs(block_to_download).await?;

        match result {
            Some(block) => {
                println!(
                    "Downloaded block {}, tx count {}",
                    block_to_download,
                    block.transactions.len()
                );

                let receiver_count = sender.send(block)?;
                println!(
                    "Block {} sent to {} receivers",
                    block_to_download, receiver_count
                );
            }
            None => {
                println!("Result for block {} is Ok(None)", block_to_download);
                todo!("Handle this condition")
            }
        }

        block_to_download += 1;
    }

    Ok(())
}

#[allow(dead_code)]
async fn subscribe_to_blocks(
    provider: Provider<Ws>,
    sender: tokio::sync::mpsc::Sender<u64>,
) -> Result<(), Box<dyn Error>> {
    let mut stream = provider.subscribe_blocks().await?;

    while let Some(block) = stream.next().await {
        println!("New block number {}", block.number.unwrap_or(U64::from(0)));

        if let Some(block_number) = block.number {
            sender.send(block_number.as_u64()).await?;
        }
    }
    Ok(())
}

#[allow(dead_code)]
async fn download_and_broadcast_block(
    provider: Provider<Http>,
    mut receiver: tokio::sync::mpsc::Receiver<u64>,
    sender: tokio::sync::broadcast::Sender<Block<Transaction>>,
) -> Result<(), Box<dyn Error>> {
    while let Some(block_number) = receiver.recv().await {
        let result = provider.get_block_with_txs(block_number).await?;

        match result {
            Some(block) => {
                println!(
                    "Downloaded block {}, tx count {}",
                    block_number,
                    block.transactions.len()
                );

                let receiver_count = sender.send(block)?;
                println!(
                    "Block {} sent to {} receivers",
                    block_number, receiver_count
                );
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

    const RPC_URL_WS: &str = "wss://mainnet.infura.io/ws/v3/d27480148c2646b6a42d1a4c2f786449";
    const RPC_URL: &str = "https://eth.llamarpc.com";

    #[ignore]
    #[tokio::test]
    async fn test_download_blocks_until_latest() {
        let provider =
            Provider::<Http>::try_from(RPC_URL).expect("Couldn't instantiate http provider");

        let (sender, _receiver) = broadcast::channel(10);
        //12965030
        download_blocks_until_latest(&provider, &sender, 12965030)
            .await
            .unwrap();
    }

    #[ignore]
    #[tokio::test]
    async fn test_subscribe_to_blocks() {
        let provider = Provider::<Ws>::connect(RPC_URL_WS)
            .await
            .expect("Couldn't instantiated ws provider");

        let (sender, mut receiver) = tokio::sync::mpsc::channel(10);

        let handle = tokio::spawn(async move {
            let mut x = 0;
            while let Some(block_number) = receiver.recv().await {
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

        let (sender, mut receiver) = tokio::sync::mpsc::channel(10);

        let handle = tokio::spawn(async move {
            match subscribe_to_blocks(provider, sender).await {
                Ok(()) => {}
                Err(e) => println!("error while subscribing: {}", e),
            }
        });

        let mut x = 0;
        while let Some(block_number) = receiver.recv().await {
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
