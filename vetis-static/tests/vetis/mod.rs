#[cfg(feature = "runtime-tokio")]
mod tokio {
    use crate::common::{CA_CERT, SERVER_CERT, SERVER_KEY};
    use caramelo::{expect, matchers::eq};
    use deboa::{
        cert::{CertificateExt as _, ContentEncoding},
        request,
    };
    use deboa_tokio::cert::DeboaCertificate;
    use http::{StatusCode, Version};
    use serde::Deserialize;
    use std::{
        net::{IpAddr, Ipv4Addr},
        path::Path,
    };
    use tokio::fs::read_to_string;
    use vetis::{VetisServer, VetisTestResult, host::HostConfig, security::TlsConfig};
    use vetis_macros::status_pages;
    use vetis_static::{StaticPathConfig, tokio::StaticPath};
    use vetis_tokio::{ServerConfig, Vetis, host::Host};

    #[allow(unused)]
    #[derive(Deserialize, Clone)]
    pub struct VetisConfig {
        log_level: String,
        worker_threads: usize,
        max_blocking_threads: usize,
        server: ServerConfig,
    }

    #[tokio::test]
    async fn test_from_file() -> VetisTestResult<()> {
        let config_path = Path::new("files/vetis-static.yaml");
        if config_path.exists() {
            let file = read_to_string(&config_path).await;
            if let Ok(file) = file {
                let config = serde_yaml_ng::from_str::<VetisConfig>(&file);
                if let Ok(config) = config {
                    let config_clone = config
                        .server
                        .clone();
                    expect(
                        config_clone
                            .hosts()
                            .len(),
                    )
                    .to_be(eq(1));

                    let host = &config_clone.hosts()[0];
                    expect(host.hostname()).to_be(eq("localhost"));
                    expect(host.paths().len()).to_be(eq(1));

                    let static_path_config = &host.paths()[0];
                    let static_path = static_path_config.boxed_path();
                    expect(static_path.uri()).to_be(eq("/"));

                    let mut server = Vetis::from_config(config.server).await?;
                    server
                        .start()
                        .await?;

                    let cert = tokio::fs::read(format!("../{CA_CERT}")).await?;
                    let client = deboa_tokio::Client::builder()
                        .certificate(DeboaCertificate::from_slice(&cert, ContentEncoding::DER))
                        .build();

                    let request = request::get("https://localhost:8443/")?
                        .version(Version::HTTP_2)
                        .send_with(&client)
                        .await?;

                    assert_eq!(request.status(), http::StatusCode::OK);

                    server
                        .stop()
                        .await?;
                } else {
                    eprintln!(
                        "Could not read : {}",
                        config
                            .err()
                            .unwrap()
                    );
                }
            } else {
                eprintln!("Failed to start server: {}", config_path.display());
            }
        } else {
            eprintln!(
                "Failed to start server: Config file does not exist: {}",
                config_path.display()
            );
        }

        Ok(())
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
        virtual_host.add_path(StaticPath::new(
            StaticPathConfig::builder()
                .uri("/")
                .directory("files")
                .index_files(vec!["index.html".to_string()])
                .build()?,
        ));

        let mut server = Vetis::builder()
            .add_host(virtual_host)
            .await?
            .build();

        server
            .start()
            .await?;

        let cert = tokio::fs::read(format!("../{CA_CERT}")).await?;
        let client = deboa_tokio::Client::builder()
            .certificate(DeboaCertificate::from_slice(&cert, ContentEncoding::DER))
            .build();

        let request = request::get("https://localhost:9100/")?
            .version(Version::HTTP_11)
            .send_with(&client)
            .await?;

        assert_eq!(request.status(), http::StatusCode::OK);

        let expected = if cfg!(windows) {
            "<html>\r\n<head>\r\n  <title>\r\n    Tested!\r\n  </title>\r\n</head>\r\n<body>\r\n  <p>\r\n    Tested!\r\n  </p>\r\n</body>\r\n</html>\r\n"
        } else {
            "<html>\n<head>\n  <title>\n    Tested!\n  </title>\n</head>\n<body>\n  <p>\n    Tested!\n  </p>\n</body>\n</html>\n"
        };

        expect(
            request
                .text()
                .await?,
        )
        .to_be(eq(expected.to_string()));

        server
            .stop()
            .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_not_found() -> VetisTestResult<()> {
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
            .bind_addresses(&[(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 9000)])
            .status_pages(status_pages! {
               404 @ "files/404.html".to_string()
            })
            .build()?;

        let mut virtual_host = Host::new(host_config).await?;
        virtual_host.add_path(StaticPath::new(
            StaticPathConfig::builder()
                .uri("/")
                .directory("files")
                .build()?,
        ));

        let mut server = Vetis::builder()
            .add_host(virtual_host)
            .await?
            .build();

        server
            .start()
            .await?;

        let cert = tokio::fs::read(format!("../{CA_CERT}")).await?;
        let client = deboa_tokio::Client::builder()
            .certificate(DeboaCertificate::from_slice(&cert, ContentEncoding::DER))
            .build();

        let request = request::get("https://localhost:9000/some/file/here.txt")?
            .version(Version::HTTP_11)
            .send_with(&client)
            .await?;

        expect(request.status()).to_be(eq(StatusCode::NOT_FOUND));

        server
            .stop()
            .await?;

        Ok(())
    }
}
