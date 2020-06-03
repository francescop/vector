//! A mock state.

#![cfg(test)]

use async_trait::async_trait;
use futures::{
    channel::mpsc::{Receiver, Sender},
    SinkExt, StreamExt,
};
use k8s_openapi::{apimachinery::pkg::apis::meta::v1::ObjectMeta, Metadata};

#[derive(Debug, PartialEq, Eq)]
pub enum OpKind {
    Add,
    Update,
    Delete,
}

pub enum ScenarioEvent<T>
where
    T: Metadata<Ty = ObjectMeta> + Send,
{
    Op(T, OpKind),
    Resync,
}

impl<T> ScenarioEvent<T>
where
    T: Metadata<Ty = ObjectMeta> + Send,
{
    pub fn unwrap_op(self) -> (T, OpKind) {
        match self {
            ScenarioEvent::Op(val, op) => (val, op),
            ScenarioEvent::Resync => panic!("unwrap_op on resync"),
        }
    }
}

pub struct Writer<T>
where
    T: Metadata<Ty = ObjectMeta> + Send,
{
    events_tx: Sender<ScenarioEvent<T>>,
    actions_rx: Receiver<()>,
}

impl<T> Writer<T>
where
    T: Metadata<Ty = ObjectMeta> + Send,
{
    pub fn new(events_tx: Sender<ScenarioEvent<T>>, actions_rx: Receiver<()>) -> Self {
        Self {
            events_tx,
            actions_rx,
        }
    }
}

#[async_trait]
impl<T> super::Write for Writer<T>
where
    T: Metadata<Ty = ObjectMeta> + Send,
{
    type Item = T;

    async fn add(&mut self, item: Self::Item) {
        self.events_tx
            .send(ScenarioEvent::Op(item, OpKind::Add))
            .await
            .unwrap();
        self.actions_rx.next().await.unwrap();
    }

    async fn update(&mut self, item: Self::Item) {
        self.events_tx
            .send(ScenarioEvent::Op(item, OpKind::Update))
            .await
            .unwrap();
        self.actions_rx.next().await.unwrap();
    }

    async fn delete(&mut self, item: Self::Item) {
        self.events_tx
            .send(ScenarioEvent::Op(item, OpKind::Delete))
            .await
            .unwrap();
        self.actions_rx.next().await.unwrap();
    }

    async fn resync(&mut self) {
        self.events_tx.send(ScenarioEvent::Resync).await.unwrap();
        self.actions_rx.next().await.unwrap();
    }
}
