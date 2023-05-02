use std::error::Error;

use async_std::{fs::File, io::BufReader, prelude::*};

fn main() {
    println!("running");
}

trait Outbound {
    fn send(&self) {}
}

async fn run<T>(source: T, outbound: Box<dyn Outbound>) -> Result<(), Box<dyn Error>>
where
    T: Unpin + Sized + async_std::io::Read,
{
    let mut lines = BufReader::new(source).lines();

    while let Some(line) = lines.next().await {
        outbound.send()
    }

    Ok(())
}
