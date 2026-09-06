use caramelo::{expect, matchers::eq};
use deboa::{
    cert::{CertificateExt, ContentEncoding},
    request,
};
use deboa_tokio::cert::DeboaCertificate;
use http::{StatusCode, Version};
use http_body_util::BodyExt as _;
use std::error::Error;
use vetis::{host::Host as _, Response, VetisServer as _};
use vetis_rev_proxy::{tokio::ProxyPath, ProxyPathConfig};
use vetis_tokio::{
    handler_fn,
    host::{path::HandlerPath, Host},
    listener::build_listeners,
    HostConfig, ListenerConfig, SecurityConfig, Vetis,
};

use crate::common::{CA_CERT, SERVER_CERT, SERVER_KEY};

#[tokio::test]
async fn test_get_proxy_to_target() -> Result<(), Box<dyn Error>> {
    let source_listener = ListenerConfig::builder()
        .port(8084)
        .protos(vec![Version::HTTP_11])
        .interface(
            "0.0.0.0"
                .parse()
                .unwrap(),
        )
        .build()?;

    let target_listener = ListenerConfig::builder()
        .port(8085)
        .protos(vec![Version::HTTP_11])
        .interface(
            "0.0.0.0"
                .parse()
                .unwrap(),
        )
        .allow_unsafe_connections(true)
        .build()?;

    let security_config = SecurityConfig::builder()
        .ca_cert_from_bytes(CA_CERT.to_vec())
        .cert_from_bytes(SERVER_CERT.to_vec())
        .key_from_bytes(SERVER_KEY.to_vec())
        .build()?;

    let source_host_config = HostConfig::builder()
        .hostname("localhost")
        .root_directory("src/tests".into())
        .security(security_config.clone())
        .bind_addresses(vec![(
            "0.0.0.0"
                .parse()
                .unwrap(),
            8084,
        )])
        .build()?;

    let mut source_host = Host::new(source_host_config);
    source_host.add_path(ProxyPath::new(
        ProxyPathConfig::builder()
            .uri("/")
            .target("http://localhost:8085")
            .build()?,
    ));

    let target_host_config = HostConfig::builder()
        .hostname("localhost")
        .root_directory("src/tests".into())
        .bind_addresses(vec![(
            "0.0.0.0"
                .parse()
                .unwrap(),
            8085,
        )])
        .build()?;

    let mut target_host = Host::new(target_host_config);
    target_host.add_path(
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
        target_host
            .config()
            .hostname(),
        "localhost"
    );

    let mut server = Vetis::builder()
        .add_listeners(build_listeners(source_listener))?
        .add_listeners(build_listeners(target_listener))?
        .add_host(source_host)?
        .add_host(target_host)?
        .build();

    server
        .start()
        .await?;

    let client = deboa_tokio::Client::builder()
        .certificate(DeboaCertificate::from_slice(CA_CERT, ContentEncoding::DER))
        .prior_knowledge(true)
        .build();

    let request = request::get("https://localhost:8084/")?
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
        .protos(vec![Version::HTTP_11])
        .interface(
            "0.0.0.0"
                .parse()
                .unwrap(),
        )
        .build()?;

    let target_listener = ListenerConfig::builder()
        .port(9094)
        .protos(vec![Version::HTTP_11])
        .interface(
            "0.0.0.0"
                .parse()
                .unwrap(),
        )
        .allow_unsafe_connections(true)
        .build()?;

    let security_config = SecurityConfig::builder()
        .ca_cert_from_bytes(CA_CERT.to_vec())
        .cert_from_bytes(SERVER_CERT.to_vec())
        .key_from_bytes(SERVER_KEY.to_vec())
        .build()?;

    let source_host_config = HostConfig::builder()
        .hostname("localhost")
        .root_directory("src/tests".into())
        .security(security_config.clone())
        .bind_addresses(vec![(
            "0.0.0.0"
                .parse()
                .unwrap(),
            9093,
        )])
        .build()?;

    let mut source_host = Host::new(source_host_config);
    source_host.add_path(ProxyPath::new(
        ProxyPathConfig::builder()
            .uri("/")
            .target("http://localhost:9094")
            .build()?,
    ));

    let target_host_config = HostConfig::builder()
        .hostname("localhost")
        .root_directory("src/tests".into())
        .bind_addresses(vec![(
            "0.0.0.0"
                .parse()
                .unwrap(),
            9094,
        )])
        .build()?;

    let mut target_host = Host::new(target_host_config);
    target_host.add_path(
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
        target_host
            .config()
            .hostname(),
        "localhost"
    );

    let mut server = Vetis::builder()
        .add_listeners(build_listeners(source_listener))?
        .add_listeners(build_listeners(target_listener))?
        .add_host(source_host)?
        .add_host(target_host)?
        .build();

    server
        .start()
        .await?;

    let client = deboa_tokio::Client::builder()
        .certificate(DeboaCertificate::from_slice(CA_CERT, ContentEncoding::DER))
        .prior_knowledge(true)
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
