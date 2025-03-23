use dashmap::iter;
use rust_decimal::Decimal;

use tracing::{info, warn};

use super::price_levels::SparseVec;
use super::types::*;
use super::{orders::*, price_levels};

use std::{
    collections::{BTreeSet, HashMap, VecDeque},
    net::Incoming,
};

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
                if let Some(mut existing_order) = price_level.pop_front() {
                    let fill_qty = existing_order.filled_by(incoming_order, price);
                    executions.push(TradeExecution::new(
                        fill_qty,
                        price,
                        incoming_order,
                        &existing_order,
                        self.s.opposite(),
                    ));
                    if existing_order.remaining_qty > Decimal::ZERO {
                        price_level.push_front(existing_order);
                    }
                }
            }
            if price_level.is_empty() {
                self.price_levels.remove(&price);
                self.price_set.remove(&price);
            }
        }
        executions
    }

    pub fn best_price(&self) -> Option<Price> {
        match self.s {
            Side::Ask => self.price_levels.min_index(),
            Side::Bid => self.price_levels.max_index(),
        }
    }

    pub fn get_price_level(&self, price: &Price) -> Option<&PriceLevel> {
        self.price_levels.get(price)
    }

    pub fn iter_prices(&self) -> impl Iterator<Item = Price> {
        match self.s {
            Side::Ask => self
                .price_set
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .into_iter(),
            Side::Bid => self
                .price_set
                .iter()
                .rev()
                .cloned()
                .collect::<Vec<_>>()
                .into_iter(),
        }
    }

    pub fn show_depth(&self) {
        let prices: Vec<_> = match self.s {
            Side::Ask => self.price_set.iter().rev().cloned().collect(),
            Side::Bid => self.price_set.iter().rev().cloned().collect(),
        };
        self.print_price_levels(prices.iter());
    }

    fn print_price_levels<'a, I>(&self, prices: I)
    where
        I: Iterator<Item = &'a Price>,
    {
        for price in prices {
            let level = self.get_price_level(price).unwrap();
            println!(
                "Price:{} Qty: {}",
                price,
                level
                    .iter()
                    .fold(Decimal::ZERO, |acc, o| acc + o.remaining_qty)
            );
        }
    }

    pub fn get_total_qty(&self, price: &Price) -> Option<Price> {
        Some(
            self.price_levels
                .get(price)?
                .iter()
                .fold(Decimal::ZERO, |acc, o| acc + o.remaining_qty),
        )
    }

    pub fn get_available_quantity(&self, target_price: impl Into<Price>) -> Quantity {
        let target_price = target_price.into();
        self.iter_prices()
            .take_while(|&p| match self.s {
                Side::Ask => p <= target_price,
                Side::Bid => p >= target_price,
            })
            .map(|p| self.get_total_qty(&p).unwrap_or(Decimal::ZERO))
            .sum()
    }

    pub fn get_levels(&self) -> Vec<(Price, Quantity)> {
        self.iter_prices()
            .map(|price| (price, self.get_total_qty(&price).unwrap_or(Decimal::ZERO)))
            .collect()
    }

    pub fn get_total_volume(&self) -> Quantity {
        self.iter_prices()
            .map(|price| self.get_total_qty(&price).unwrap_or(Decimal::ZERO))
            .sum()
    }

    pub fn get_depth(&self) -> usize {
        self.price_set.len()
    }

    pub fn get_price_range(&self) -> Option<Price> {
        if self.price_set.is_empty() {
            return None;
        }

        let min = *self.price_set.iter().next()?;
        let max = *self.price_set.iter().next_back()?;
        Some(max - min)
    }

    pub fn get_orders_at_price(&self, price: impl Into<Price>) -> Option<Vec<&TradeOrder>> {
        let price = price.into();
        self.price_levels
            .get(&price)
            .map(|level| level.iter().collect())
    }

    pub fn is_empty(&self) -> bool {
        self.price_set.is_empty()
    }

    pub fn get_order(&self, price: impl Into<Price>, order_id: OrderId) -> Option<&TradeOrder> {
        let price = price.into();
        self.price_levels
            .get(&price)
            .and_then(|level| level.iter().find(|o| o.id == order_id))
    }

    pub fn get_order_mut(&mut self, price: &Price, order_id: &OrderId) -> Option<&mut TradeOrder> {
        self.price_levels
            .get_mut(price)
            .and_then(|level| level.iter_mut().find(|o| o.id == *order_id))
    }

    pub fn get_order_count(&self) -> usize {
        self.price_levels.iter().map(|(_, level)| level.len()).sum()
    }

    pub fn clear(&mut self) {
        self.price_set.clear();
        self.price_levels = SparseVec::with_capacity(10_000);
    }
}

#[derive(Debug)]
pub struct OrderBookState {
    pub asks: Vec<(Price, Quantity)>,
    pub bids: Vec<(Price, Quantity)>,
}

#[derive(Debug)]
pub struct OrderBook {
    pub asks: HalfBook,
    pub bids: HalfBook,
    pub order_loc: HashMap<OrderId, (Side, Price)>,
}

impl Default for OrderBook {
    fn default() -> Self {
        Self {
            asks: HalfBook::new(Side::Ask),
            bids: HalfBook::new(Side::Bid),
            order_loc: HashMap::with_capacity(10_000),
        }
    }
}

impl OrderBook {
    fn get_mut_opposite_book(&mut self, side: &Side) -> &mut HalfBook {
        match side {
            Side::Ask => &mut self.bids,
            Side::Bid => &mut self.asks,
        }
    }

    pub fn show_depth(&self) {
        println!("Asks:");
        self.asks.show_depth();
        println!("Bids:");
        self.bids.show_depth();
    }

    pub fn best_price_liq(&self) -> Option<()> {
        println!("Best Bid Price: {}", self.best_bid()?);
        println!(
            "Bid price quantity: {}",
            self.bids.get_total_qty(&self.best_bid()?)?
        );
        println!("Best Ask Price:{}", self.best_ask()?);
        println!(
            "Ask price quantity: {}",
            self.asks.get_total_qty(&self.best_ask()?)?
        );
        println!(
            "Spread: {}",
            ((self.best_ask()? - self.best_bid()?) / self.best_ask()?)
        );
        Some(())
    }

    pub fn best_bid(&self) -> Option<Price> {
        self.bids.best_price()
    }

    pub fn best_ask(&self) -> Option<Price> {
        self.asks.best_price()
    }

    pub fn best_prices(&self) -> (Option<Price>, Option<Price>) {
        (self.bids.best_price(),self.asks.best_price())
    }

    pub fn delete_order(&mut self,order_id: OrderId) -> Option<OrderResult> {
        let (side,price) =self.order_loc.remove(&order_id)?;
        
    
    }


}
