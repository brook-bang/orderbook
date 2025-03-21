mod engine;
mod errors;
mod notifications;
mod orderbook;
mod tui;

pub use engine::{MatchingEngine, TradingPair};
pub use errors::Result;
pub use notifications::{Notification, NotificationHandler};

pub use orderbook::{
    HalfBook, OrderBook, OrderBookState, OrderId, OrderRequest, OrderResult, OrderStatus,
    OrderType, Price, Quantity, Side, TradeExecution, TradeOrder,
};

use tracing_subscriber::fmt::format::FmtSpan;
pub use tui::App;
