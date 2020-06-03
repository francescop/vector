//! Local representation of the Kubernetes API resources state.

// use futures::future::BoxFuture;
use async_trait::async_trait;
use k8s_openapi::{apimachinery::pkg::apis::meta::v1::ObjectMeta, Metadata};

// pub mod delayed_delete;
pub mod evmap;
pub mod mock;

// mod combine_maintenance;
// use combine_maintenance::combine_maintenance;

/// Provides the interface for write access to the cached state.
/// Used by [`super::reflector::Reflector`].
///
/// This abstraction allows easily stacking storage behaviour logic, without
/// exploding the complexity at the [`super::reflector::Reflector`] level.
#[async_trait]
pub trait Write {
    /// A type of the k8s resource the state operates on.
    type Item: Metadata<Ty = ObjectMeta> + Send;

    /// Add an object to the state.
    async fn add(&mut self, item: Self::Item);

    /// Update an object at the state.
    async fn update(&mut self, item: Self::Item);

    /// Delete on object from the state.
    async fn delete(&mut self, item: Self::Item);

    /// Notify the state that resync is in progress.
    async fn resync(&mut self);

    // /// Perform the internal maintenance operations on the storage.
    // /// If storage requires maintenance, this function returns a future that
    // /// has to be polled to carry on the background operations.
    // fn maintainance(&mut self) -> Vec<BoxFuture<'_, ()>> {
    //     vec![]
    // }

    // fn maintainance(&mut self) {
    //     vec![]
    // }

    // fn wait_for_next_maintenance(&self) {
    //     vec![]
    // }
}

// struct Maintenance {
//     fn wait_for_next_request(&self) -> impl Future<Output = ()>{

//     }
// }
