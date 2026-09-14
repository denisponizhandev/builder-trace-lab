use thiserror::Error;

use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

use crate::pipeline::message::PipelineMessage;

#[derive(Debug, Error)]
pub enum IngestError {
    #[error("ingest channel closed")]
    ChannelClosed,
}

pub async fn run_ingest(tx: mpsc::Sender<PipelineMessage>) -> Result<(), IngestError> {
    println!("ingest started");

    loop {
        let ping = PipelineMessage::Ping;
        tx.send(ping).await.map_err(|_| IngestError::ChannelClosed)?;

        sleep(Duration::from_millis(1000)).await;
    }

    println!("ingest stopping");

    Ok(())
}