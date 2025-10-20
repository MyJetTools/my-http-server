use bytes::Bytes;
use futures::{channel::mpsc::SendError, SinkExt};

pub struct HttpOutputProducer {
    tx: futures::channel::mpsc::Sender<Result<hyper::body::Frame<Bytes>, hyper::Error>>,
}

impl HttpOutputProducer {
    pub fn new(
        tx: futures::channel::mpsc::Sender<Result<hyper::body::Frame<Bytes>, hyper::Error>>,
    ) -> Self {
        Self { tx }
    }

    pub async fn send(&mut self, bytes: Vec<u8>) -> Result<(), SendError> {
        let bytes: Bytes = bytes.into();

        let frame = hyper::body::Frame::data(bytes);

        self.tx.send(Ok(frame)).await
    }
}
