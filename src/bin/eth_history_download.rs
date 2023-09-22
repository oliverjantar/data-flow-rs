use ethers::providers::{Http, Middleware, Provider};

use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + 'static>> {
    let mut from_block;

    {
        let provider = Provider::<Http>::try_from("").expect("Couldn't instantiate http provider");

        from_block = provider
            .get_block_number()
            .await
            .unwrap_or_else(|err| panic!("Couldn't get latest block. Error: {err}"))
            .as_u64();
    }

    from_block -= 10;

    let downloader = data_flow_rs::startup::run("".to_owned(), "".to_owned(), 1)?;
    downloader
        .download_history_and_subscribe_for_new_blocks(from_block)
        .await?;

    Ok(())
}
