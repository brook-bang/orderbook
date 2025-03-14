use std::{
    cmp::Ordering,
    collections::{BTreeSet, HashMap},
    fmt::Display,
    usize,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    pub fn new(side: char) -> Option<Side> {
        match side {
            'B' => Some(Side::Buy),
            'S' => Some(Side::Sell),
            _ => None,
        }
    }
}

impl std::ops::Not for Side {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Buy => Self::Sell,
            Self::Sell => Self::Buy,
        }
    }
}

impl From<char> for Side {
    fn from(side: char) -> Self {
        Self::new(side).unwrap()
    }
}

impl Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ch = match self {
            Side::Buy => 'B',
            Side::Sell => 'S',
        };
        write!(f, "{}", ch)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Order {
    pub user_id: usize,
    pub order_id: usize,
    pub price: usize,
    pub volume: usize,
    pub side: Side,
}

impl Order {
    pub fn new(side: Side, user_id: usize, order_id: usize, price: usize, volume: usize) -> Order {
        Order {
            user_id,
            order_id,
            price,
            volume,
            side,
        }
    }

    fn price_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.price != other.price {
            self.price.partial_cmp(&other.price)
        } else if self.volume != other.volume {
            self.volume.partial_cmp(&other.volume)
        } else {
            match self.side {
                Side::Buy => other.order_id.partial_cmp(&self.order_id),
                Side::Sell => self.order_id.partial_cmp(&other.order_id),
            }
        }
    }
}

impl PartialOrd for Order {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.side {
            Side::Buy => match other.side {
                Side::Buy => self.price_cmp(other),
                Side::Sell => Some(Ordering::Greater),
            },
            Side::Sell => match other.side {
                Side::Buy => Some(Ordering::Less),
                Side::Sell => self.price_cmp(other),
            },
        }
    }
}

