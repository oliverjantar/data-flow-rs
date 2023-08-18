/*







NOT USED, old approach. New download pipeline is in evm_sync.rs







*/
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
use data_flow_rs::streaming_helpers::{pipe, DataSender};
use std::error::Error;
use tokio::task;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + 'static>> {
    run_evm_pipeline().await?;

    Ok(())
}

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::{
    fs::OpenOptions,
    sync::mpsc::{self, Receiver},
    task::JoinHandle,
};

use ethers::{
    providers::{Middleware, Provider, StreamExt, Ws},
    types::{Address, Block, Bytes, Transaction, H256, U256, U64},
};

async fn run_evm_pipeline() -> anyhow::Result<()> {
    let provider = Arc::new(
        Provider::<Ws>::connect("wss://mainnet.infura.io/ws/v3/d27480148c2646b6a42d1a4c2f786449")
            .await
            .expect("Couldn't initialize evm data provider."),
    );

    let (subscribe, h1) = start_subscribe_block_thread(provider.clone());
    let (download, h2) = start_download_transaction(provider.clone(), subscribe);
    let (transform, h3) = start_transform_thread(download);
    let h4 = start_output_writer_thread("./eth.txt", transform).await;

    h1.await?;
    h2.await?;
    h3.await?;
    h4.await?;

    Ok(())
}

fn start_subscribe_block_thread(
    provider: Arc<Provider<Ws>>,
) -> (mpsc::Receiver<U64>, JoinHandle<()>) {
    let (sender, receiver) = tokio::sync::mpsc::channel(1);

    let handle = tokio::spawn(async move {
        while let Some(block) = provider
            .subscribe_blocks()
            .await
            .expect("Error while subscribing to blocks")
            .next()
            .await
        {
            if let Some(block_number) = block.number {
                sender
                    .send(block_number)
                    .await
                    .expect("Error while sending block_number to channel.");
            }
        }
    });

    (receiver, handle)
}

fn start_download_transaction(
    provider: Arc<Provider<Ws>>,
    mut receiver: mpsc::Receiver<ethers::types::U64>,
) -> (Receiver<Block<ethers::types::Transaction>>, JoinHandle<()>) {
    let (sender, block_receiver) = tokio::sync::mpsc::channel(1);

    let handle = tokio::spawn(async move {
        while let Some(block_number) = receiver.recv().await {
            println!("new block: {:?}", block_number);
            match provider.get_block_with_txs(block_number).await {
                Ok(block) => match block {
                    Some(block) => {
                        // println!(
                        //     "Ts: {:?}, block number: {} -> {:?}",
                        //     block.timestamp,
                        //     block.number.unwrap(),
                        //     block.hash.unwrap(),
                        //     //  block.transactions
                        // );

                        if sender.send(block).await.is_err() {
                            println!("Error while sending block with txs to channel.");
                            break;
                        };
                    }
                    None => {
                        println!("No block for {}", block_number);
                    }
                },
                Err(e) => println!("Error while fetching block, {}", e),
            }
        }
    });

    (block_receiver, handle)
}

fn start_transform_thread(
    mut receiver: mpsc::Receiver<Block<ethers::types::Transaction>>,
) -> (Receiver<EvmBlock>, JoinHandle<()>) {
    let (sender, block_receiver) = tokio::sync::mpsc::channel(1);

    let handle = tokio::spawn(async move {
        //some transformation as an example.. this doesn't need to be in a separate thread

        while let Some(block) = receiver.recv().await {
            let evm_block: EvmBlock = EvmBlock::map_block(&block);

            //Bincode can only encode sequences and maps that have a knowable size ahead of time
            if sender.send(evm_block).await.is_err() {
                println!("Error while sending blockto channel.");
                break;
            };
        }
    });

    (block_receiver, handle)
}

async fn start_output_writer_thread(
    output_file_path: &str,
    mut receiver: mpsc::Receiver<EvmBlock>,
) -> JoinHandle<()> {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(output_file_path)
        .await
        .expect("Error while opening file.");

    tokio::spawn(async move {
        while let Some(block) = receiver.recv().await {
            println!("writing block {} to file", block.number.unwrap());
            // if add(&mut file, &block).await.is_err() {
            //     println!("Error while writing to file.");
            //     break;
            // }
        }
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EvmBlock {
    pub hash: H256,
    pub parent_hash: H256,
    pub number: Option<U64>,
    pub timestamp: U256,
    pub uncles: Vec<H256>,
    pub transactions: Vec<EvmTransaction>,
}

impl EvmBlock {
    pub fn new() -> Self {
        EvmBlock::default()
    }

    pub fn map_block(block: &Block<Transaction>) -> EvmBlock {
        let block = EvmBlock {
            hash: block.hash.unwrap(),
            parent_hash: block.parent_hash,
            number: block.number,
            timestamp: block.timestamp,
            uncles: block.uncles.clone(),
            transactions: block
                .transactions
                .iter()
                .map(EvmTransaction::map_tx)
                .collect(),
        };
        block
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EvmTransaction {
    pub hash: H256,
    pub nonce: U256,
    pub transaction_index: Option<U64>,
    pub from: Address,
    pub to: Option<Address>,
    pub value: U256,
    pub gas_price: Option<U256>,
    pub gas: U256,
    pub input: Bytes,
    /// ECDSA recovery id
    pub v: U64,
    /// ECDSA signature r
    pub r: U256,
    /// ECDSA signature s
    pub s: U256,
}

impl EvmTransaction {
    pub fn new() -> Self {
        EvmTransaction::default()
    }

    pub fn map_tx(tx: &Transaction) -> EvmTransaction {
        EvmTransaction {
            hash: tx.hash,
            nonce: tx.nonce,
            transaction_index: tx.transaction_index,
            from: tx.from,
            to: tx.to,
            value: tx.value,
            gas_price: tx.gas_price,
            gas: tx.gas,
            input: tx.input.clone(),
            v: tx.v,
            r: tx.r,
            s: tx.s,
        }
    }
}
