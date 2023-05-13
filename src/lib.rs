use anyhow::anyhow;
use async_std::prelude::*;
use std::error::Error;
use tokio::sync::broadcast;

#[cfg(test)]
use mockall::{automock, mock, predicate::*};
#[cfg_attr(test, automock)]
trait Outbound<T> {
    fn send(&self, packet: T) -> anyhow::Result<()>;
}

struct BroadcastSender<T> {
    sender: broadcast::Sender<T>,
}

impl<T> BroadcastSender<T>
where
    T: Clone,
{
    fn new() -> Self {
        let (sender, _recv) = broadcast::channel(100);

        Self { sender }
    }
}

impl<T> Outbound<T> for BroadcastSender<T> {
    fn send(&self, packet: T) -> anyhow::Result<()> {
        self.sender
            .send(packet)
            .map_err(|err| anyhow!("Error while sending packet to sender: {}", err))?;
        Ok(())
    }
}

async fn run<T, S, U>(mut source: T, outbound: S) -> Result<(), Box<dyn Error>>
where
    T: Stream<Item = Result<U, std::io::Error>> + Unpin,
    S: Outbound<U>,
{
    while let Some(result) = source.next().await {
        outbound.send(result?)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;
    use async_std::fs::File;
    use async_std::io::BufReader;
    use async_std::stream::StreamExt;
    use async_std::task;
    use mockall::mock;

    #[test]
    fn test_simple_receiver() {
        assert!(async_std::task::block_on(run_test_stream_data()).is_ok());
    }

    async fn run_test_stream_data() -> Result<(), Box<dyn Error>> {
        let source = File::open("./data/1").await?;
        let lines = BufReader::new(source).lines();

        let sender = BroadcastSender::<String>::new();

        let mut receiver = sender.sender.subscribe();

        let handle = task::spawn_local(async move {
            let mut values = vec![];
            while let Ok(value) = receiver.recv().await {
                values.push(value);
            }

            assert_eq!(values, vec!["this", "is", "a", "test"]);
        });

        run(lines, sender).await?;
        handle.await;

        Ok(())
    }

    #[test]
    fn test_multiple_receivers() {
        assert!(async_std::task::block_on(run_multiple_receivers()).is_ok());
    }

    async fn run_multiple_receivers() -> Result<(), Box<dyn Error>> {
        let source = File::open("./data/1").await?;
        let sender = BroadcastSender::<String>::new();
        let lines = BufReader::new(source).lines();

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

        run(lines, sender).await?;
        handle1.await;
        handle2.await;

        Ok(())
    }

    #[tokio::test]
    async fn run_with_mock_sender() {
        let mut source = futures::stream::iter(vec![Ok(1), Ok(2), Ok(1)]);

        let mut outbound = MockOutbound::new();
        outbound.expect_send().times(3).returning(|value| {
            assert!(matches!(value, 1 | 2 | 3));
            Ok(())
        });

        let result = run(&mut source, outbound).await;
        assert!(result.is_ok());
    }
}
