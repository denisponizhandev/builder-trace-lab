use thiserror::Error;

use tokio::sync::mpsc;
use sqlx::PgPool;

use crate::pipeline::message::PipelineMessage;

#[derive(Debug, Error)] 
pub enum ProcessorError {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

async fn run_test_query(pool: &PgPool) -> Result<(), ProcessorError> {
    let i: i32 = sqlx::query_scalar("SELECT 1;").fetch_one(pool).await?;

    println!("db ok: {}", i);

    Ok(())
}

pub async fn run_processor(mut rx: mpsc::Receiver<PipelineMessage>, pool: PgPool) -> Result<(), ProcessorError> {
    println!("processor started");

    while let Some(msg) = rx.recv().await {
        match msg {
            PipelineMessage::Ping => {
                println!("received {:?} from channel", msg);
                run_test_query(&pool).await?;
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