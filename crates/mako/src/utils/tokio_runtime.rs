use std::future::Future;
use std::sync::OnceLock;

use tokio;

static TOKIO_RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

fn build_tokio_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .worker_threads(2)
        .thread_name("Mako-tokio-worker")
        .build()
        .expect("Mako: failed to create tokio runtime.")
}

pub fn spawn<F>(future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    TOKIO_RUNTIME.get_or_init(build_tokio_runtime).spawn(future)
}

#[allow(dead_code)]
pub fn block_on<F: Future>(future: F) -> F::Output {
    TOKIO_RUNTIME
        .get_or_init(build_tokio_runtime)
        .block_on(future)
}
