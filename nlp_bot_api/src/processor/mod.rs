pub mod container;
pub mod message;

use crate::store::SqlStore;

pub struct Processor {
    store: SqlStore,
}

impl Processor {
    pub fn new(store: SqlStore) -> Self {
        Self { store }
    }

    // TODO: Handle edits
    pub async fn add_message(&self, message: message::Message) {
        self.store.add_message(message).await;
    }

    pub async fn add_container(&self, container: container::Container) {
        self.store.add_container(container).await;
    }
}
