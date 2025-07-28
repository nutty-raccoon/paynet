use anyhow::{Error, anyhow};
use async_stream::try_stream;
use futures::{Stream, StreamExt};
use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::{Duration, Instant},
};
use tokio::time::sleep;
use tokio_retry::strategy::ExponentialBackoff;

use crate::pb::sf::substreams::rpc::v2::{
    BlockScopedData, BlockUndoSignal, Request, Response, response::Message,
};
use crate::pb::sf::substreams::v1::Modules;

use crate::substreams::SubstreamsEndpoint;

pub enum BlockResponse {
    New(BlockScopedData),
    Undo(BlockUndoSignal),
}

pub struct SubstreamsStream {
    stream: Pin<Box<dyn Stream<Item = Result<BlockResponse, Error>> + Send>>,
}

impl SubstreamsStream {
    pub fn new(
        endpoint: Arc<SubstreamsEndpoint>,
        cursor: Option<String>,
        modules: Option<Modules>,
        output_module_name: String,
        start_block: i64,
        end_block: u64,
    ) -> Self {
        SubstreamsStream {
            stream: Box::pin(stream_blocks(
                endpoint,
                cursor,
                modules,
                output_module_name,
                start_block,
                end_block,
            )),
        }
    }
}

// Create the Stream implementation that streams blocks with auto-reconnection.
fn stream_blocks(
    endpoint: Arc<SubstreamsEndpoint>,
    cursor: Option<String>,
    modules: Option<Modules>,
    output_module_name: String,
    start_block_num: i64,
    stop_block_num: u64,
) -> impl Stream<Item = Result<BlockResponse, Error>> {
    let mut latest_cursor = cursor.unwrap_or_default();
    let mut backoff = ExponentialBackoff::from_millis(500).max_delay(Duration::from_secs(45));
    let mut last_progress_report = Instant::now();

    try_stream! {
        loop {
            println!("Blockstreams disconnected, connecting (endpoint {}, start block {}, stop block {}, cursor {})",
                &endpoint,
                start_block_num,
                stop_block_num,
                &latest_cursor
            );

            let result = endpoint.clone().substreams(Request {
                start_block_num,
                start_cursor: latest_cursor.clone(),
                stop_block_num,
                final_blocks_only: false,
                modules: modules.clone(),
                output_module: output_module_name.clone(),
                production_mode: true,
                debug_initial_store_snapshot_for_modules: vec![],
            }).await;

            match result {
                Ok(stream) => {
                    println!("Blockstreams connected");

                    let mut encountered_error = false;
                    for await response in stream{
                        match process_substreams_response(response, &mut last_progress_report).await {
                            BlockProcessedResult::BlockScopedData(block_scoped_data) => {
                                backoff = ExponentialBackoff::from_millis(500).max_delay(Duration::from_secs(45));

                                let cursor = block_scoped_data.cursor.clone();
                                yield BlockResponse::New(block_scoped_data);

                                latest_cursor = cursor;
                            },
                            BlockProcessedResult::BlockUndoSignal(block_undo_signal) => {
                                backoff = ExponentialBackoff::from_millis(500).max_delay(Duration::from_secs(45));

                                let cursor = block_undo_signal.last_valid_cursor.clone();
                                yield BlockResponse::Undo(block_undo_signal);

                                latest_cursor = cursor;
                            },
                            BlockProcessedResult::Skip() => {},
                            BlockProcessedResult::TonicError(status) => {
                                if status.code() == tonic::Code::Unauthenticated {
                                    return Err(anyhow::Error::new(status.clone()))?;
                                }

                                println!("Received tonic error {:#}", status);
                                encountered_error = true;
                                break;
                            },
                        }
                    }

                    if !encountered_error {
                        println!("Stream completed, reached end block");
                        return
                    }
                },
                Err(e) => {
                    println!("Unable to connect to endpoint: {:#}", e);
                }
            }

            if let Some(duration) = backoff.next() {
                sleep(duration).await
            } else {
                return Err(anyhow!("backoff requested to stop retrying, quitting"))?;
            }
        }
    }
}

enum BlockProcessedResult {
    Skip(),
    BlockScopedData(BlockScopedData),
    BlockUndoSignal(BlockUndoSignal),
    TonicError(tonic::Status),
}

async fn process_substreams_response(
    result: Result<Response, tonic::Status>,
    last_progress_report: &mut Instant,
) -> BlockProcessedResult {
    let response = match result {
        Ok(v) => v,
        Err(e) => return BlockProcessedResult::TonicError(e),
    };

    match response.message {
        Some(Message::Session(session)) => {
            println!(
                "Received session message (Workers {}, Trace ID {})",
                session.max_parallel_workers, &session.trace_id
            );
            BlockProcessedResult::Skip()
        }
        Some(Message::BlockScopedData(block_scoped_data)) => {
            BlockProcessedResult::BlockScopedData(block_scoped_data)
        }
        Some(Message::BlockUndoSignal(block_undo_signal)) => {
            BlockProcessedResult::BlockUndoSignal(block_undo_signal)
        }
        Some(Message::Progress(progress)) => {
            let now = Instant::now();
            if now.duration_since(*last_progress_report) > Duration::from_secs(15) {
                *last_progress_report = now;
                println!("Progress: {:#?}", progress);
            }

            BlockProcessedResult::Skip()
        }
        None => {
            println!("Got None on substream message");
            BlockProcessedResult::Skip()
        }
        _ => BlockProcessedResult::Skip(),
    }
}

impl Stream for SubstreamsStream {
    type Item = Result<BlockResponse, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.stream.poll_next_unpin(cx)
    }
}
