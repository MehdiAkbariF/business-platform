use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::task::JoinSet;

#[tokio::test]
#[ignore = "Run explicitly for stress testing"]
async fn stress_test_concurrent_search_requests() {
    let client = reqwest::Client::builder()
        .pool_max_idle_per_host(200)
        .build()
        .unwrap();

    let total_requests = 5_000;
    let concurrency = 50;

    let successful = Arc::new(AtomicUsize::new(0));
    let failed = Arc::new(AtomicUsize::new(0));

    println!("\n🚀 Starting Stress Test: {} requests with concurrency {}...", total_requests, concurrency);
    let start = Instant::now();

    let mut set = JoinSet::new();
    let requests_per_worker = total_requests / concurrency;

    for _ in 0..concurrency {
        let client = client.clone();
        let successful = successful.clone();
        let failed = failed.clone();

        set.spawn(async move {
            for _ in 0..requests_per_worker {
                let res = client.get("http://127.0.0.1:8080/api/v1/search?q=کافه").send().await;
                match res {
                    Ok(resp) if resp.status().is_success() => {
                        successful.fetch_add(1, Ordering::Relaxed);
                    }
                    _ => {
                        failed.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        });
    }

    while let Some(res) = set.join_next().await {
        res.unwrap();
    }

    let elapsed = start.elapsed();
    let total_ok = successful.load(Ordering::Relaxed);
    let total_err = failed.load(Ordering::Relaxed);
    let rps = total_ok as f64 / elapsed.as_secs_f64();

    println!("\n================ STRESS TEST RESULTS ================");
    println!("⏱️  Total Duration: {:.2?}", elapsed);
    println!("✅ Successful Requests: {}", total_ok);
    println!("❌ Failed Requests: {}", total_err);
    println!("⚡ Throughput (RPS): {:.2} req/sec", rps);
    println!("====================================================\n");

    assert!(total_ok > 0);
    assert_eq!(total_err, 0);
}