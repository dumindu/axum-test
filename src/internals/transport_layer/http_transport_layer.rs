use ::anyhow::Result;
use ::axum::body::Body;
use ::bytes::Bytes;
use ::http::response::Parts;
use ::http::Request;
use ::http_body_util::BodyExt;
use ::hyper_util::client::legacy::Client;
use ::reserve_port::ReservedPort;
use ::std::future::Future;
use ::std::pin::Pin;
use ::tokio::task::JoinHandle;
use ::url::Url;

use crate::transport_layer::TransportLayer;

#[derive(Debug)]
pub struct HttpTransportLayer {
    server_handle: JoinHandle<()>,

    /// If this has reserved a port for the test,
    /// then it is stored here.
    ///
    /// It's stored here until we `Drop` (as it's reserved).
    #[allow(dead_code)]
    maybe_reserved_port: Option<ReservedPort>,

    url: Url,
}

impl HttpTransportLayer {
    pub(crate) fn new(
        server_handle: JoinHandle<()>,
        maybe_reserved_port: Option<ReservedPort>,
        url: Url,
    ) -> Self {
        Self {
            server_handle,
            maybe_reserved_port,
            url,
        }
    }
}

impl TransportLayer for HttpTransportLayer {
    fn send<'a>(
        &'a self,
        request: Request<Body>,
    ) -> Pin<Box<dyn 'a + Future<Output = Result<(Parts, Bytes)>>>> {
        Box::pin(async {
            let client = Client::builder(hyper_util::rt::TokioExecutor::new()).build_http();
            let hyper_response = client.request(request).await?;

            let (parts, response_body) = hyper_response.into_parts();
            let response_bytes = response_body.collect().await?.to_bytes();

            Ok((parts, response_bytes))
        })
    }

    fn url<'a>(&'a self) -> Option<&'a Url> {
        Some(&self.url)
    }
}

impl Drop for HttpTransportLayer {
    fn drop(&mut self) {
        self.server_handle.abort()
    }
}
