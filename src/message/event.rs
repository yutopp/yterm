#[derive(Clone, Debug)]
pub enum Event {
    Testing,
    KeyInput(u64, char),
    WindowWrite(u64, String),
}
