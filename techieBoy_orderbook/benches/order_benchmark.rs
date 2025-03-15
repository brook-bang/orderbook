use core::time::Duration;
use criterion::{Criterion, SamplingMode, criterion_group, criterion_main};
use orderbooklib::{OrderBook, Side};
use rand::{Rng, rng};
use rand_distr::{Distribution, Normal};

fn initialize_orderbook(
    num_orders: i32,
    rng: &mut rand::prelude::ThreadRng,
    normal: Normal<f64>,
) -> OrderBook {
    let mut ob = OrderBook::new("Random".to_string());
    for _ in 0..num_orders {
        if rng.random_bool(0.5) {
            ob.add_limit_order(
                Side::Bid,
                normal.sample(rng) as u64,
                rng.random_range(1..=500),
            );
        } else {
            ob.add_limit_order(
                Side::Ask,
                normal.sample(rng) as u64,
                rng.random_range(1..=500),
            );
        }
    }
    ob
}

fn match_orders(ob: &mut OrderBook, rng: &mut rand::prelude::ThreadRng, normal: Normal<f64>) {
    for _ in 0..10000 {
        let _fr = ob.add_limit_order(
            Side::Ask,
            normal.sample(rng) as u64,
            rng.random_range(1..=500),
        );
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = rng();
    let normal = Normal::new(5000.0, 500.0).unwrap();
    let mut ob = initialize_orderbook(100_000, &mut rng, normal);
    let mut group = c.benchmark_group("order-benchmark");

    group.sample_size(10);
    group.measurement_time(Duration::new(20, 0));
    group.bench_function("Init 100k orders", |b| {
        b.iter(|| initialize_orderbook(1_000_00, &mut rng, normal));
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);