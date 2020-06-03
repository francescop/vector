//! A mock watcher.

#![cfg(test)]

use super::watcher::{self, Watcher};
use async_stream::try_stream;
use futures::channel::mpsc::{Receiver, Sender};
use futures::{future::BoxFuture, stream::BoxStream, SinkExt, StreamExt};
use k8s_openapi::{WatchOptional, WatchResponse};
use serde::de::DeserializeOwned;

#[derive(Debug, PartialEq)]
pub enum ScenarioEvent {
    Invocation,
    Stream,
}

pub enum ScenarioActionInvocation<T>
where
    T: DeserializeOwned,
{
    Ok(Receiver<ScenarioActionStream<T>>),
    ErrDesync,
    ErrOther,
}

pub enum ScenarioActionStream<T>
where
    T: DeserializeOwned,
{
    Ok(WatchResponse<T>),
    Err,
    Done,
}

/// A mock watcher, useful for tests.
pub struct ChanMockWatcher<T>
where
    T: DeserializeOwned,
{
    events_tx: Sender<ScenarioEvent>,
    invocation_rx: Receiver<ScenarioActionInvocation<T>>,
}

impl<T> ChanMockWatcher<T>
where
    T: DeserializeOwned,
{
    /// Create a new [`ChanMockWatcher`].
    pub fn new(
        events_tx: Sender<ScenarioEvent>,
        invocation_rx: Receiver<ScenarioActionInvocation<T>>,
    ) -> Self {
        Self {
            events_tx,
            invocation_rx,
        }
    }
}

impl<T> Watcher for ChanMockWatcher<T>
where
    T: DeserializeOwned + Send + Sync + Unpin + 'static,
{
    type Object = T;

    type StreamError = StreamError;
    type Stream = BoxStream<'static, Result<WatchResponse<Self::Object>, Self::StreamError>>;

    type InvocationError = InvocationError;

    fn watch<'a>(
        &'a mut self,
        _watch_optional: WatchOptional<'a>,
    ) -> BoxFuture<'a, Result<Self::Stream, watcher::invocation::Error<Self::InvocationError>>>
    {
        let mut stream_events_tx = self.events_tx.clone();
        Box::pin(async move {
            self.events_tx
                .send(ScenarioEvent::Invocation)
                .await
                .unwrap();

            let action = self.invocation_rx.next().await.unwrap();
            match action {
                ScenarioActionInvocation::Ok(mut stream_rx) => {
                    let stream = Box::pin(try_stream! {
                        loop {
                            stream_events_tx.send(ScenarioEvent::Stream)
                                .await
                                .unwrap();

                            let action = stream_rx.next().await.unwrap();
                            match action {
                                ScenarioActionStream::Ok(val) => {
                                    yield val
                                },
                                ScenarioActionStream::Err => {
                                    Err(StreamError)?;
                                    break;
                                },
                                ScenarioActionStream::Done => break,
                            }
                        }
                    })
                        as BoxStream<
                            'static,
                            Result<WatchResponse<Self::Object>, Self::StreamError>,
                        >;
                    Ok(stream)
                }
                ScenarioActionInvocation::ErrDesync => {
                    Err(watcher::invocation::Error::desync(InvocationError))
                }
                ScenarioActionInvocation::ErrOther => {
                    Err(watcher::invocation::Error::other(InvocationError))
                }
            }
        })
    }
}

pub use super::mock_watcher::{InvocationError, StreamError};
use std::fmt::Debug;