impl Ord for Order {
    fn cmp(&self, other: &Self) -> Ordering {
        self.price_cmp(other).unwrap()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogEntry {
    Acknowledge {
        user_id: usize,
        order_id: usize,
    },
    Reject {
        user_id: usize,
        order_id: usize,
    },
    TopOfBook {
        side: Option<Side>,
        price: usize,
        volume: usize,
    },
    SideElimination(Side),
    Trade {
        user_id_buy: usize,
        order_id_buy: usize,
        user_id_sell: usize,
        order_id_sell: usize,
        price: usize,
        volume: usize,
    },
}

struct OrderBookEntry {
    pub orders: BTreeSet<Order>,
    pub log: Vec<LogEntry>,
}

impl OrderBookEntry {
    pub fn new() -> OrderBookEntry {
        OrderBookEntry {
            orders: BTreeSet::new(),
            log: Vec::new(),
        }
    }
}

pub struct OrderBook {
    order_book: HashMap<String, OrderBookEntry>,
    index: HashMap<(usize, usize), (String, Order)>,
}

impl OrderBook {
    pub fn new() -> OrderBook {
        OrderBook {
            order_book: HashMap::new(),
            index: HashMap::new(),
        }
    }

    pub fn add(&mut self, symbol: &str, order: &Order) {
        let top = self.top(order.side, symbol);
        let other_top = self.top(!order.side, symbol);
        let order_book = self
            .order_book
            .entry(symbol.to_owned())
            .or_insert(OrderBookEntry::new());

        if top.is_some() && other_top.is_some() {
            let top = top.unwrap();
            let other_top = other_top.unwrap();
            let crossed = match top.side {
                Side::Sell => other_top.price >= order.price,
                Side::Buy => order.price >= other_top.price,
            };
            if crossed {
                order_book.log.push(LogEntry::Reject {
                    user_id: order.user_id,
                    order_id: order.order_id,
                });
                return;
            }
        }

        order_book.log.push(LogEntry::Acknowledge {
            user_id: order.user_id,
            order_id: order.order_id,
        });

        self.index
            .insert((order.user_id, order.order_id), (symbol.to_owned(), *order));
        order_book.orders.insert(*order);
        let new_top = self.top(order.side, symbol);
        self.log_top_of_book(symbol, top, new_top);
    }

    pub fn cancel(&mut self, user_id: usize, order_id: usize) {
        match self.index.remove(&(user_id, order_id)) {
            Some((symbol, order)) => {
                let old_top = self.top(order.side, &symbol);
                let order_book = self.order_book.get_mut(&symbol).unwrap();
                order_book.orders.remove(&order);
                order_book
                    .log
                    .push(LogEntry::Acknowledge { user_id, order_id });
                let new_top = self.top(order.side, &symbol);
                self.log_top_of_book(&symbol, old_top, new_top);
            }
            None => (),
        }
    }

    pub fn top(&self, side: Side, symbol: &str) -> Option<Order> {
        match self.order_book.get(symbol) {
            None => None,
            Some(ref order_entry) => {
                let order = match side {
                    Side::Buy => order_entry.orders.last(),
                    Side::Sell => order_entry.orders.first(),
                };
                match order {
                    None => None,
                    Some(o) => {
                        let mut o = *o;
                        if o.side == side {
                            (o.volume, o.order_id) = match side {
                                Side::Sell => {
                                    self.total_volume(order_entry.orders.iter(), o.user_id, o.price)
                                }
                                Side::Buy => self.total_volume(
                                    order_entry.orders.iter().rev(),
                                    o.user_id,
                                    o.price,
                                ),
                            };
                            Some(o)
                        } else {
                            None
                        }
                    }
                }
            }
        }
    }

    pub fn get_logs(&self, symbol: &str) -> Option<&Vec<LogEntry>> {
        match self.order_book.get(symbol) {
            None => None,
            Some(order_entry) => Some(&order_entry.log),
        }
    }

    fn total_volume<'a>(
        &self,
        it: impl Iterator<Item = &'a Order>,
        user_id: usize,
        price: usize,
    ) -> (usize, usize) {
        let mut min_order_id = usize::MAX;
        let total = it
            .take_while(|x| x.user_id == user_id && x.price == price)
            .fold(0, |acc, x| {
                if x.order_id < min_order_id {
                    min_order_id = x.order_id;
                }
                acc + x.volume
            });
        (total, min_order_id)
    }

    fn log_top_of_book(&mut self, symbol: &str, old_top: Option<Order>, new_top: Option<Order>) {
        let order_book = self.order_book.get_mut(symbol).unwrap();
        if new_top.is_none() {
            order_book.log.push(LogEntry::TopOfBook {
                side: None,
                price: 0,
                volume: 0,
            });
        } else if old_top.is_none() || old_top.unwrap() != new_top.unwrap() {
            let order = new_top.unwrap();
            order_book.log.push(LogEntry::TopOfBook {
                side: Some(order.side),
                price: order.price,
                volume: order.volume,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_side() {
        assert_eq!(Side::new('B'), Some(Side::Buy));
        assert_eq!(Side::new('S'), Some(Side::Sell));
        assert_eq!(Side::new('T'), None);
        assert_eq!(Side::from('B'), Side::Buy);
        assert_eq!(Side::from('S'), Side::Sell);
        assert_eq!(format!("{}", Side::Buy), "B".to_owned());
        assert_eq!(format!("{}", Side::Sell), "S".to_owned());
    }

    #[test]
    #[should_panic]
    fn test_side_invalid() {
        let _ = Side::from('T');
    }

    #[test]
    fn test_order_compare() {
        assert!(Order::new(Side::Sell, 0, 0, 0, 0) < Order::new(Side::Buy, 0, 0, 0, 0));
        assert!(Order::new(Side::Buy, 0, 0, 0, 0) > Order::new(Side::Sell, 0, 0, 0, 0));
        assert!(Order::new(Side::Buy, 0, 0, 100, 20) > Order::new(Side::Buy, 0, 1, 100, 20));
        assert!(Order::new(Side::Sell, 0, 0, 100, 20) < Order::new(Side::Sell, 0, 1, 100, 20));
        assert!(Order::new(Side::Buy, 0, 0, 100, 10) < Order::new(Side::Buy, 0, 0, 200, 5));
        assert!(Order::new(Side::Buy, 0, 0, 200, 5) > Order::new(Side::Buy, 0, 0, 100, 10));
    }
}
