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
    use std::error::Error;
    use vetis::{
        listener::ListenerConfig, security::SecurityConfig, server::ServerConfig,
        virtual_host::VirtualHostConfig, VetisServer,
    };
    use vetis_macros::status_pages;
    use vetis_static::{tokio::StaticPath, StaticPathConfig};
    use vetis_tokio::{virtual_host::VirtualHostImpl, Vetis};

    #[tokio::test]
    async fn test_index() -> Result<(), Box<dyn Error>> {
        let listener = ListenerConfig::builder()
            .port(9100)
            .protocol_version(Version::HTTP_11)
            .interface("0.0.0.0")
            .build()?;

        let config = ServerConfig::builder()
            .add_listener(listener)
            .build()?;

        let security_config = SecurityConfig::builder()
            .ca_cert_from_bytes(CA_CERT.to_vec())
            .cert_from_bytes(SERVER_CERT.to_vec())
            .key_from_bytes(SERVER_KEY.to_vec())
            .build()?;

        let host_config = VirtualHostConfig::builder()
            .hostname("localhost")
            .port(9100)
            .root_directory("tests")
            .security(security_config.clone())
            .build()?;

        let mut virtual_host = VirtualHostImpl::new(host_config);
        virtual_host.add_path(StaticPath::new(
            StaticPathConfig::builder()
                .uri("/")
                .directory("tests/files")
                .index_files(vec!["index.html".to_string()])
                .build()?,
        ));

        let mut server = Vetis::new(config);
        server
            .add_virtual_host(virtual_host)
            .await;

        server
            .start()
            .await?;

        let client = deboa_tokio::Client::builder()
            .certificate(DeboaCertificate::from_slice(CA_CERT, ContentEncoding::DER))
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
    async fn test_not_found() -> Result<(), Box<dyn Error>> {
        let listener = ListenerConfig::builder()
            .port(9000)
            .protocol_version(Version::HTTP_11)
            .interface("0.0.0.0")
            .build()?;

        let config = ServerConfig::builder()
            .add_listener(listener)
            .build()?;

        let security_config = SecurityConfig::builder()
            .ca_cert_from_bytes(CA_CERT.to_vec())
            .cert_from_bytes(SERVER_CERT.to_vec())
            .key_from_bytes(SERVER_KEY.to_vec())
            .build()?;

        let host_config = VirtualHostConfig::builder()
            .hostname("localhost")
            .port(9000)
            .root_directory("tests")
            .security(security_config.clone())
            .status_pages(status_pages! {
               404 @ "tests/files/404.html".to_string()
            })
            .build()?;

        let mut virtual_host = VirtualHostImpl::new(host_config);
        virtual_host.add_path(StaticPath::new(
            StaticPathConfig::builder()
                .uri("/")
                .directory("tests/files")
                .build()?,
        ));

        let mut server = Vetis::new(config);
        server
            .add_virtual_host(virtual_host)
            .await;

        server
            .start()
            .await?;

        let client = deboa_tokio::Client::builder()
            .certificate(DeboaCertificate::from_slice(CA_CERT, ContentEncoding::DER))
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

/*
#[cfg(feature = "runtime-compio")]
mod compio {
    use crate::common::{CA_CERT, SERVER_CERT, SERVER_KEY};
    use caramelo::{expect, matchers::eq};
    use deboa::{
        cert::{CertificateExt as _, ContentEncoding},
        request,
    };
    use deboa_compio::{cert::DeboaCertificate, Client};
    use http::{StatusCode, Version};
    use std::error::Error;
    use vetis::{
        listener::ListenerConfig, security::SecurityConfig, server::ServerConfig,
        virtual_host::VirtualHostConfig, VetisServer,
    };
    use vetis_compio::{virtual_host::VirtualHostImpl, Protocol, Vetis};
    use vetis_macros::status_pages;
    use vetis_static::{compio::StaticPath, StaticPathConfig};

    #[compio::test]
    async fn test_index() -> Result<(), Box<dyn Error>> {
        let listener = ListenerConfig::builder()
            .port(9100)
            .protocol_version(Version::HTTP_11)
            .interface("0.0.0.0")
            .build()?;

        let config = ServerConfig::builder()
            .add_listener(listener)
            .build()?;

        let security_config = SecurityConfig::builder()
            .ca_cert_from_bytes(CA_CERT.to_vec())
            .cert_from_bytes(SERVER_CERT.to_vec())
            .key_from_bytes(SERVER_KEY.to_vec())
            .build()?;

        let host_config = VirtualHostConfig::builder()
            .hostname("localhost")
            .port(9100)
            .root_directory("tests")
            .security(security_config.clone())
            .build()?;

        let mut virtual_host = VirtualHostImpl::new(host_config);
        virtual_host.add_path(StaticPath::new(
            StaticPathConfig::builder()
                .uri("/")
                .directory("tests/files")
                .index_files(vec!["index.html".to_string()])
                .build()?,
        ));

        let mut server = Vetis::new(config);
        server
            .add_virtual_host(virtual_host)
            .await;

        server
            .start()
            .await?;

        let client = Client::builder()
            .certificate(DeboaCertificate::from_slice(CA_CERT, ContentEncoding::DER))
            .build();

        let request = request::get("https://localhost:9100/")?
            .send_with(&client)
            .await?;

        assert_eq!(request.status(), http::StatusCode::OK);

        let expected = if cfg!(windows) {
            "<html>\r\n<head>\r\n  <title>\r\n    Tested!\r\n  </title>\r\n</head>\r\n<body>\r\n  <p>\r\n    Tested!\r\n  </p>\r\n</body>\r\n</html>\n"
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

    #[compio::test]
    async fn test_not_found() -> Result<(), Box<dyn Error>> {
        let listener = ListenerConfig::builder()
            .port(9000)
            .protocol_version(Version::HTTP_11)
            .interface("0.0.0.0")
            .build()?;

        let config = ServerConfig::builder()
            .add_listener(listener)
            .build()?;

        let security_config = SecurityConfig::builder()
            .ca_cert_from_bytes(CA_CERT.to_vec())
            .cert_from_bytes(SERVER_CERT.to_vec())
            .key_from_bytes(SERVER_KEY.to_vec())
            .build()?;

        let host_config = VirtualHostConfig::builder()
            .hostname("localhost")
            .port(9000)
            .root_directory("tests")
            .security(security_config.clone())
            .status_pages(status_pages! {
               404 @ "tests/files/404.html".to_string()
            })
            .build()?;

        let mut virtual_host = VirtualHostImpl::new(host_config);
        virtual_host.add_path(StaticPath::new(
            StaticPathConfig::builder()
                .uri("/")
                .directory("tests/files")
                .build()?,
        ));

        let mut server = Vetis::new(config);
        server
            .add_virtual_host(virtual_host)
            .await;

        server
            .start()
            .await?;

        let client = Client::builder()
            .certificate(Certificate::from_slice(CA_CERT, ContentEncoding::DER))
            .build();

        let request = request::get("https://localhost:9000/some/file/here.txt")?
            .send_with(&client)
            .await?;

        expect(request.status()).to_be(eq(StatusCode::NOT_FOUND));

        server
            .stop()
            .await?;

        Ok(())
    }
}
*/
