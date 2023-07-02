use std::error::Error;

use ethers::providers::{Http, Middleware, Provider, Ws};

use ethers::types::U64;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + 'static>> {
    // run_evm_pipeline().await?;

    Ok(())
}

#[allow(dead_code)]
async fn download_blocks_until_latest(
    from_block: U64,
    provider: Provider<Http>,
) -> Result<(), Box<dyn Error>> {
    let latest_block = provider.get_block_number().await.unwrap_or_else(|err| {
        panic!("Couldn't get latest block. Error: {err}");
    });

    let mut block_to_download = from_block;

    while block_to_download <= latest_block {
        let result = provider.get_block_with_txs(block_to_download).await;

        match result {
            Ok(Some(block)) => {
                println!(
                    "Downloaded block {}, tx count {}",
                    block_to_download,
                    block.transactions.len()
                );
                block_to_download += U64::one();
                todo!("process transactions");
            }
            Ok(None) => {
                println!("Result for block {} is Ok(None)", block_to_download);
                todo!("Handle this condition")
            }
            Err(e) => {
                eprintln!(
                    "Error while downloading block {}. Error: {}",
                    block_to_download, e
                );
                todo!("handle errors")
            }
        }
    }

    Ok(())
}

#[allow(dead_code)]
async fn subscribe_to_blocks(provider: Provider<Ws>) {
    while let Some(block) = provider
        .subscribe_blocks()
        .await
        .expect("Error while subscribing to blocks")
        .next()
        .await
    {
        println!(
            "Downloaded block {}",
            block.number.expect("No block number")
        )
        // if let Some(block_number) = block.number {
        //     sender
        //         .send(block_number)
        //         .await
        //         .expect("Error while sending block_number to channel.");
        // }
    }
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
        //12965030
        download_blocks_until_latest(U64([12965030]), provider)
            .await
            .unwrap();
    }

    #[ignore]
    #[tokio::test]
    async fn test_subscribe_to_blocks() {
        let provider = Provider::<Ws>::connect(RPC_URL_WS)
            .await
            .expect("Couldn't instantiated ws provider");

        subscribe_to_blocks(provider).await;
    }
}
