use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use ff_ipc::{JsonRpcCodec, JsonRpcError, JsonRpcMessage, JsonRpcRequest};
use futures_util::{SinkExt, StreamExt};
use tokio::net::UnixListener;
use tokio::sync::Mutex;
use tokio_util::codec::Framed;
use tracing::{info, warn};

use crate::lifecycle::DaemonLifecycle;
use ff_query::{QueryDispatcher, QueryParams, QueryType, StubDispatcher, parse_query_params};

#[allow(dead_code)]
pub struct SocketServer<D: QueryDispatcher> {
    listener: UnixListener,
    dispatcher: Arc<D>,
    lifecycle: Arc<Mutex<DaemonLifecycle>>,
    query_timeout: Duration,
}

impl<D: QueryDispatcher + 'static> SocketServer<D> {
    #[allow(dead_code)]
    pub fn new(
        listener: UnixListener,
        dispatcher: D,
        lifecycle: Arc<Mutex<DaemonLifecycle>>,
        query_timeout: Duration,
    ) -> Self {
        Self {
            listener,
            dispatcher: Arc::new(dispatcher),
            lifecycle,
            query_timeout,
        }
    }

    #[allow(dead_code)]
    pub async fn run(self, mut shutdown_rx: tokio::sync::watch::Receiver<bool>) -> Result<()> {
        info!("socket server starting");

        loop {
            tokio::select! {
                accept_result = self.listener.accept() => {
                    match accept_result {
                        Ok((stream, _addr)) => {
                            info!("client connected");
                            let dispatcher = self.dispatcher.clone();
                            let lifecycle = self.lifecycle.clone();
                            let timeout = self.query_timeout;
                            tokio::spawn(async move {
                                if let Err(e) = handle_connection(stream, dispatcher, lifecycle, timeout).await {
                                    warn!("connection error: {e}");
                                }
                            });
                        }
                        Err(e) => {
                            warn!("accept error: {e}");
                        }
                    }
                }
                _ = shutdown_rx.changed() => {
                    info!("socket server shutting down");
                    break;
                }
            }
        }

        Ok(())
    }
}

async fn handle_connection<D: QueryDispatcher + 'static>(
    stream: tokio::net::UnixStream,
    dispatcher: Arc<D>,
    lifecycle: Arc<Mutex<DaemonLifecycle>>,
    query_timeout: Duration,
) -> Result<()> {
    let mut framed: Framed<_, JsonRpcCodec> = Framed::new(stream, JsonRpcCodec::new());

    while let Some(msg_result) = framed.next().await {
        let msg = match msg_result {
            Ok(msg) => msg,
            Err(e) => {
                warn!("decode error: {e}");
                let error_resp =
                    ff_ipc::JsonRpcResponse::error(0, JsonRpcError::parse_error(e.to_string()));
                framed.send(JsonRpcMessage::Response(error_resp)).await?;
                continue;
            }
        };

        match msg {
            JsonRpcMessage::Request(request) => {
                {
                    let mut lc = lifecycle.lock().await;
                    lc.notify_activity();
                }

                let (notifications, response) =
                    process_request(&*dispatcher, &request, query_timeout).await;

                for notification in notifications {
                    framed
                        .send(JsonRpcMessage::Notification(notification))
                        .await?;
                }

                framed.send(JsonRpcMessage::Response(response)).await?;

                if QueryType::from_method(&request.method) == Some(QueryType::Shutdown) {
                    let lc = lifecycle.lock().await;
                    lc.request_shutdown();
                    break;
                }
            }
            JsonRpcMessage::Notification(_) => {
                warn!("unexpected notification from client, ignoring");
            }
            JsonRpcMessage::Response(_) => {
                warn!("unexpected response from client, ignoring");
            }
        }
    }

    info!("client disconnected");
    Ok(())
}

async fn process_request<D: QueryDispatcher>(
    dispatcher: &D,
    request: &JsonRpcRequest,
    query_timeout: Duration,
) -> (Vec<ff_ipc::JsonRpcNotification>, ff_ipc::JsonRpcResponse) {
    let query_type = match QueryType::from_method(&request.method) {
        Some(qt) => qt,
        None => {
            return (
                vec![],
                ff_ipc::JsonRpcResponse::error(
                    request.id,
                    JsonRpcError::method_not_found(&request.method),
                ),
            );
        }
    };

    let params = match parse_query_params(&request.method, request.params.clone()) {
        Ok(p) => p,
        Err(e) => {
            return (vec![], ff_ipc::JsonRpcResponse::error(request.id, e));
        }
    };

    let result = tokio::time::timeout(query_timeout, async {
        match params {
            QueryParams::Grep(p) => dispatcher.dispatch_grep(request.id, p).await,
            QueryParams::Search(p) => dispatcher.dispatch_search(request.id, p).await,
            QueryParams::Find(p) => dispatcher.dispatch_find(request.id, p).await,
            QueryParams::Empty => match query_type {
                QueryType::Ping => dispatcher.dispatch_ping(request.id),
                QueryType::Shutdown => dispatcher.dispatch_shutdown(request.id),
                _ => unreachable!(),
            },
        }
    })
    .await;

    match result {
        Ok(query_result) => (query_result.notifications, query_result.response),
        Err(_) => (
            vec![],
            ff_ipc::JsonRpcResponse::error(
                request.id,
                JsonRpcError::internal_error(format!(
                    "query timed out after {}s",
                    query_timeout.as_secs()
                )),
            ),
        ),
    }
}

