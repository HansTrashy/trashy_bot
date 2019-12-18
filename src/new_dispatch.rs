use std::collections::HashMap;
use std::hash::Hash;
use std::time::{Duration, Instant};

type ListenerAction = Box<dyn Fn() + Send + Sync>;

pub struct Listener {
    action: ListenerAction,
    expiration: Instant,
}

impl Listener {
    pub fn new(duration: Duration, action: ListenerAction) -> Self {
        Self {
            action,
            expiration: Instant::now() + duration,
        }
    }
}

pub struct Dispatcher<K> {
    listener: HashMap<K, Vec<Listener>>,
}

impl<K> Dispatcher<K>
where
    K: Eq + Hash,
{
    pub fn new() -> Self {
        Self {
            listener: HashMap::new(),
        }
    }

    pub fn add_listener(&mut self, id: K, listener: Listener) {
        let entry = self.listener.entry(id).or_default();
        entry.push(listener);
    }

    pub fn dispatch_event(&self, id: K) {
        if let Some(listener) = self.listener.get(&id) {
            for l in listener {
                (l.action)()
            }
        }
    }

    pub fn check_expiration(&mut self) {
        for (_, listener) in self.listener.iter_mut() {
            listener.retain(|l| Instant::now() < l.expiration);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_listener() {
        let mut dispatcher = Dispatcher::new();

        dispatcher.add_listener(
            1,
            Listener::new(Duration::from_secs(100), Box::new(|| println!("Test"))),
        );

        dispatcher.dispatch_event(1);
    }
}
