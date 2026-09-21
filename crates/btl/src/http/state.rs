use tokio::sync::mpsc;

use crate::pipeline::message::PipelineMessage;

#[derive(Clone)]
pub struct HttpState {
    pub tx: mpsc::Sender<PipelineMessage>
}

impl HttpState {
    pub fn new(tx: mpsc::Sender<PipelineMessage>) -> Self {
        Self {
            tx
        }
    }
}