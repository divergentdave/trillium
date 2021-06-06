use crate::Url;
use async_io::Timer;
use std::{future::Future, time::Duration};
use trillium::Handler;

/**
Starts an trillium handler using the smol server bound to a random
available port on localhost, run the async tests provided as the
second argument, and then shut down the server. useful for full
integration tests that actually exercise the tcp layer.

See
[`trillium_client::Conn`](https://docs.trillium.rs/trillium_client/struct.conn)
for usage examples.

stability note: this doesn't really feel like it fits in the testing
crate, as it would not work well with a tokio-specific handler. it may
go away entirely at some point, or be moved to the trillium_smol crate
*/
pub fn with_server<H, Fun, Fut>(handler: H, tests: Fun)
where
    H: Handler,
    Fun: Fn(Url) -> Fut,
    Fut: Future<Output = Result<(), Box<dyn std::error::Error>>>,
{
    async_global_executor::block_on(async move {
        let port = portpicker::pick_unused_port().expect("could not pick a port");
        let url = format!("http://localhost:{}", port).parse().unwrap();
        let stopper = trillium_smol::Stopper::new();

        let server_future = async_global_executor::spawn(
            trillium_smol::config()
                .with_host("localhost")
                .with_port(port)
                .with_stopper(stopper.clone())
                .run_async(handler),
        );

        Timer::after(Duration::from_millis(500)).await;
        let result = tests(url).await;
        stopper.stop();
        server_future.await;
        result.unwrap()
    })
}