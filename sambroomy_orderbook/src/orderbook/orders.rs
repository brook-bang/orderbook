use std::fmt::{Display, write};

use log::warn;
use rust_decimal::Decimal;

use super::types::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrderType {
    Market,
    Limit(Price),
    IOC(Price),
    FOK(Price),
    SystemLevel(Price),
}

impl OrderType {
    pub fn limit(price: impl Into<Price>) -> Self {
        OrderType::Limit(price.into())
    }

    pub fn ioc(price: impl Into<Price>) -> Self {
        OrderType::IOC(price.into())
    }

    pub fn fok(price: impl Into<Price>) -> Self {
        OrderType::FOK(price.into())
    }

    pub fn system_level(price: impl Into<Price>) -> Self {
        OrderType::SystemLevel(price.into())
    }

    pub fn generate_id(&self) -> OrderId {
        match self {
            OrderType::Market => create_order_id(),
            OrderType::Limit(_) => create_order_id(),
            OrderType::IOC(_) => create_order_id(),
            OrderType::FOK(_) => create_order_id(),
            OrderType::SystemLevel(p) => create_id_from_bytes(p.to_string().as_bytes()),
        }
    }

    pub fn price(&self) -> Option<Price> {
        match self {
            OrderType::Market => None,
            OrderType::Limit(price) => Some(*price),
            OrderType::IOC(price) => Some(*price),
            OrderType::FOK(price) => Some(*price),
            OrderType::SystemLevel(price) => Some(*price),
        }
    }
}

impl Display for OrderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderType::Market => write!(f, "Matket"),
            OrderType::Limit(_) => write!(f, "Limit"),
            OrderType::IOC(_) => write!(f, "IOC"),
            OrderType::FOK(_) => write!(f, "FOK"),
            OrderType::SystemLevel(_) => write!(f, "SystemLevel"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrderStatus {
    Open,
    Filled,
    PartiallyFilled,
    Cancelled,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Fill {
    pub qty: Quantity,
    pub price: Price,
    pub timestamp: Timestamp,
    pub order_id: OrderId,
}

impl Fill {
    pub fn new(qty: Quantity, price: Price, order_id: OrderId) -> Self {
        Self {
            qty,
            price,
            timestamp: timestamp(),
            order_id,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct OrderRequest {
    id: OrderId,
    pub side: Side,
    pub qty: Quantity,
    pub order_type: OrderType,
}

impl OrderRequest {
    pub fn new(side: Side, qty: impl Into<Quantity>, order_type: OrderType) -> Self {
        let id = order_type.generate_id();
        Self {
            id,
            side,
            qty: qty.into(),
            order_type,
        }
    }

    pub fn new_with_id(
        id: OrderId,
        side: Side,
        qty: impl Into<Quantity>,
        order_type: OrderType,
    ) -> Self {
        Self {
            id,
            side,
            qty: qty.into(),
            order_type,
        }
    }

    pub fn new_with_other_id(
        id: impl AsRef<[u8]>,
        side: Side,
        qty: impl Into<Quantity>,
        order_type: OrderType,
    ) -> Self {
        Self {
            id: create_id_from_bytes(id),
            side,
            qty: qty.into(),
            order_type,
        }
    }

    pub fn price(&self) -> Option<Price> {
        self.order_type.price()
    }

    pub fn id(&self) -> OrderId {
        self.id
    }
}

pub struct TradeOrder {
    pub id: OrderId,
    pub side: Side,
    pub remaining_qty: Quantity,
    initial_qty: Quantity,
    fills: Vec<Fill>,
    pub order_type: OrderType,
    creation_timestamp: Timestamp,
    last_modified_timestamp: Timestamp,
}

impl From<OrderRequest> for TradeOrder {
    fn from(order_request: OrderRequest) -> Self {
        let ts = timestamp();
        Self {
            id: order_request.id,
            side: order_request.side,
            remaining_qty: order_request.qty,
            initial_qty: order_request.qty,
            fills: Vec::new(),
            order_type: order_request.order_type,
            creation_timestamp: ts,
            last_modified_timestamp: ts,
        }
    }
}

impl TradeOrder {
    pub fn new(qty: impl Into<Quantity>) -> Self {
        let qty = qty.into();
        let ts = timestamp();
        Self {
            id: create_order_id(),
            side: Side::Ask,
            remaining_qty: qty,
            initial_qty: qty,
            fills: Vec::new(),
            order_type: OrderType::Market,
            creation_timestamp: ts,
            last_modified_timestamp: ts,
        }
    }

    pub fn fill(&mut self, qty: &mut Quantity, price: impl Into<Price>, order_id: OrderId) {
        let price = price.into();
        let fill_qty = (*qty).min(self.remaining_qty);
        self.remaining_qty -= fill_qty;
        self.fills.push(Fill::new(fill_qty, price, order_id));
        *qty -= fill_qty;
        self.last_modified_timestamp = timestamp();
    }

    pub fn filled_by(&mut self, other: &mut TradeOrder, price: impl Into<Price>) -> Quantity {
        let price = price.into();
        let fill_qty = other.remaining_qty.min(self.remaining_qty);
        self.remaining_qty -= fill_qty;
        other.remaining_qty -= fill_qty;
        self.fills.push(Fill::new(fill_qty, price, other.id));
        other.fills.push(Fill::new(fill_qty, price, self.id));
        self.last_modified_timestamp = timestamp();
        fill_qty
    }

    pub fn filled_quantity(&self) -> Quantity {
        self.initial_qty - self.remaining_qty
    }

    pub fn cancel(&mut self,qty: impl Into<Quantity>) {
        let qty = qty.into();
        let qty = qty.min(self.remaining_qty);
        self.remaining_qty -= qty
    }

    pub fn mergable(&self,other: &mut TradeOrder) -> bool {
        self.side == other.side && self.order_type == other.order_type
    }

    pub fn merage(&mut self,mut other: TradeOrder) -> Option<Self> {
        if !self.mergable(&mut other) {
            warn!("Cannot merge orders with different side or order type");
            return Some(other);
        }
        self.remaining_qty += other.remaining_qty;
        self.initial_qty += other.initial_qty;
        self.fills.append(&mut other.fills);
        self.last_modified_timestamp = timestamp();
        None
    }
}

#[allow(dead_code)]
#[derive(Debug,Clone)]
pub struct OrderResult {
    traid_id: OrderId,
    side: Side,
    order_type: OrderType,
    initial_qty: Quantity,
    pub remaining_qty: Quantity,
    filles: Vec<Fill>,
    pub status: OrderStatus,
}

impl From<TradeOrder> for OrderResult { 
    fn from(trade_order: TradeOrder) -> Self {
        OrderStatus::Filled
    }
    
}