//! A state with delayed deletions.

// use super::combine_maintenance;
use futures::future::BoxFuture;
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};
use tokio::time::delay_until;

pub struct DelayedDelete<T>
where
    T: super::Write,
{
    inner: T,
    queue: VecDeque<(<T as super::Write>::Item, Instant)>,
    delay_for: Duration,
}

impl<T> DelayedDelete<T>
where
    T: super::Write,
{
    pub fn new(inner: T, delay_for: Duration) -> Self {
        let queue = VecDeque::new();
        Self {
            inner,
            delay_for,
            queue,
        }
    }

    pub fn schedule_delete(&mut self, item: <T as super::Write>::Item) {
        let deadline = Instant::now() + self.delay_for;
        self.queue.push_back((item, deadline));
    }

    pub fn perform(&mut self) {
        let now = Instant::now();
        while let Some(deadline) = self.next_deadline() {
            if deadline > now {
                break;
            }
            let (item, _) = self.queue.pop_front().unwrap();
            self.inner.delete(item);
        }
    }

    pub fn next_deadline(&self) -> Option<Instant> {
        self.queue.front().map(|(_, instant)| *instant)
    }
}

impl<T> super::Write for DelayedDelete<T>
where
    T: super::Write + Send,
    <T as super::Write>::Item: Send,
{
    type Item = <T as super::Write>::Item;

    fn add(&mut self, item: Self::Item) {
        self.inner.add(item);
    }

    fn update(&mut self, item: Self::Item) {
        self.inner.update(item);
    }

    fn delete(&mut self, item: Self::Item) {
        self.schedule_delete(item);
    }

    fn resync(&mut self) {
        self.queue.clear();
        self.inner.resync();
    }

    fn maintainance(&mut self) -> Vec<BoxFuture<'_, ()>> {
        let next_deadline = self.next_deadline();

        let mut maintenance = self.inner.maintainance();

        if let Some(next_deadline) = next_deadline {
            maintenance.push(Box::pin(async {
                delay_until(next_deadline.into()).await;

                let now = Instant::now();
                while let Some(deadline) = self.next_deadline() {
                    if deadline > now {
                        break;
                    }
                    let (item, _) = self.queue.pop_front().unwrap();
                    self.inner.delete(item);
                }
            }) as BoxFuture<'_, ()>)
        }

        maintenance
    }
}
