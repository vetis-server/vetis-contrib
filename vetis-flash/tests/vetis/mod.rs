#[cfg(feature = "runtime-tokio")]
mod tokio {
    use crate::common::CA_CERT;
    use caramelo::{expect, matchers::eq};
    use deboa::{
        cert::{CertificateExt as _, ContentEncoding},
        request,
    };
    use deboa_tokio::cert::DeboaCertificate;
    use http::Version;
    use serde::Deserialize;
    use std::path::Path;
    use tokio::fs::read_to_string;
    use vetis::{VetisServer as _, VetisTestResult};
    #[allow(unused_imports)]
    use vetis_flash::FlashPathConfig;
    use vetis_tokio::{ServerConfig, Vetis};

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
        let config_path = Path::new("../files/vetis-flash.yaml");
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
                    expect(host.paths().len()).to_be(eq(2));

                    let static_path_config = &host.paths()[0];
                    let static_path = static_path_config.boxed_path();
                    expect(static_path.uri()).to_be(eq("/"));

                    let mut server = Vetis::from_config(config.server).await?;
                    server
                        .start()
                        .await?;

                    let cert = tokio::fs::read(format! {"../{CA_CERT}"}).await?;
                    let client = deboa_tokio::Client::builder()
                        .certificate(DeboaCertificate::from_slice(&cert, ContentEncoding::DER))
                        .build();

                    let request = request::get("https://localhost:8443/")?
                        .version(Version::HTTP_2)
                        .send_with(&client)
                        .await?;

                    assert_eq!(request.status(), http::StatusCode::OK);
                    assert_eq!(
                        request
                            .text()
                            .await?,
                        "Hola VeTiS!"
                    );

                    let request = request::get("https://localhost:8443/template")?
                        .version(Version::HTTP_2)
                        .send_with(&client)
                        .await?;

                    assert_eq!(request.status(), http::StatusCode::OK);
                    assert_eq!(
                        request
                            .text()
                            .await?,
                        "\n<html>\n\n<head>\n    <title>\n        Vetis!\n    </title>\n</head>\n\n<body>\n\n</body>\n\n</html>"
                    );

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
}
