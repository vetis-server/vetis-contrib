use caramelo::{expect, matchers::eq};
use deboa::{
    cert::{CertificateExt, ContentEncoding},
    request,
};
use deboa_tokio::cert::DeboaCertificate;
use http::{StatusCode, Version};
use http_body_util::BodyExt as _;
use tokio::fs;
use vetis::{Response, VetisServer as _, VetisTestResult, host::Host as _};
use vetis_rev_proxy::{ReverseProxyPathConfig, tokio::ReveseProxyPath};
use vetis_tokio::{
    HostConfig, TlsConfig, Vetis,
    host::{
        Host,
        path::{HandlerPath, handler_fn},
    },
};

use crate::common::{CA_CERT, SERVER_CERT, SERVER_KEY};

#[tokio::test]
async fn test_get_proxy_to_target() -> VetisTestResult<()> {
    let security_config = TlsConfig::builder()
        .ca_file(CA_CERT)
        .cert_file(SERVER_CERT)
        .key_file(SERVER_KEY)
        .build()?;

    let source_host_config = HostConfig::builder()
        .hostname("localhost")
        .root_directory("..")
        .protos(&[Version::HTTP_11])
        .tls(security_config)
        .bind_addresses(&[(
            "0.0.0.0"
                .parse()
                .unwrap(),
            8084,
        )])
        .build()?;

    let mut source_host = Host::new(source_host_config).await?;
    source_host.add_path(ReveseProxyPath::new(
        ReverseProxyPathConfig::builder()
            .uri("/")
            .target("http://localhost:8085")
            .build()?,
    ));

    let target_host_config = HostConfig::builder()
        .hostname("localhost")
        .root_directory("..")
        .bind_addresses(&[(
            "0.0.0.0"
                .parse()
                .unwrap(),
            8085,
        )])
        .build()?;

    let mut target_host = Host::new(target_host_config).await?;
    target_host.add_path(
        HandlerPath::builder()
            .uri("/")
            .handler(handler_fn(|_req, _ctx| async move {
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
        .add_host(source_host)
        .await?
        .add_host(target_host)
        .await?
        .build();

    server
        .start()
        .await?;

    let cert = fs::read(format!("../{CA_CERT}")).await?;
    let client = deboa_tokio::Client::builder()
        .certificate(DeboaCertificate::from_slice(&cert, ContentEncoding::DER))
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
async fn test_post_proxy_to_target() -> VetisTestResult<()> {
    let security_config = TlsConfig::builder()
        .ca_file(CA_CERT)
        .cert_file(SERVER_CERT)
        .key_file(SERVER_KEY)
        .build()?;

    let source_host_config = HostConfig::builder()
        .hostname("localhost")
        .root_directory("..")
        .protos(&[Version::HTTP_11])
        .tls(security_config.clone())
        .bind_addresses(&[(
            "0.0.0.0"
                .parse()
                .unwrap(),
            9093,
        )])
        .build()?;

    let mut source_host = Host::new(source_host_config).await?;
    source_host.add_path(ReveseProxyPath::new(
        ReverseProxyPathConfig::builder()
            .uri("/")
            .target("http://localhost:9094")
            .build()?,
    ));

    let target_host_config = HostConfig::builder()
        .hostname("localhost")
        .root_directory("..")
        .bind_addresses(&[(
            "0.0.0.0"
                .parse()
                .unwrap(),
            9094,
        )])
        .build()?;

    let mut target_host = Host::new(target_host_config).await?;
    target_host.add_path(
        HandlerPath::builder()
            .uri("/")
            .handler(handler_fn(|req, _ctx| async move {
                let (_parts, body) = req.into_parts();
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
        .add_host(source_host)
        .await?
        .add_host(target_host)
        .await?
        .build();

    server
        .start()
        .await?;

    let cert = fs::read(format!("../{CA_CERT}")).await?;
    let client = deboa_tokio::Client::builder()
        .certificate(DeboaCertificate::from_slice(&cert, ContentEncoding::DER))
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
