use std::error::Error;

use async_std::io::prelude::*;

use async_std::{io::BufReader, prelude::*};
use tokio::sync::broadcast;

trait Outbound<T> {
    fn send(&self, packet: T);
}

struct DataModuleSender<T> {
    sender: broadcast::Sender<T>,
}

impl<T> DataModuleSender<T>
where
    T: Clone,
{
    fn new() -> Self {
        let (sender, _recv) = broadcast::channel(100);

        Self { sender }
    }
}

impl<T> Outbound<T> for DataModuleSender<T> {
    fn send(&self, packet: T) {
        if self.sender.send(packet).is_err() {
            eprintln!("Couldn't send packet to sender.");
        }
    }
}

async fn run<T>(source: T, outbound: Box<dyn Outbound<String>>) -> Result<(), Box<dyn Error>>
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

#[cfg(test)]
mod tests {

    use super::*;
    use async_std::fs::File;
    use async_std::task;

    #[test]
    fn test_simple_receiver() -> Result<(), Box<dyn Error>> {
        async_std::task::block_on(run_test_2())
    }

    async fn run_test_2() -> Result<(), Box<dyn Error>> {
        let source = File::open("./data/1").await?;
        let sender = DataModuleSender::<String>::new();

        let mut receiver = sender.sender.subscribe();

        let handle = task::spawn_local(async move {
            let mut values = vec![];
            while let Ok(value) = receiver.recv().await {
                values.push(value);
            }

            assert_eq!(values, vec!["this", "is", "a", "test"]);
        });

        run(source, Box::new(sender)).await?;
        handle.await;

        Ok(())
    }
}