#[allow(dead_code)]
pub fn create_server(
    lifecycle: Arc<Mutex<DaemonLifecycle>>,
    query_timeout: Duration,
) -> Result<SocketServer<StubDispatcher>> {
    let lc = lifecycle
        .try_lock()
        .map_err(|_| anyhow::anyhow!("lifecycle lock is held, cannot create socket"))?;
    let listener = lc.create_socket()?;
    drop(lc);

    Ok(SocketServer::new(
        listener,
        StubDispatcher,
        lifecycle,
        query_timeout,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ff_ipc::JsonRpcRequest;
    use std::path::PathBuf;

    fn test_lifecycle() -> Arc<Mutex<DaemonLifecycle>> {
        let dir = tempfile::TempDir::new().unwrap();
        unsafe {
            std::env::set_var("XDG_RUNTIME_DIR", dir.path());
        }
        Arc::new(Mutex::new(DaemonLifecycle::new(
            PathBuf::from("/test"),
            Duration::from_secs(300),
        )))
    }

    #[tokio::test]
    async fn process_ping_request() {
        let dispatcher = StubDispatcher;
        let request = JsonRpcRequest::new(1, "ping", serde_json::json!({}));
        let (notifications, response) =
            process_request(&dispatcher, &request, Duration::from_secs(30)).await;

        assert!(notifications.is_empty());
        assert_eq!(response.id, 1);
        assert!(response.result.is_some());
        let result = response.result.unwrap();
        assert_eq!(result["status"], "ok");
    }

    #[tokio::test]
    async fn process_shutdown_request() {
        let dispatcher = StubDispatcher;
        let request = JsonRpcRequest::new(2, "shutdown", serde_json::json!({}));
        let (notifications, response) =
            process_request(&dispatcher, &request, Duration::from_secs(30)).await;

        assert!(notifications.is_empty());
        assert_eq!(response.id, 2);
        let result = response.result.unwrap();
        assert_eq!(result["status"], "shutting_down");
    }

    #[tokio::test]
    async fn process_unknown_method() {
        let dispatcher = StubDispatcher;
        let request = JsonRpcRequest::new(3, "unknown", serde_json::json!({}));
        let (notifications, response) =
            process_request(&dispatcher, &request, Duration::from_secs(30)).await;

        assert!(notifications.is_empty());
        assert_eq!(response.id, 3);
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, -32601);
    }

    #[tokio::test]
    async fn process_invalid_params() {
        let dispatcher = StubDispatcher;
        let request = JsonRpcRequest::new(4, "grep", serde_json::json!({"wrong": "fields"}));
        let (notifications, response) =
            process_request(&dispatcher, &request, Duration::from_secs(30)).await;

        assert!(notifications.is_empty());
        assert_eq!(response.id, 4);
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, -32602);
    }

    #[tokio::test]
    async fn process_grep_request() {
        let dispatcher = StubDispatcher;
        let request = JsonRpcRequest::new(
            5,
            "grep",
            serde_json::json!({"pattern": "TODO", "caseMode": "smart"}),
        );
        let (notifications, response) =
            process_request(&dispatcher, &request, Duration::from_secs(30)).await;

        assert!(notifications.is_empty());
        assert_eq!(response.id, 5);
        assert!(response.result.is_some());
        let result = response.result.unwrap();
        assert_eq!(result["totalMatched"], 0);
    }

    #[tokio::test]
    async fn connection_roundtrip() {
        let lifecycle = test_lifecycle();
        let dispatcher = Arc::new(StubDispatcher);

        {
            let lc = lifecycle.lock().await;
            lc.check_and_clean_stale().unwrap();
            lc.create_pid_file().unwrap();
        }

        let (client_stream, server_stream) = tokio::net::UnixStream::pair().unwrap();

        let server_handle = tokio::spawn(async move {
            handle_connection(
                server_stream,
                dispatcher,
                lifecycle,
                Duration::from_secs(30),
            )
            .await
            .unwrap();
        });

        let mut client: Framed<_, JsonRpcCodec> = Framed::new(client_stream, JsonRpcCodec::new());
        let request = JsonRpcRequest::new(1, "ping", serde_json::json!({}));
        client.send(JsonRpcMessage::Request(request)).await.unwrap();

        let response = client.next().await.unwrap().unwrap();
        match response {
            JsonRpcMessage::Response(resp) => {
                assert_eq!(resp.id, 1);
                assert!(resp.result.is_some());
                assert_eq!(resp.result.unwrap()["status"], "ok");
            }
            _ => panic!("expected response"),
        }

        drop(client);
        server_handle.await.unwrap();
    }

    #[tokio::test]
    async fn concurrent_connections() {
        let lifecycle = test_lifecycle();
        let dispatcher = Arc::new(StubDispatcher);

        {
            let lc = lifecycle.lock().await;
            lc.check_and_clean_stale().unwrap();
            lc.create_pid_file().unwrap();
        }

        let mut handles = vec![];
        for i in 0..5 {
            let (client_stream, server_stream) = tokio::net::UnixStream::pair().unwrap();
            let dispatcher = dispatcher.clone();
            let lifecycle = lifecycle.clone();

            handles.push(tokio::spawn(async move {
                let server_handle = tokio::spawn(async move {
                    handle_connection(
                        server_stream,
                        dispatcher,
                        lifecycle,
                        Duration::from_secs(30),
                    )
                    .await
                    .unwrap();
                });

                let mut client: Framed<_, JsonRpcCodec> =
                    Framed::new(client_stream, JsonRpcCodec::new());
                let request = JsonRpcRequest::new(i, "ping", serde_json::json!({}));
                client.send(JsonRpcMessage::Request(request)).await.unwrap();

                let response = client.next().await.unwrap().unwrap();
                match response {
                    JsonRpcMessage::Response(resp) => {
                        assert_eq!(resp.id, i);
                    }
                    _ => panic!("expected response"),
                }

                drop(client);
                server_handle.await.unwrap();
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }
    }
}
