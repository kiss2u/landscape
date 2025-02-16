#[derive(Debug, Clone)]
pub enum IfaceObserverAction {
    Up(String),
    Down(String),
}
