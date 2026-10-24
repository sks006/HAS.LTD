// backend/api/src/middleware/argon2_gate.rs

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use crate::metrics::METRICS;

#[derive(Debug, thiserror::Error)]
pub enum GateError {
    #[error("Argon2 gate timeout waiting for permit")]
    Timeout,
    #[error("Worker task execution failed: {0}")]
    JoinError(#[from] tokio::task::JoinError),
    #[error("Semaphore closed")]
    AcquireError,
}

#[derive(Clone)]
pub struct Argon2Gate {
    pub semaphore: Arc<Semaphore>,
    pub timeout: Duration,
}

impl Argon2Gate {
    #[allow(dead_code)]
    pub fn new(permits: usize, timeout: Duration) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(permits)),
            timeout,
        }
    }

    pub fn from_semaphore(semaphore: Arc<Semaphore>, timeout: Duration) -> Self {
        Self { semaphore, timeout }
    }

    pub async fn run<F, T>(&self, f: F) -> Result<T, GateError>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        METRICS.gauge_inc("auth_argon2_queue_depth");

        let permit_res = tokio::time::timeout(self.timeout, self.semaphore.clone().acquire_owned()).await;
        let permit = match permit_res {
            Ok(Ok(permit)) => permit,
            Ok(Err(_)) => {
                METRICS.gauge_dec("auth_argon2_queue_depth");
                return Err(GateError::AcquireError);
            }
            Err(_) => {
                METRICS.gauge_dec("auth_argon2_queue_depth");
                METRICS.counter_inc("auth_argon2_queue_timeout_total");
                return Err(GateError::Timeout);
            }
        };

        let result = tokio::task::spawn_blocking(move || {
            let res = f();
            drop(permit);
            res
        })
        .await
        .map_err(GateError::JoinError);

        METRICS.gauge_dec("auth_argon2_queue_depth");
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Instant;

    #[tokio::test]
    async fn test_argon2_gate_concurrency_and_queuing() {
        let permits = 4;
        let gate = Argon2Gate::new(permits, Duration::from_millis(1500));
        let active_tasks = Arc::new(AtomicUsize::new(0));
        let max_observed_active = Arc::new(AtomicUsize::new(0));

        let mut handles = Vec::new();
        let start = Instant::now();

        // Spawn 8 concurrent tasks
        for _ in 0..8 {
            let gate_clone = gate.clone();
            let active_clone = active_tasks.clone();
            let max_clone = max_observed_active.clone();

            let handle = tokio::spawn(async move {
                gate_clone
                    .run(move || {
                        let current = active_clone.fetch_add(1, Ordering::SeqCst) + 1;
                        max_clone.fetch_max(current, Ordering::SeqCst);
                        std::thread::sleep(Duration::from_millis(100));
                        active_clone.fetch_sub(1, Ordering::SeqCst);
                        42
                    })
                    .await
            });
            handles.push(handle);
        }

        for handle in handles {
            let res = handle.await.unwrap();
            assert!(res.is_ok(), "Task must succeed within timeout");
            assert_eq!(res.unwrap(), 42);
        }

        let elapsed = start.elapsed();
        // Since only 4 permits run at a time for 100ms each, 8 tasks require at least ~200ms
        assert!(elapsed >= Duration::from_millis(180), "8 tasks with 4 permits must take at least 2 waves");
        assert!(
            max_observed_active.load(Ordering::SeqCst) <= permits,
            "Active concurrent workers must never exceed semaphore permit limit"
        );
    }

    #[tokio::test]
    async fn test_argon2_gate_timeout() {
        // 1 permit, 50ms timeout
        let gate = Argon2Gate::new(1, Duration::from_millis(50));

        let gate1 = gate.clone();
        let slow_task = tokio::spawn(async move {
            gate1
                .run(|| {
                    std::thread::sleep(Duration::from_millis(150));
                })
                .await
        });

        // Give slow_task a moment to acquire the permit
        tokio::time::sleep(Duration::from_millis(10)).await;

        let gate2 = gate.clone();
        let timed_out_task = gate2
            .run(|| {
                // Should not reach here
            })
            .await;

        match timed_out_task {
            Err(GateError::Timeout) => (),
            other => panic!("Expected GateError::Timeout, got {:?}", other),
        }

        slow_task.await.unwrap().unwrap();
    }
}
