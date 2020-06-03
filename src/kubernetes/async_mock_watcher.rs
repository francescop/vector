//! A mock watcher.

#![cfg(test)]

use super::watcher::{self, Watcher};
use futures::{future::BoxFuture, stream::BoxStream};
use k8s_openapi::{WatchOptional, WatchResponse};
use serde::de::DeserializeOwned;
use std::{marker::PhantomData, task::Poll};

/// A mock watcher, useful for tests.
#[derive(Debug)]
pub struct AsyncMockWatcher<T, FI> {
    data_type: PhantomData<T>,
    invoke_fn: FI,
}

impl<T, FI> AsyncMockWatcher<T, FI> {
    /// Create a new [`AsyncMockWatcher`].
    pub fn new(invoke_fn: FI) -> Self {
        let data_type = PhantomData;
        Self {
            data_type,
            invoke_fn,
        }
    }
}

impl<T, FI, FS> Watcher for AsyncMockWatcher<T, FI>
where
    T: DeserializeOwned + Send + Sync + 'static,
    FI: for<'a> FnMut(
            WatchOptional<'a>,
        ) -> Poll<Result<FS, watcher::invocation::Error<InvocationError>>>
        + Send
        + 'static,
    FS: FnMut() -> Option<Result<WatchResponse<T>, StreamError>> + Send + Sync + Unpin + 'static,
{
    type Object = T;

    type StreamError = StreamError;
    type Stream = BoxStream<'static, Result<WatchResponse<Self::Object>, Self::StreamError>>;

    type InvocationError = InvocationError;

    fn watch<'a>(
        &'a mut self,
        watch_optional: WatchOptional<'a>,
    ) -> BoxFuture<'a, Result<Self::Stream, watcher::invocation::Error<Self::InvocationError>>>
    {
        let val = (self.invoke_fn)(watch_optional);
        let result = match val {
            Poll::Pending => {
                return Box::pin(async {
                    tokio::task::yield_now().await;
                    panic!()
                });
            }
            Poll::Ready(result) => result,
        };
        let result = result.map(|stream_fn| {
            Box::pin(MockWatcherStream {
                stream_fn,
                data_type: PhantomData,
            }) as Self::Stream
        });
        Box::pin(futures::future::ready(result))
    }
}

use super::mock_watcher::MockWatcherStream;
pub use super::mock_watcher::{InvocationError, StreamError};
