use crate::ReverseProxyPathConfig;
use vetis::{
    VetisTestResult,
    errors::{ConfigError, VetisError},
};

#[test]
fn test_proxy_path() -> VetisTestResult<()> {
    let some_path = ReverseProxyPathConfig::builder()
        .uri("/test")
        .target("http://localhost:8080")
        .build()?;

    assert_eq!(some_path.uri(), "/test");
    assert_eq!(some_path.target(), "http://localhost:8080");

    Ok(())
}

#[test]
fn test_invalid_proxy_path() -> VetisTestResult<()> {
    let some_path = ReverseProxyPathConfig::builder()
        .uri("")
        .target("http://localhost:8080")
        .build();

    assert!(some_path.is_err());
    assert_eq!(
        some_path.err(),
        Some(VetisError::Config(ConfigError::Path("URI cannot be empty".into(),)))
    );

    Ok(())
}

#[test]
fn test_invalid_proxy_path_target() -> VetisTestResult<()> {
    let some_path = ReverseProxyPathConfig::builder()
        .uri("/test")
        .target("")
        .build();

    assert!(some_path.is_err());
    assert_eq!(
        some_path.err(),
        Some(VetisError::Config(ConfigError::Path("Target cannot be empty".into(),)))
    );

    Ok(())
}
