use bytes::{Bytes, BytesMut};
use tokio_util::codec::{Decoder, Encoder, LengthDelimitedCodec};

use crate::protocol::{JsonRpcMessage, ProtocolError};

pub const MAX_FRAME_LENGTH: usize = 100 * 1024 * 1024;

#[derive(Debug)]
pub struct JsonRpcCodec {
    inner: LengthDelimitedCodec,
    max_frame_length: usize,
}

impl Default for JsonRpcCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonRpcCodec {
    #[must_use]
    pub fn new() -> Self {
        let inner = LengthDelimitedCodec::builder()
            .max_frame_length(MAX_FRAME_LENGTH)
            .length_field_length(4)
            .big_endian()
            .new_codec();
        Self {
            inner,
            max_frame_length: MAX_FRAME_LENGTH,
        }
    }

    #[must_use]
    pub fn with_max_frame_length(max_frame_length: usize) -> Self {
        let inner = LengthDelimitedCodec::builder()
            .max_frame_length(max_frame_length)
            .length_field_length(4)
            .big_endian()
            .new_codec();
        Self {
            inner,
            max_frame_length,
        }
    }
}

impl Decoder for JsonRpcCodec {
    type Item = JsonRpcMessage;
    type Error = ProtocolError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match self.inner.decode(src)? {
            Some(frame) => {
                let message = JsonRpcMessage::from_json(&frame)?;
                Ok(Some(message))
            }
            None => Ok(None),
        }
    }
}

impl Encoder<JsonRpcMessage> for JsonRpcCodec {
    type Error = ProtocolError;

    fn encode(&mut self, item: JsonRpcMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let json = item.to_json()?;
        if json.len() > self.max_frame_length {
            return Err(ProtocolError::MessageTooLarge {
                size: json.len(),
                max: self.max_frame_length,
            });
        }
        self.inner
            .encode(Bytes::from(json), dst)
            .map_err(|e| ProtocolError::Transport(format!("failed to encode frame: {e}")))?;
        Ok(())
    }
}

impl Encoder<Bytes> for JsonRpcCodec {
    type Error = ProtocolError;

    fn encode(&mut self, item: Bytes, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.inner
            .encode(item, dst)
            .map_err(|e| ProtocolError::Transport(format!("failed to encode frame: {e}")))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};

    #[test]
    fn encode_decode_request() {
        let mut codec = JsonRpcCodec::new();
        let request = JsonRpcRequest::new(1, "ping", serde_json::json!({}));
        let message = JsonRpcMessage::Request(request);

        let mut buf = BytesMut::new();
        codec.encode(message.clone(), &mut buf).unwrap();

        assert!(buf.len() > 4);
        let length = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
        assert_eq!(length, buf.len() - 4);

        let decoded = codec.decode(&mut buf).unwrap().unwrap();
        match decoded {
            JsonRpcMessage::Request(req) => {
                assert_eq!(req.id, 1);
                assert_eq!(req.method, "ping");
            }
            _ => panic!("expected request"),
        }
    }

    #[test]
    fn encode_decode_response() {
        let mut codec = JsonRpcCodec::new();
        let response = JsonRpcResponse::success(1, serde_json::json!({"status": "ok"}));
        let message = JsonRpcMessage::Response(response);

        let mut buf = BytesMut::new();
        codec.encode(message, &mut buf).unwrap();

        let decoded = codec.decode(&mut buf).unwrap().unwrap();
        match decoded {
            JsonRpcMessage::Response(resp) => {
                assert_eq!(resp.id, Some(1));
                assert!(resp.result.is_some());
            }
            _ => panic!("expected response"),
        }
    }

    #[test]
    fn encode_decode_notification() {
        let mut codec = JsonRpcCodec::new();
        let notification = JsonRpcNotification::result(serde_json::json!([{"path": "test.rs"}]));
        let message = JsonRpcMessage::Notification(notification);

        let mut buf = BytesMut::new();
        codec.encode(message, &mut buf).unwrap();

        let decoded = codec.decode(&mut buf).unwrap().unwrap();
        match decoded {
            JsonRpcMessage::Notification(notif) => {
                assert_eq!(notif.method, "result");
            }
            _ => panic!("expected notification"),
        }
    }

