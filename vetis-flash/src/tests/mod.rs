use deboa::TestResult;

use crate::FlashPathConfig;

#[test]
fn test_static_files_config() -> TestResult<()> {
    let static_files_config = FlashPathConfig::builder()
        .uri("/static")
        .response("Welcome to VeTiS!")
        .status_code(200)
        .build()?;
    assert_eq!(static_files_config.uri(), "/static");
    assert_eq!(static_files_config.response(), &Some("Welcome to VeTiS!".to_owned()));
    assert_eq!(static_files_config.status_code(), 200);
    Ok(())
}
