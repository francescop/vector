use futures::{future::BoxFuture, stream::FuturesUnordered};

pub fn combine_maintenance(
    iter: impl IntoIterator<Item = Option<BoxFuture<'static, ()>>>,
) -> Option<BoxFuture<'static, ()>> {
    let list: FuturesUnordered<BoxFuture<'static, ()>> =
        iter.into_iter().filter_map(|val| val).collect();
    if list.is_empty() {
        return None;
    }
    Some(Box::pin(async {
        list.await;
    } as BoxFuture<'_, ()>))
}
