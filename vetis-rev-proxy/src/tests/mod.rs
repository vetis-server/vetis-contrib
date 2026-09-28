use crate::ReverseProxyPathConfig;
use vetis::VetisTestResult;

mod path;

#[test]
fn test_reverse_proxy_config() -> VetisTestResult<()> {
    let reverse_proxy_config = ReverseProxyPathConfig::builder()
        .uri("/")
        .target("http://localhost:8081")
        .build()?;
    assert_eq!(reverse_proxy_config.uri(), "/");
    assert_eq!(reverse_proxy_config.target(), "http://localhost:8081");
    Ok(())
}
