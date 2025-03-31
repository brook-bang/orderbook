use orderbooklib::{OrderBook, OrderRequest, OrderStatus, OrderType, Side};
use polars::{frame::row::Row, prelude::*};
use std::time::Instant;
use tracing::debug;

fn main() -> Result<(),Box<dyn std::error::Error>>{
    let mut order_book = OrderBook::default();
    let schema = Schema::from_iter(vec![
        Field::new("Time".into(), DataType::Float64),
        Field::new(name, dtype)
    ])
}
