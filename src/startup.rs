use colored::Colorize;
use std::error::Error;
use tokio::sync::broadcast;

use ethers::types::{Block, Transaction};

use crate::downloader::Downloader;

pub async fn run(
    ws_url: String,
    http_url: String,
    buffer_size: usize,
    from_block: u64,
) -> Result<(), Box<dyn Error>> {
    //block download and processing
    let (sender, mut receiver) = broadcast::channel::<Block<Transaction>>(buffer_size);

    let downloader = Downloader::new(ws_url, http_url, buffer_size, sender);

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

    downloader
        .download_history_and_subscribe_for_new_blocks(from_block)
        .await?;

    processing_handle.await?;

    Ok(())
}
