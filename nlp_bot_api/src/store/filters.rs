use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum Order {
    Ascending,
    Descending,
}

impl FromStr for Order {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "asc" => Ok(Self::Ascending),
            "desc" => Ok(Self::Descending),
            _ => Err(String::from("Invalid order")),
        }
    }
}

#[derive(Debug)]
pub struct NgramsByCountFilter {
    pub sender_id: Option<String>,
    pub length: Option<u32>,
    pub container_ids: Vec<String>,
    pub limit: u32,
    pub order: Order,
}

impl Default for NgramsByCountFilter {
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

#[derive(Debug)]
pub struct NgramsByContentFilter {
    pub content: String,
    pub sender_id: Option<String>,
    pub container_ids: Vec<String>,
}
