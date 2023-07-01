use std::error::Error;

use ethers::providers::{Http, Middleware, Provider};

use ethers::types::{Address, Block, BlockNumber, Bytes, Transaction, H256, U256, U64};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + 'static>> {
    // run_evm_pipeline().await?;

    Ok(())
}

async fn download_blocks_until_latest(
    from_block: u64,
    provider: Provider<Http>,
) -> Result<(), Box<dyn Error>> {
    let mut latest_block = None;

    while latest_block.is_none() {
        latest_block = get_latest_block(&provider).await;
    }

    Ok(())
}

async fn get_latest_block(provider: &Provider<Http>) -> Option<Block<H256>> {
    let latest_block = provider.get_block(BlockNumber::Latest).await;

    if let Err(e) = latest_block {
        panic!("Couldn't get latest block. Error: {e}");
    }
    latest_block.unwrap()
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

        download_blocks_until_latest(123, provider).await.unwrap();
    }
}
