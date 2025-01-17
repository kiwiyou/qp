use super::loop_factorial20;
use async_trait::async_trait;
use criterion::Bencher;
use futures::prelude::*;
use qp::pool::Pool;
use qp::resource::Factory;
use std::convert::Infallible;
use std::time::Instant;
use tokio::runtime::Runtime;

pub struct IntFactory;

#[async_trait]
impl Factory for IntFactory {
    type Output = i32;
    type Error = Infallible;

    async fn try_create(&self) -> Result<Self::Output, Self::Error> {
        Ok(0)
    }

    async fn validate(&self, resource: &Self::Output) -> bool {
        resource >= &0
    }
}

pub fn bench_with_input(bencher: &mut Bencher, input: &(usize, usize)) {
    bencher
        .to_async(Runtime::new().unwrap())
        .iter_custom(|iters| async move {
            let pool = Pool::new(IntFactory, input.0);
            drop(future::join_all((0..input.0).map(|_| pool.acquire())).await);
            let start = Instant::now();
            for _ in 0..iters {
                let handles = (0..input.1)
                    .map(|_| {
                        let pool = pool.clone();
                        tokio::spawn(async move {
                            let int = pool.acquire().await.unwrap();
                            loop_factorial20();
                            criterion::black_box(*int);
                        })
                    })
                    .collect::<Vec<_>>();
                for handle in handles {
                    handle.await.unwrap();
                }
            }
            start.elapsed()
        })
}
