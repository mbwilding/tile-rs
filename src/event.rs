use std::collections::HashMap;

pub struct Event<T>
where
    T: Clone,
{
    subscribers: HashMap<usize, Box<dyn Fn(T) + Send + Sync>>,
    next_id: usize,
}

impl<T> Event<T>
where
    T: Clone,
{
    pub fn new() -> Self {
        Event {
            subscribers: HashMap::new(),
            next_id: 0,
        }
    }
}

impl<T> Default for Event<T>
where
    T: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Event<T>
where
    T: Clone,
{
    pub fn subscribe<F>(&mut self, callback: F) -> usize
    where
        F: Fn(T) + Send + Sync + 'static,
    {
        let next_id = &mut self.next_id;

        let id = *next_id;
        self.subscribers.insert(id, Box::new(callback));
        *next_id += 1;

        id
    }

    pub fn unsubscribe(&mut self, id: usize) {
        self.subscribers.remove(&id);
    }

    pub fn emit(&self, payload: &T)
    where
        T: Clone + Send + Sync + 'static,
    {
        for subscriber in self.subscribers.values() {
            subscriber(payload.clone());
        }
    }
}
