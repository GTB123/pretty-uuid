use futures::Stream;
use nanoid::nanoid;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

use crate::store::uuid_server::Uuid;
use crate::store::{watch_response, UuidRequest, UuidResponse, WatchResponse};

//uuid server implementation
#[derive(Clone, Debug)]
enum LogMessage {
    Request(UuidRequest),
    Response(UuidResponse),
}


#[derive(Debug)]
pub struct UuidService {
    pub uuids: Arc<Mutex<HashMap<String, String>>>,
    log_channel: (
        broadcast::Sender<LogMessage>,
        broadcast::Receiver<LogMessage>,
    ),
}

impl Default for UuidService {
    fn default() -> Self {
        Self::new()
    }
}

impl UuidService {
    pub fn new() -> Self {
        let (tx, rx) = broadcast::channel(100); // Adjust the channel size as needed
        Self {
            uuids: Arc::new(Mutex::new(HashMap::new())),
            log_channel: (tx, rx),
        }
    }
}

#[tonic::async_trait]
impl Uuid for UuidService {
    async fn generate_uuid(
        &self,
        request: Request<UuidRequest>,
    ) -> Result<Response<UuidResponse>, Status> {
        let request = request.into_inner();
        let prefix = request.prefix.clone();
        let entity = request.entity.clone();
        //define a custom alphabet for the nanoid
        let alphabet = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz"
            .chars()
            .collect::<Vec<_>>();
        let uuid = nanoid!(22, &alphabet);
        let formatted_uuid = format!("{}:{}:{}", prefix, entity, uuid);

        let response = UuidResponse {
            uuid: formatted_uuid,
        };

        // Send the request to the logging channel
        let log_request = request.clone(); // Clone the request for logging
        let log_sender = self.log_channel.0.clone();
        let log_message = response.clone();
        tokio::spawn(async move {
            log_sender.send(LogMessage::Request(log_request)).unwrap();
            log_sender.send(LogMessage::Response(log_message)).unwrap();
        });

        Ok(Response::new(response))
    }

    type WatchStream = Pin<Box<dyn Stream<Item = Result<WatchResponse, Status>> + Send + Sync>>;

    async fn watch(
        &self,
        _request: Request<UuidRequest>,
    ) -> Result<Response<Self::WatchStream>, Status> {
        let (tx, rx) = mpsc::channel(4);
        let mut log_receiver = self.log_channel.0.subscribe(); // Assuming .1 is the Receiver part of the log_channel

        tokio::spawn(async move {
            while let Ok(log_message) = log_receiver.recv().await {
                match log_message {
                    LogMessage::Request(request) => {
                        println!("{:?}", request);
                        // Wrap the request in the new WatchResponse type
                        let watch_response = WatchResponse {
                            response_type: Some(watch_response::ResponseType::UuidRequest(request)),
                            //message: request_str
                        };
                        // Send the wrapped request
                        if tx.send(Ok(watch_response)).await.is_err() {
                            eprintln!("Error sending request through channel");
                            break;
                        }
                    }
                    LogMessage::Response(response) => {
                        println!("{:?}", response);
                        let watch_response = WatchResponse {
                            response_type: Some(watch_response::ResponseType::UuidResponse(response)),
                            //message: response_str
                        };
                        // Send the wrapped response
                        if tx.send(Ok(watch_response)).await.is_err() {
                            eprintln!("Error sending response through channel");
                            break;
                        }
                    }
                }
            }
        });

        let stream = ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(stream) as Self::WatchStream))
    }
}
