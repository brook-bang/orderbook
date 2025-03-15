use orderbooklib::OrderBook;
use orderbooklib::Side;
use rand::{rng, Rng};

fn main() {
    println!("Creating new Orderbook");
    let mut ob = OrderBook::new("BTC".to_string());
    let mut rng = rng();
    
    for _ in 1..100000 {
        ob.add_limit_order(Side::Bid, rng.random_range(1..5000), rng.random_range(1..=500));
    }

    println!("Done adding orders, Starting to fill");

    for _ in 1..10 {
        for _ in 1..10000 {
            let fr = ob.add_limit_order(Side::Ask, rng.random_range(1..5000), rng.random_range(1..=500));
        }
    }
    println!("Done!");
    ob.get_bbo();

}
