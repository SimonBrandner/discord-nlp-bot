pub enum Order {
    Ascending,
    Descending,
}

pub struct MostUsedNgramFilter {
    pub sender_id: Option<String>,
    pub length: Option<u32>,
    pub container_ids: Vec<String>,
    pub limit: u32,
    pub order: Order,
}

impl Default for MostUsedNgramFilter {
    fn default() -> Self {
        Self {
            sender_id: None,
            length: None,
            container_ids: Vec::new(),
            limit: 10,
            order: Order::Descending,
        }
    }
}
