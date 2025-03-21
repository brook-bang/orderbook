#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Side {
    Ask,
    Bid,
}

impl Side {
    pub fn opposite(&self) -> Side {
        match self {
            Side::Ask => Side::Bid,
            Side::Bid => Side::Ask,
        }
    }
}