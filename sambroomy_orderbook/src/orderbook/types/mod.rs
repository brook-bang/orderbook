use rust_decimal::Decimal;
use uuid::Uuid;

use super::TradeOrder;

pub type OrderId = uuid::Uuid;

pub type PriceLevel = std::collections::VecDeque<TradeOrder>;
pub type Timestamp = std::time::SystemTime;

mod side;

pub type Price = Decimal;
pub type Quantity = Decimal;

pub use side::Side;

pub fn timestamp() -> Timestamp {
    std::time::SystemTime::now()
}

pub fn create_order_id() -> OrderId {
    uuid::Uuid::now_v7()
}

pub fn create_id_from_bytes(bytes: impl AsRef<[u8]>) -> OrderId {
    Uuid::new_v5(&Uuid::NAMESPACE_DNS, bytes.as_ref())
}