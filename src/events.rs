use std::{collections::HashMap, sync::Arc};
use tokio::sync::{
    mpsc::{UnboundedReceiver, UnboundedSender},
    RwLock,
};

#[derive(Clone)]
struct EventSourceInner<T> {
    subscribers: Vec<UnboundedSender<T>>,
    last_event: Option<T>,
}

#[derive(Clone)]
pub struct EventSource<T>(Arc<RwLock<EventSourceInner<T>>>);

impl<T> EventSource<T>
where
    T: Clone,
{
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(EventSourceInner {
            subscribers: Vec::new(),
            last_event: None,
        })))
    }

    pub async fn subscribe(&self) -> (UnboundedReceiver<T>, Option<T>) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        let mut inner = self.0.write().await;

        inner.subscribers.push(tx);

        (rx, inner.last_event.clone())
    }

    pub async fn last_event(&self) -> Option<T> {
        self.0.read().await.last_event.clone()
    }

    pub async fn publish(&self, event: T) {
        let mut inner = self.0.write().await;
        let mut not_ok = Vec::new();
        for (i, tx) in inner.subscribers.iter().enumerate() {
            if tx.send(event.clone()).is_err() {
                not_ok.push(i);
            }
        }

        for i in not_ok.into_iter().rev() {
            inner.subscribers.swap_remove(i);
        }
        inner.last_event = Some(event);
    }
}

pub struct DispatcherInner<K, V>
where
    V: Clone,
{
    sources: HashMap<K, EventSource<V>>,
}

#[derive(Clone)]
pub struct Dispatcher<K, V>(Arc<RwLock<DispatcherInner<K, V>>>)
where
    V: Clone;

impl<K, V> Dispatcher<K, V>
where
    K: Eq + std::hash::Hash,
    V: Clone,
{
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(DispatcherInner {
            sources: HashMap::new(),
        })))
    }

    pub async fn subscribe(&self, key: K) -> (UnboundedReceiver<V>, Option<V>) {
        if let Some(source) = self.0.read().await.sources.get(&key) {
            return source.subscribe().await;
        };

        let source = EventSource::new();
        let (rx, last_event) = source.subscribe().await;

        let mut inner = self.0.write().await;
        inner.sources.insert(key, source);

        (rx, last_event)
    }

    pub async fn publish(&self, key: K, event: V) {
        if let Some(source) = self.0.read().await.sources.get(&key) {
            return source.publish(event).await;
        }

        let source = EventSource::new();
        source.publish(event).await;
        self.0.write().await.sources.insert(key, source);
    }

    pub async fn last_event(&self, key: K) -> Option<V> {
        if let Some(source) = self.0.read().await.sources.get(&key) {
            return source.last_event().await;
        }

        None
    }
}
