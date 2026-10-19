use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

use reqwest::dns::{Addrs, Name, Resolve, Resolving};
use tower_layer::Layer;
use tower_service::Service;

/// Connection phases observed while a request future is polled.
#[derive(Debug, Default, Clone, Copy)]
pub struct Phases {
    pub dns: Option<Duration>,
    /// Full connector time, including DNS, TCP and TLS.
    pub connect: Option<Duration>,
}

pub type Recorder = Arc<Mutex<Phases>>;

tokio::task_local! {
    static PHASES: Recorder;
}

/// Runs `future` with a recorder that the resolver and connector write into.
pub async fn measure<F: Future>(recorder: Recorder, future: F) -> F::Output {
    PHASES.scope(recorder, future).await
}

fn record(update: impl FnOnce(&mut Phases)) {
    // Outside a measured request (or in a background connect) there is nothing to record.
    let _ = PHASES.try_with(|phases| {
        if let Ok(mut phases) = phases.lock() {
            update(&mut phases);
        }
    });
}

/// System DNS resolver that records lookup time.
pub struct TimingResolver;

impl Resolve for TimingResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let host = name.as_str().to_string();
        Box::pin(async move {
            let start = Instant::now();
            let addrs = tokio::net::lookup_host((host.as_str(), 0)).await?;
            record(|p| p.dns = Some(start.elapsed()));
            Ok(Box::new(addrs.collect::<Vec<_>>().into_iter()) as Addrs)
        })
    }
}

/// Connector layer that records how long establishing a connection takes.
#[derive(Clone)]
pub struct TimingLayer;

impl<S> Layer<S> for TimingLayer {
    type Service = TimingService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TimingService { inner }
    }
}

#[derive(Clone)]
pub struct TimingService<S> {
    inner: S,
}

impl<S, R> Service<R> for TimingService<S>
where
    S: Service<R>,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<S::Response, S::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: R) -> Self::Future {
        let future = self.inner.call(request);
        Box::pin(async move {
            let start = Instant::now();
            let result = future.await;
            record(|p| p.connect = Some(start.elapsed()));
            result
        })
    }
}
