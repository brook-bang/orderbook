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
                Side::Buy => order.price>=other_top.price,
            };
            if crossed {
                
            }
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
}