    #[test]
    fn decode_partial_frame() {
        let mut codec = JsonRpcCodec::new();
        let request = JsonRpcRequest::new(1, "ping", serde_json::json!({}));
        let message = JsonRpcMessage::Request(request);

        let mut buf = BytesMut::new();
        codec.encode(message, &mut buf).unwrap();

        let partial = buf.split_to(5);
        let mut partial_buf = partial;
        let result = codec.decode(&mut partial_buf).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn decode_multiple_messages() {
        let mut codec = JsonRpcCodec::new();
        let mut buf = BytesMut::new();

        for i in 0..5 {
            let request = JsonRpcRequest::new(i, "ping", serde_json::json!({}));
            codec
                .encode(JsonRpcMessage::Request(request), &mut buf)
                .unwrap();
        }

        for i in 0..5 {
            let decoded = codec.decode(&mut buf).unwrap().unwrap();
            match decoded {
                JsonRpcMessage::Request(req) => assert_eq!(req.id, i),
                _ => panic!("expected request"),
            }
        }

        assert!(codec.decode(&mut buf).unwrap().is_none());
    }

    #[test]
    fn decode_invalid_json() {
        let mut codec = JsonRpcCodec::new();

        let json = b"not valid json";
        let length = (json.len() as u32).to_be_bytes();
        let mut buf = BytesMut::new();
        buf.extend_from_slice(&length);
        buf.extend_from_slice(json);

        let result = codec.decode(&mut buf);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ProtocolError::JsonParse(_)));
    }

    #[test]
    fn connection_remains_open_after_invalid_json() {
        let mut codec = JsonRpcCodec::new();
        let mut buf = BytesMut::new();

        let invalid = b"not valid json";
        let length = (invalid.len() as u32).to_be_bytes();
        buf.extend_from_slice(&length);
        buf.extend_from_slice(invalid);

        let request = JsonRpcRequest::new(2, "ping", serde_json::json!({}));
        codec
            .encode(JsonRpcMessage::Request(request), &mut buf)
            .unwrap();

        let result = codec.decode(&mut buf);
        assert!(result.is_err());

        let decoded = codec.decode(&mut buf).unwrap().unwrap();
        match decoded {
            JsonRpcMessage::Request(req) => {
                assert_eq!(req.id, 2);
                assert_eq!(req.method, "ping");
            }
            _ => panic!("expected request after invalid frame"),
        }
    }

    #[test]
    fn length_prefix_is_big_endian_u32() {
        let mut codec = JsonRpcCodec::new();
        let request = JsonRpcRequest::new(1, "ping", serde_json::json!({}));
        let message = JsonRpcMessage::Request(request);

        let mut buf = BytesMut::new();
        codec.encode(message, &mut buf).unwrap();

        let length_bytes = [buf[0], buf[1], buf[2], buf[3]];
        let length = u32::from_be_bytes(length_bytes);
        assert_eq!(length as usize, buf.len() - 4);

        let expected_json = br#"{"jsonrpc":"2.0","id":1,"method":"ping","params":{}}"#;
        assert_eq!(&buf[4..], expected_json);
    }

    #[test]
    fn large_message_roundtrip() {
        let mut codec = JsonRpcCodec::new();

        let large_items: Vec<serde_json::Value> = (0..10_000)
            .map(|i| {
                serde_json::json!({
                    "path": format!("src/file_{i}.rs"),
                    "lineNumber": i,
                    "content": "x".repeat(100)
                })
            })
            .collect();

        let notification = JsonRpcNotification::result(serde_json::json!(large_items));
        let message = JsonRpcMessage::Notification(notification);

        let mut buf = BytesMut::new();
        codec.encode(message, &mut buf).unwrap();

        let json_size = buf.len() - 4;
        assert!(json_size > 100_000);

        let decoded = codec.decode(&mut buf).unwrap().unwrap();
        match decoded {
            JsonRpcMessage::Notification(notif) => {
                assert_eq!(notif.method, "result");
                let items = notif.params["items"].as_array().unwrap();
                assert_eq!(items.len(), 10_000);
            }
            _ => panic!("expected notification"),
        }
    }
}
