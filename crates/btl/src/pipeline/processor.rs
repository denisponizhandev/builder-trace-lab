use thiserror::Error;

use tokio::sync::mpsc;
use sqlx::PgPool;

use crate::pipeline::message::{PipelineMessage, AcceptedBundle};

#[derive(Debug, Error)] 
pub enum ProcessorError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

async fn persist_inbound_bundle(pool: &PgPool, bundle: &AcceptedBundle) -> Result<(), ProcessorError> {
    sqlx::query(
        r#"
        INSERT INTO bundles (bundle_hash, target_block, tx_count, received_at)
        VALUES ($1, $2, $3, $4)
        "#
    )
    .bind(bundle.bundle_hash())
    .bind(bundle.target_block())
    .bind(bundle.tx_count())
    .bind(bundle.received_at())
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn run_processor(mut rx: mpsc::Receiver<PipelineMessage>, pool: PgPool) -> Result<(), ProcessorError> {
    println!("processor started");

    while let Some(msg) = rx.recv().await {
        match msg {
            PipelineMessage::BundleAccepted(b) => {                
                persist_inbound_bundle(&pool, &b).await?;
            },
            PipelineMessage::Shutdown => {
                println!("received {:?} from channel", msg);
                break;
            },
        }
    }

    println!("processor exited");

    Ok(())
}