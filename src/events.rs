use std::{collections::HashMap, sync::Arc};
use tokio::sync::{
    mpsc::{UnboundedReceiver, UnboundedSender},
    Mutex, RwLock,
};

#[derive(Clone)]
struct EventSourceInner<T> {
    subscribers: Vec<UnboundedSender<T>>,
    last_event: Option<T>,
}

#[derive(Clone)]
pub struct EventSource<T>(Arc<Mutex<EventSourceInner<T>>>);

impl<T> EventSource<T>
where
    T: Clone,
{
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(EventSourceInner {
            subscribers: Vec::new(),
            last_event: None,
        })))
    }

    pub async fn subscribe(&self) -> (UnboundedReceiver<T>, Option<T>) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        let mut inner = self.0.lock().await;

        inner.subscribers.push(tx);

        (rx, inner.last_event.clone())
    }

    pub async fn last_event(&self) -> Option<T> {
        self.0.lock().await.last_event.clone()
    }

    pub async fn publish(&self, event: T) {
        let mut inner = self.0.lock().await;

        inner
            .subscribers
            .retain(|tx| tx.send(event.clone()).is_ok());

        inner.last_event = Some(event.clone());
    }
}

#[derive(Clone)]
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
        let res = {
            let inner = self.0.read().await;

            if let Some(source) = inner.sources.get(&key) {
                Some(source.subscribe().await)
            } else {
                None
            }
        };

        match res {
            Some((rx, last_event)) => (rx, last_event),
            None => {
                let source = EventSource::new();
                let (rx, last_event) = source.subscribe().await;

                let mut inner = self.0.write().await;
                inner.sources.insert(key, source);

                (rx, last_event)
            }
        }
    }

    pub async fn publish(&self, key: K, event: V) {
        let mut inner = self.0.write().await;

        if let Some(source) = inner.sources.get(&key) {
            source.publish(event).await;
        } else {
            let source = EventSource::new();
            source.publish(event).await;
            inner.sources.insert(key, source);
        }
    }

    pub async fn last_event(&self, key: K) -> Option<V> {
        let inner = self.0.read().await;

        if let Some(source) = inner.sources.get(&key) {
            source.last_event().await
        } else {
            None
        }
    }
}
