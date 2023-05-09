use std::error::Error;

use async_std::io::prelude::*;

use anyhow::anyhow;
use async_std::{io::BufReader, prelude::*};
use tokio::sync::broadcast;

trait Outbound<T> {
    fn send(&self, packet: T) -> anyhow::Result<()>;
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
    fn send(&self, packet: T) -> anyhow::Result<()> {
        self.sender
            .send(packet)
            .map_err(|err| anyhow!("Error while sending packet to sender: {}", err))?;
        Ok(())
    }
}

async fn run<T, S>(source: T, outbound: S) -> Result<(), Box<dyn Error>>
where
    T: Unpin + Sized + async_std::io::Read,
    S: Outbound<String>,
{
    let mut lines = BufReader::new(source).lines();

    while let Some(result) = lines.next().await {
        outbound.send(result?)?
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;
    use async_std::fs::File;
    use async_std::task;

    #[test]
    fn test_simple_receiver() {
        assert!(async_std::task::block_on(run_test_stream_data()).is_ok());
    }

    async fn run_test_stream_data() -> Result<(), Box<dyn Error>> {
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

        run(source, sender).await?;
        handle.await;

        Ok(())
    }

    #[test]
    fn test_multiple_receivers() {
        assert!(async_std::task::block_on(run_multiple_receivers()).is_ok());
    }

    async fn run_multiple_receivers() -> Result<(), Box<dyn Error>> {
        let source = File::open("./data/1").await?;
        let sender = DataModuleSender::<String>::new();

        let mut receiver1 = sender.sender.subscribe();
        let mut receiver2 = sender.sender.subscribe();

        let handle1 = task::spawn_local(async move {
            let mut values = vec![];
            while let Ok(value) = receiver1.recv().await {
                values.push(value);
            }

            assert_eq!(values, vec!["this", "is", "a", "test"]);
        });

        let handle2 = task::spawn_local(async move {
            let mut values = vec![];
            while let Ok(value) = receiver2.recv().await {
                values.push(value);
            }

            assert_eq!(values, vec!["this", "is", "a", "test"]);
        });

        run(source, sender).await?;
        handle1.await;
        handle2.await;

        Ok(())
    }
}
