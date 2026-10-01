#[cfg(feature = "runtime-tokio")]
mod tokio {
    use crate::common::{CA_CERT, SERVER_CERT, SERVER_KEY};
    use deboa::{
        cert::{CertificateExt as _, ContentEncoding},
        request::DeboaRequestBuilder,
    };
    use deboa_tokio::cert::DeboaCertificate;
    use deboa_ws::{IntoWebSocket as _, WebSocketExt, WebSocketRead as _, WebsocketRequestBuilder};
    use http::Version;
    use hyper::upgrade::Upgraded;
    use hyper_util::rt::TokioIo;
    use std::net::{IpAddr, Ipv4Addr};
    use vetis::{VetisServer, VetisTestResult, host::HostConfig, security::TlsConfig};
    use vetis_tokio::{Vetis, host::Host};
    use vetis_ws::{
        WebSocketClient,
        proto::websocket::WebSocketExt as _,
        tokio::{WebSocketPath, io::websocket::WebSocketProto},
    };

    #[derive(Clone, Copy, Default)]
    pub struct ChatClient;

    impl ChatClient {
        pub fn new() -> Self {
            Self
        }
    }

    impl WebSocketClient for ChatClient {
        type WebSocket = vetis_ws::WebSocket<TokioIo<Upgraded>, WebSocketProto>;

        fn run(
            &self,
            mut socket: Self::WebSocket,
        ) -> impl std::future::Future<Output = vetis::VetisResult<()>> + Send {
            async move {
                loop {
                    if let Ok(Some(message)) =
                        vetis_ws::proto::websocket::WebSocketRead::read_message(&mut socket).await
                    {
                        match message {
                            vetis_ws::proto::websocket::Message::Text(text) => {
                                println!("Server received: {}", text);
                                socket
                                    .send_text(&format!("Echo: {}", text))
                                    .await?;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    struct ChatClientFactory;

    impl Default for ChatClientFactory {
        fn default() -> Self {
            Self
        }
    }

    impl vetis_ws::WebSocketClientFactory for ChatClientFactory {
        type WebSocketClient = ChatClient;

        fn create_client(&self) -> Self::WebSocketClient {
            Self::WebSocketClient::new()
        }
    }

    #[tokio::test]
    async fn test_index() -> VetisTestResult<()> {
        let security_config = TlsConfig::builder()
            .ca_file(CA_CERT)
            .cert_file(SERVER_CERT)
            .key_file(SERVER_KEY)
            .build()?;

        let host_config = HostConfig::builder()
            .hostname("localhost")
            .protos(&[Version::HTTP_11])
            .root_directory("..")
            .tls(security_config)
            .bind_addresses(&[(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 9100)])
            .build()?;

        let mut virtual_host = Host::new(host_config).await?;

        virtual_host.add_path(
            WebSocketPath::builder()
                .uri("/chat")
                .sub_proto(WebSocketProto::default())
                .factory(ChatClientFactory::default())
                .build()?,
        );

        let mut server = Vetis::builder()
            .add_host(virtual_host)
            .await?
            .build();

        server
            .start()
            .await?;

        let cert = tokio::fs::read(format!("../{CA_CERT}")).await?;
        let mut client = deboa_tokio::Client::builder()
            .certificate(DeboaCertificate::from_slice(&cert, ContentEncoding::DER))
            .build();

        let request = DeboaRequestBuilder::websocket("wss://localhost:9100/chat")?;
        let response = request
            .send_with(&mut client)
            .await
            .unwrap();
        let mut ws = response
            .into_websocket()
            .await
            .unwrap();
        ws.send_text("Hello, WebSocket!")
            .await
            .unwrap();
        while let Ok(Some(deboa_ws::Message::Text(message))) = ws
            .read_message()
            .await
        {
            println!("Client received: {}", message);
        }

        server
            .stop()
            .await?;

        Ok(())
    }
}
