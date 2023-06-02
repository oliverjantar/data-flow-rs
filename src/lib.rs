#![feature(async_fn_in_trait)]
#![feature(return_position_impl_trait_in_trait)]

use anyhow::{anyhow, Result};
use core::fmt::Debug;
use futures_lite::Stream;
use std::error::Error;
use tokio::sync::broadcast;
use tokio_stream::StreamExt;

#[cfg(test)]
use mockall::{automock, predicate::*};
#[cfg_attr(test, automock)]
trait Outbound<T> {
    fn send(&self, packet: T) -> anyhow::Result<()>;
}

async fn pipe<T, S, U>(mut source: T, outbound: S) -> Result<(), Box<dyn Error>>
where
    T: Stream<Item = Result<U, std::io::Error>> + Unpin,
    S: Outbound<U>,
{
    while let Some(result) = source.next().await {
        outbound.send(result?)?;
    }

    Ok(())
}

struct DataSender<T> {
    sender: broadcast::Sender<T>,
}

impl<T> DataSender<T>
where
    T: Clone,
{
    fn new(capacity: usize) -> Self {
        let (sender, _recv) = broadcast::channel(capacity);

        Self { sender }
    }
}

impl<T> Outbound<T> for DataSender<T> {
    fn send(&self, packet: T) -> anyhow::Result<()> {
        self.sender
            .send(packet)
            .map_err(|err| anyhow!("Error while sending packet to sender: {}", err))?;
        Ok(())
    }
}

struct DataReceiver<T, U>
where
    T: Clone,
    U: ProcessingModule<T>,
{
    receiver: broadcast::Receiver<T>,
    //I have decided to have here only one processing module for now and to scale the processing of different modules by creating multiple subscribers.
    //With this approach it won't call dynamic dispatch because every processing module is known at compile time.
    //It would be good to create a different data receiver with a simple receiver (not broadcast) and to have multiple processing modules in it.
    //Then start receiving would spawn multiple tasks for every processing module. This approach will use the dynamic dispatch as there would be different implementations of processing modules
    processing_module: U,
}

impl<T, U> DataReceiver<T, U>
where
    T: Clone + Send + 'static,
    U: ProcessingModule<T>,
{
    fn new(receiver: broadcast::Receiver<T>, processing_module: U) -> Self {
        Self {
            receiver,
            processing_module,
        }
    }

    // async fn receiver_to_stream(&self) -> impl Stream<Item = T> {
    //     let stream = BroadcastStream::new(self.receiver);

    // stream.filter_map(|result| {
    //     if let Ok(value) = result {
    //         async move { self.processing_module.process(value) }
    //     }
    // })
    // }

    async fn start_receiving(&mut self) -> anyhow::Result<()> {
        while let Ok(value) = self.receiver.recv().await {
            self.processing_module.process(value).await?
        }
        Ok(())
    }
}

trait ProcessingModule<T> {
    async fn process(&self, value: T) -> anyhow::Result<()>;

    //  async fn process_stream(&self, stream: impl Stream<Item = T> + Unpin) -> impl Stream<Item = T>;
}

struct FilterMapProcessingModule<T> {
    filters: Vec<Box<dyn Fn(&T) -> bool + Send + Sync>>,
    maps: Vec<Box<dyn Fn(&mut T) + Send + Sync>>,
}

impl<T> FilterMapProcessingModule<T> {
    fn new(
        filters: Vec<Box<dyn Fn(&T) -> bool + Send + Sync>>,
        maps: Vec<Box<dyn Fn(&mut T) + Send + Sync>>,
    ) -> Self {
        Self { filters, maps }
    }
}

impl<T> ProcessingModule<T> for FilterMapProcessingModule<T>
where
    T: Debug,
{
    async fn process(&self, mut value: T) -> anyhow::Result<()> {
        for func in self.filters.iter() {
            if !func(&value) {
                return Ok(());
            }
        }

        for func in self.maps.iter() {
            func(&mut value);
        }

        println!("value {:?}:", value);

        Ok(())
    }

    // async fn process_stream(
    //     &self,
    //     mut stream: impl Stream<Item = T> + Unpin,
    // ) -> impl Stream<Item = T> {
    //     let maps_clone = self.maps.clone();
    //     Box::pin(stream.filter_map(move |item| {
    //         let mut current_item = Some(item);
    //         for map in &maps_clone {
    //             current_item = map(current_item?);
    //         }
    //         futures::future::ready(current_item)
    //     }))
    // }
}

#[cfg(test)]
mod tests {

    use std::vec;

    use super::*;
    use tokio::task;

    #[tokio::test]
    async fn run_test_stream_data() -> Result<(), Box<dyn Error>> {
        let stream = futures::stream::iter([
            Ok("this".to_owned()),
            Ok("is".to_owned()),
            Ok("a".to_owned()),
            Ok("test".to_owned()),
        ]);

        let sender = DataSender::<String>::new(100);

        let mut receiver = sender.sender.subscribe();

        let handle = task::spawn(async move {
            let mut values = vec![];
            while let Ok(value) = receiver.recv().await {
                values.push(value);
            }

            assert_eq!(values, vec!["this", "is", "a", "test"]);
        });

        pipe(stream, sender).await?;
        handle.await?;

        Ok(())
    }

    #[tokio::test]
    async fn run_multiple_receivers() -> Result<(), Box<dyn Error>> {
        let mut stream = futures::stream::iter(vec![
            Ok("this".to_owned()),
            Ok("is".to_owned()),
            Ok("a".to_owned()),
            Ok("test".to_owned()),
        ]);

        let sender = DataSender::<String>::new(100);

        let mut receiver1 = sender.sender.subscribe();
        let mut receiver2 = sender.sender.subscribe();

        let handle1 = task::spawn(async move {
            let mut values = vec![];
            while let Ok(value) = receiver1.recv().await {
                values.push(value);
            }

            assert_eq!(values, vec!["this", "is", "a", "test"]);
        });

        let handle2 = task::spawn(async move {
            let mut values = vec![];
            while let Ok(value) = receiver2.recv().await {
                values.push(value);
            }

            assert_eq!(values, vec!["this", "is", "a", "test"]);
        });

        pipe(stream, sender).await?;
        handle1.await?;
        handle2.await?;

        Ok(())
    }

    #[tokio::test]
    async fn run_with_mock_sender() {
        let source = futures::stream::iter(vec![Ok(1), Ok(2), Ok(1)]);

        let mut outbound = MockOutbound::new();
        outbound.expect_send().times(3).returning(|value| {
            assert!(matches!(value, 1 | 2 | 3));
            Ok(())
        });

        let result = pipe(source, outbound).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_filter_map_processing() -> Result<(), Box<dyn Error>> {
        let stream = futures::stream::iter([
            Ok("this".to_owned()),
            Ok("is".to_owned()),
            Ok("a".to_owned()),
            Ok("test".to_owned()),
        ]);

        let sender = DataSender::<String>::new(100);

        let filter: Box<dyn Fn(&String) -> bool + Send + Sync> = Box::new(|value| value != "test");
        let map: Box<dyn Fn(&mut String) + Send + Sync> = Box::new(|_| {});

        let filters = vec![filter];
        let maps = vec![map];

        let processing_module = FilterMapProcessingModule::<String>::new(filters, maps);

        let receiver = sender.sender.subscribe();

        let mut data_processing = DataReceiver::new(receiver, processing_module);

        let handle = task::spawn(async move { data_processing.start_receiving().await });

        pipe(stream, sender).await?;
        handle.await??;

        Ok(())
    }
}
