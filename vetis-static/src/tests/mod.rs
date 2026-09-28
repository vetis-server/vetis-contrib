use crate::StaticPathConfig;
use vetis::VetisTestResult;

mod path;

#[test]
fn test_static_files_config() -> VetisTestResult<()> {
    let static_files_config = StaticPathConfig::builder()
        .uri("/static")
        .extensions("html,css,js")
        .directory("/var/vetis/www")
        .index_files(vec!["index.html".to_string(), "index.htm".to_string()])
        .build()?;
    assert_eq!(static_files_config.uri(), "/static");
    assert_eq!(static_files_config.extensions(), "html,css,js");
    assert_eq!(static_files_config.directory(), "/var/vetis/www");
    assert_eq!(
        static_files_config.index_files(),
        &Some(vec!["index.html".to_string(), "index.htm".to_string()])
    );
    Ok(())
}
