use rust_decimal::Decimal;

use tracing::{info, warn};

use super::{orders::*, price_levels};
use super::price_levels::SparseVec;
use super::types::*;

use std::{collections::{BTreeSet, HashMap, VecDeque}, net::Incoming};

#[derive(Debug)]
pub struct HalfBook {
    s: Side,
    price_set: BTreeSet<Price>,
    price_levels: SparseVec<Price, PriceLevel>,
}

impl HalfBook {
    pub fn new(s: Side) -> HalfBook {
        HalfBook {
            s,
            price_set: BTreeSet::new(),
            price_levels: SparseVec::with_capacity(10_000),
        }
    }

    pub fn add_order(&mut self, price: impl Into<Price>, order: TradeOrder) {
        let price = price.into();
        if let Some(level) = self.price_levels.get_mut(&price) {
            level.push_back(order);
        } else {
            self.price_set.insert(price);
            self.price_levels.insert(price, VecDeque::from(vec![order]));
        }
    }

    pub fn remove_order(&mut self, price: &Price, order_id: OrderId) -> Option<TradeOrder> {
        let level = self.price_levels.get_mut(price)?;
        let removed_order = level
            .iter()
            .position(|o| o.id == order_id)
            .map(|i| level.remove(i))?;
        if level.is_empty() {
            self.price_levels.remove(price);
            self.price_set.remove(price);
        }
        removed_order
    }

    pub fn match_order(
        &mut self,
        incoming_order: &mut TradeOrder,
        price: impl Into<Price>,
    ) -> Vec<TradeExecution> {
        let price = price.into();
        let mut executions = Vec::new();
        if let Some(price_level) = self.price_levels.get_mut(&price) {
            while !price_level.is_empty() && incoming_order.remaining_qty > Decimal::ZERO {
                
            }
        }
    }
}
