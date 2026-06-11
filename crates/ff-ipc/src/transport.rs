use std::path::{Path, PathBuf};
use tokio::net::UnixStream;
use tokio_util::codec::Framed;

use crate::codec::JsonRpcCodec;

pub type IpcFramed = Framed<UnixStream, JsonRpcCodec>;

#[derive(Debug)]
pub struct IpcTransport {
    inner: IpcFramed,
}

impl IpcTransport {
    pub async fn connect(path: impl AsRef<Path>) -> std::io::Result<Self> {
        let stream = UnixStream::connect(path).await?;
        Ok(Self::from_stream(stream))
    }

    #[must_use]
    pub fn from_stream(stream: UnixStream) -> Self {
        Self {
            inner: Framed::new(stream, JsonRpcCodec::new()),
        }
    }

    #[must_use]
    pub fn from_stream_with_codec(stream: UnixStream, codec: JsonRpcCodec) -> Self {
        Self {
            inner: Framed::new(stream, codec),
        }
    }

    pub fn into_inner(self) -> IpcFramed {
        self.inner
    }

    pub fn into_split(
        self,
    ) -> (
        tokio_util::codec::FramedRead<tokio::net::unix::OwnedReadHalf, JsonRpcCodec>,
        tokio_util::codec::FramedWrite<tokio::net::unix::OwnedWriteHalf, JsonRpcCodec>,
    ) {
        let (read, write) = self.inner.into_inner().into_split();
        (
            tokio_util::codec::FramedRead::new(read, JsonRpcCodec::new()),
            tokio_util::codec::FramedWrite::new(write, JsonRpcCodec::new()),
        )
    }
}

#[must_use]
pub fn socket_path(root: &Path) -> PathBuf {
    ff_common::paths::socket_path(root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{JsonRpcMessage, JsonRpcRequest, JsonRpcResponse};
    use futures_util::{SinkExt, StreamExt};

    #[tokio::test]
    async fn unix_socket_roundtrip() {
        let (server_stream, client_stream) = UnixStream::pair().unwrap();

        let server_handle = tokio::spawn(async move {
            let mut server: IpcFramed = Framed::new(server_stream, JsonRpcCodec::new());
            let msg = tokio::time::timeout(std::time::Duration::from_secs(1), server.next())
                .await
                .unwrap()
                .unwrap()
                .unwrap();
            match msg {
                JsonRpcMessage::Request(req) => {
                    assert_eq!(req.id, 1);
                    assert_eq!(req.method, "ping");
                }
                _ => panic!("expected request"),
            }
        });

        let mut client: IpcFramed = Framed::new(client_stream, JsonRpcCodec::new());
        let request = JsonRpcRequest::new(1, "ping", serde_json::json!({}));
        client.send(JsonRpcMessage::Request(request)).await.unwrap();

        server_handle.await.unwrap();
    }

    #[tokio::test]
    async fn unix_socket_bidirectional() {
        let (server_stream, client_stream) = UnixStream::pair().unwrap();

        let server_handle = tokio::spawn(async move {
            let mut server: IpcFramed = Framed::new(server_stream, JsonRpcCodec::new());

            let msg = tokio::time::timeout(std::time::Duration::from_secs(1), server.next())
                .await
                .unwrap()
                .unwrap()
                .unwrap();

            let req = match msg {
                JsonRpcMessage::Request(req) => req,
                _ => panic!("expected request"),
            };
            assert_eq!(req.id, 42);

            let response = JsonRpcResponse::success(42, serde_json::json!({"totalMatched": 5}));
            server
                .send(JsonRpcMessage::Response(response))
                .await
                .unwrap();
        });

        let mut client: IpcFramed = Framed::new(client_stream, JsonRpcCodec::new());
        let request = JsonRpcRequest::new(42, "grep", serde_json::json!({"pattern": "test"}));
        client.send(JsonRpcMessage::Request(request)).await.unwrap();

        let msg = tokio::time::timeout(std::time::Duration::from_secs(1), client.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap();

        match msg {
            JsonRpcMessage::Response(resp) => {
                assert_eq!(resp.id, Some(42));
                assert_eq!(resp.result.unwrap()["totalMatched"], 5);
            }
            _ => panic!("expected response"),
        }

        server_handle.await.unwrap();
    }

    #[tokio::test]
    async fn transport_split_works() {
        let (server_stream, client_stream) = UnixStream::pair().unwrap();

        let server = IpcTransport::from_stream(server_stream);
        let client = IpcTransport::from_stream(client_stream);

        let (_server_read, _server_write) = server.into_split();
        let (_client_read, _client_write) = client.into_split();
    }

    #[test]
    fn socket_path_returns_sock_file() {
        let path = socket_path(Path::new("/home/user/project"));
        assert!(path.to_string_lossy().ends_with(".sock"));
    }
}
