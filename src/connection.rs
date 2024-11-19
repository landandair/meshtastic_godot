#[derive(Debug, Clone, Default)]
pub enum Connection {
    TCP(String, u16),
    Serial(String),
    #[default]
    None,
}