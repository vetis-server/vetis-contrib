use caramelo::{expect, matchers::eq};
use deboa::{
    cert::{CertificateExt, ContentEncoding},
    request,
};
use deboa_tokio::cert::DeboaCertificate;
use http::{StatusCode, Version};
use http_body_util::BodyExt as _;
use std::error::Error;
use vetis::{virtual_host::VirtualHost as _, Response, VetisServer as _};
use vetis_proxy::{tokio::ProxyPath, ProxyPathConfig};
use vetis_tokio::{
    handler_fn,
    virtual_host::{path::HandlerPath, VirtualHostImpl},
    ListenerConfig, SecurityConfig, ServerConfig, Vetis, VirtualHostConfig,
};

use crate::common::{CA_CERT, SERVER_CERT, SERVER_KEY};

#[tokio::test]
async fn test_get_proxy_to_target() -> Result<(), Box<dyn Error>> {
    let source_listener = ListenerConfig::builder()
        .port(8084)
        .protocol_version(Version::HTTP_11)
        .interface("0.0.0.0")
        .build()?;

    let target_listener = ListenerConfig::builder()
        .port(8085)
        .protocol_version(Version::HTTP_11)
        .interface("0.0.0.0")
        .build()?;

    let config = ServerConfig::builder()
        .add_listener(source_listener)
        .add_listener(target_listener)
        .build()?;

    let security_config = SecurityConfig::builder()
        .ca_cert_from_bytes(CA_CERT.to_vec())
        .cert_from_bytes(SERVER_CERT.to_vec())
        .key_from_bytes(SERVER_KEY.to_vec())
        .build()?;

    let source_config = VirtualHostConfig::builder()
        .hostname("localhost")
        .port(8084)
        .root_directory("src/tests")
        .security(security_config.clone())
        .build()?;

    let mut source_virtual_host = VirtualHostImpl::new(source_config);
    source_virtual_host.add_path(ProxyPath::new(
        ProxyPathConfig::builder()
            .uri("/")
            .target("http://localhost:8085")
            .build()?,
    ));

    let target_config = VirtualHostConfig::builder()
        .hostname("localhost")
        .port(8085)
        .root_directory("src/tests")
        .build()?;

    let mut target_virtual_host = VirtualHostImpl::new(target_config);
    target_virtual_host.add_path(
        HandlerPath::builder()
            .uri("/")
            .handler(handler_fn(|_request| async move {
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .text("Hello, world!"))
            }))
            .build()?,
    );

    assert_eq!(
        target_virtual_host
            .config()
            .hostname(),
        "localhost"
    );

    let mut server = Vetis::new(config);
    server
        .add_virtual_host(source_virtual_host)
        .await;
    server
        .add_virtual_host(target_virtual_host)
        .await;

    server
        .start()
        .await?;

    let client = deboa_tokio::Client::builder()
        .certificate(DeboaCertificate::from_slice(CA_CERT, ContentEncoding::DER))
        .build();

    let request = request::get("https://localhost:8085/")?
        .version(Version::HTTP_11)
        .send_with(&client)
        .await?;

    expect(request.status()).to_be(eq(StatusCode::OK));
    expect(
        request
            .text()
            .await?,
    )
    .to_be(eq("Hello, world!"));

    server
        .stop()
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_post_proxy_to_target() -> Result<(), Box<dyn Error>> {
    let source_listener = ListenerConfig::builder()
        .port(9093)
        .protocol_version(Version::HTTP_11)
        .interface("0.0.0.0")
        .build()?;

    let target_listener = ListenerConfig::builder()
        .port(9094)
        .protocol_version(Version::HTTP_11)
        .interface("0.0.0.0")
        .build()?;

    let config = ServerConfig::builder()
        .add_listener(source_listener)
        .add_listener(target_listener)
        .build()?;

    let security_config = SecurityConfig::builder()
        .ca_cert_from_bytes(CA_CERT.to_vec())
        .cert_from_bytes(SERVER_CERT.to_vec())
        .key_from_bytes(SERVER_KEY.to_vec())
        .build()?;

    let source_config = VirtualHostConfig::builder()
        .hostname("localhost")
        .port(9093)
        .root_directory("src/tests")
        .security(security_config.clone())
        .build()?;

    let mut source_virtual_host = VirtualHostImpl::new(source_config);
    source_virtual_host.add_path(ProxyPath::new(
        ProxyPathConfig::builder()
            .uri("/")
            .target("http://localhost:9094")
            .build()?,
    ));

    let target_config = VirtualHostConfig::builder()
        .hostname("localhost")
        .port(9094)
        .root_directory("src/tests")
        .build()?;

    let mut target_virtual_host = VirtualHostImpl::new(target_config);
    target_virtual_host.add_path(
        HandlerPath::builder()
            .uri("/")
            .handler(handler_fn(|request| async move {
                let (_parts, body) = request.into_parts();
                let text = body
                    .collect()
                    .await
                    .unwrap()
                    .to_bytes();
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .bytes(text.as_ref()))
            }))
            .build()?,
    );

    assert_eq!(
        target_virtual_host
            .config()
            .hostname(),
        "localhost"
    );

    let mut server = Vetis::new(config);
    server
        .add_virtual_host(source_virtual_host)
        .await;
    server
        .add_virtual_host(target_virtual_host)
        .await;

    server
        .start()
        .await?;

    let client = deboa_tokio::Client::builder()
        .certificate(DeboaCertificate::from_slice(CA_CERT, ContentEncoding::DER))
        .build();

    let response = request::post("https://localhost:9093/")?
        .text("Something cool!")
        .version(Version::HTTP_11)
        .send_with(&client)
        .await?;

    expect(response.status()).to_be(eq(StatusCode::OK));
    expect(
        response
            .text()
            .await?,
    )
    .to_be(eq("Something cool!"));

    server
        .stop()
        .await?;

    Ok(())
}
