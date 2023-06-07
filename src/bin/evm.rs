use data_flow_rs::streaming_helpers::{pipe, DataReceiver, DataSender, Outbound, ProcessingModule};
use futures::Stream;
use std::error::Error;
use tokio::task;
use tokio_stream::StreamExt;

#[tokio::main]
pub async fn main() {
    // let x =
}

async fn test_stream_data() -> Result<(), Box<dyn Error>> {
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
