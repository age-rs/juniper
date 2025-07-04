//! [`WsIntegration`] testing for [`axum`].

#![expect(unused_crate_dependencies, reason = "integration tests")]
#![cfg_attr(windows, expect(dead_code, unused_imports, reason = "disabled"))]

use std::{net::SocketAddr, sync::Arc};

use anyhow::anyhow;
use axum::{Extension, Router, routing::get};
use futures::{SinkExt, StreamExt};
use juniper::{
    EmptyMutation, RootNode,
    http::tests::{WsIntegration, WsIntegrationMessage, graphql_transport_ws, graphql_ws},
    tests::fixtures::starwars::schema::{Database, Query, Subscription},
};
use juniper_axum::subscriptions;
use juniper_graphql_ws::ConnectionConfig;
use tokio::{
    net::{TcpListener, TcpStream},
    time::timeout,
};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};

type Schema = RootNode<Query, EmptyMutation<Database>, Subscription>;

struct TestApp(Router);

impl TestApp {
    fn new(protocol: &'static str) -> Self {
        let schema = Schema::new(Query, EmptyMutation::new(), Subscription);

        let mut router = Router::new();
        router = if protocol == "graphql-ws" {
            router.route(
                "/subscriptions",
                get(subscriptions::graphql_ws::<Arc<Schema>>(
                    ConnectionConfig::new(Database::new()),
                )),
            )
        } else {
            router.route(
                "/subscriptions",
                get(subscriptions::graphql_transport_ws::<Arc<Schema>>(
                    ConnectionConfig::new(Database::new()),
                )),
            )
        };
        router = router.layer(Extension(Arc::new(schema)));

        Self(router)
    }

    async fn process_message(
        websocket: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
        message: WsIntegrationMessage,
    ) -> Result<(), anyhow::Error> {
        match message {
            WsIntegrationMessage::Send(msg) => websocket
                .send(Message::Text(msg.to_string().into()))
                .await
                .map_err(|e| anyhow!("Could not send message: {e}"))
                .map(drop),

            WsIntegrationMessage::Expect(expected, duration) => {
                let message = timeout(duration, websocket.next())
                    .await
                    .map_err(|e| anyhow!("Timed out receiving message. Elapsed: {e}"))?;
                match message {
                    None => Err(anyhow!("No message received")),
                    Some(Err(e)) => Err(anyhow!("WebSocket error: {e}")),
                    Some(Ok(Message::Text(json))) => {
                        let actual: serde_json::Value = serde_json::from_str(&json)
                            .map_err(|e| anyhow!("Cannot deserialize received message: {e}"))?;
                        if actual != expected {
                            return Err(anyhow!(
                                "Expected message: {expected}. \
                                 Received message: {actual}",
                            ));
                        }
                        Ok(())
                    }
                    Some(Ok(Message::Close(Some(frame)))) => {
                        let actual = serde_json::json!({
                            "code": u16::from(frame.code),
                            "description": *frame.reason,
                        });
                        if actual != expected {
                            return Err(anyhow!(
                                "Expected message: {expected}. \
                                 Received message: {actual}",
                            ));
                        }
                        Ok(())
                    }
                    Some(Ok(msg)) => Err(anyhow!("Received non-text message: {msg:?}")),
                }
            }
        }
    }
}

impl WsIntegration for TestApp {
    async fn run(&self, messages: Vec<WsIntegrationMessage>) -> Result<(), anyhow::Error> {
        let listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap())
            .await
            .unwrap();
        let addr = listener.local_addr().unwrap();

        let router = self.0.clone();
        tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });

        let (mut websocket, _) = connect_async(format!("ws://{}/subscriptions", addr))
            .await
            .unwrap();

        for msg in messages {
            Self::process_message(&mut websocket, msg).await?;
        }

        Ok(())
    }
}

#[cfg(not(windows))]
#[tokio::test]
async fn test_graphql_ws_integration() {
    let app = TestApp::new("graphql-ws");
    graphql_ws::run_test_suite(&app).await;
}

#[cfg(not(windows))]
#[tokio::test]
async fn test_graphql_transport_integration() {
    let app = TestApp::new("graphql-transport-ws");
    graphql_transport_ws::run_test_suite(&app).await;
}
