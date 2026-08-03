use crate::ProxyPathConfig;

mod path;

#[test]
fn test_reverse_proxy_config() -> Result<(), Box<dyn std::error::Error>> {
    let reverse_proxy_config = ProxyPathConfig::builder()
        .uri("/")
        .target("http://localhost:8081")
        .build()?;
    assert_eq!(reverse_proxy_config.uri(), "/");
    assert_eq!(reverse_proxy_config.target(), "http://localhost:8081");
    Ok(())
}
