use crate::StaticPathConfig;
use vetis::{
    VetisTestResult,
    errors::{ConfigError, VetisError},
};

#[test]
fn test_static_path_config() -> VetisTestResult<()> {
    let path_config = StaticPathConfig::builder()
        .uri("/test")
        .extensions(".html")
        .directory("./test")
        .build()?;

    assert_eq!(path_config.uri(), "/test");
    assert_eq!(path_config.directory(), "./test");
    assert_eq!(path_config.extensions(), ".html");

    Ok(())
}

#[test]
fn test_invalid_uri() {
    let some_path = StaticPathConfig::builder()
        .uri("")
        .build();

    assert!(some_path.is_err());
    assert_eq!(
        some_path.err(),
        Some(VetisError::Config(ConfigError::Path("URI cannot be empty".into(),)))
    );
}

#[test]
fn test_invalid_extensions() {
    let some_path = StaticPathConfig::builder()
        .uri("/test")
        .extensions("")
        .build();

    assert!(some_path.is_err());
    assert_eq!(
        some_path.err(),
        Some(VetisError::Config(ConfigError::Path("Extensions cannot be empty".into(),)))
    );
}

#[test]
fn test_invalid_directory() {
    let some_path = StaticPathConfig::builder()
        .uri("/test")
        .extensions(".html")
        .directory("")
        .build();

    assert!(some_path.is_err());
    assert_eq!(
        some_path.err(),
        Some(VetisError::Config(ConfigError::Path("Directory cannot be empty".into(),)))
    );
}
