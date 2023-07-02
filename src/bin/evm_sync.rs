use std::error::Error;

use ethers::providers::{Http, Middleware, Provider};

use ethers::types::{Address, Block, BlockNumber, Bytes, Transaction, H256, U256, U64};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + 'static>> {
    // run_evm_pipeline().await?;

    Ok(())
}

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
                println!("Downloaded block {}", block_to_download);
                block_to_download += U64::one();
                todo!("process transactions");
            }
            Ok(None) => {
                println!("Result for block {} is Ok(None)", block_to_download);
                todo!("Handle this condition")
            }
            Err(e) => {
                todo!("handle errors")
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

    #[tokio::test]
    async fn test_download_blocks_until_latest() {
        let provider =
            Provider::<Http>::try_from(RPC_URL).expect("Couldn't initialize evm data provider.");
        //12965030
        download_blocks_until_latest(U64([12965030]), provider)
            .await
            .unwrap();
    }
}
