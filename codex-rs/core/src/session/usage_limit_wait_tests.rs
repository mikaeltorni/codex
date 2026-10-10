use super::wait_for_deadline;
use chrono::DateTime;
use std::sync::Arc;
use std::sync::atomic::AtomicI64;
use std::sync::atomic::Ordering;
use std::time::Duration;

#[tokio::test(start_paused = true)]
async fn wall_clock_passing_deadline_resumes_without_waiting_for_monotonic_deadline() {
    let wall_clock = Arc::new(AtomicI64::new(/*v*/ 0));
    let now = Arc::clone(&wall_clock);
    let wait = tokio::spawn(async move {
        wait_for_deadline(/*retry_at_ms*/ 3_600_000, || {
            DateTime::from_timestamp_millis(now.load(Ordering::SeqCst)).unwrap()
        })
        .await;
    });
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(/*secs*/ 1)).await;
    tokio::task::yield_now().await;
    assert!(
        !wait.is_finished(),
        "the wall-clock deadline has not passed"
    );

    // Suspend advances wall time much further than the monotonic timer.
    wall_clock.store(/*val*/ 3_600_001, Ordering::SeqCst);
    tokio::time::advance(Duration::from_secs(/*secs*/ 1)).await;
    tokio::task::yield_now().await;
    assert!(
        wait.is_finished(),
        "retry after the absolute deadline passes"
    );
    wait.await.unwrap();
}
