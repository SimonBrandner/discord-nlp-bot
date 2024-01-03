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
}
