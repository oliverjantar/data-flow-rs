use std::error::Error;

use async_std::{fs::File, io::BufReader, prelude::*};

fn main() {
    println!("running");
    run_example_1().unwrap();
}

trait Outbound {
    fn send(&self, line: String);
}

struct PrintOutbound {}

impl Outbound for PrintOutbound {
    fn send(&self, line: String) {
        println!("{}", line);
    }
}

impl PrintOutbound {
    fn new() -> Self {
        Self {}
    }
}

async fn run<T>(source: T, outbound: Box<dyn Outbound>) -> Result<(), Box<dyn Error>>
where
    T: Unpin + Sized + async_std::io::Read,
{
    let mut lines = BufReader::new(source).lines();

    while let Some(line) = lines.next().await {
        // let line = line?;
        outbound.send(line?)
    }

    Ok(())
}

fn run_example_1() -> Result<(), Box<dyn Error>> {
    async_std::task::block_on(run_test())
}

async fn run_test() -> Result<(), Box<dyn Error>> {
    let source = File::open("./data/1").await?;
    let outbound = Box::new(PrintOutbound::new());
    async_std::task::block_on(run(source, outbound))?;

    Ok(())
}
