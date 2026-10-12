#[cfg(test)]
mod tests {
    use log::Level;

    use crate::{FileLogConfig, StderrLogConfig, StdoutLogConfig};

    #[test]
    fn stdout_builder_keeps_log_level() {
        let config = StdoutLogConfig::builder()
            .log_level(Level::Debug)
            .build();

        assert_eq!(config.log_level, Level::Debug);
    }

    #[test]
    fn stderr_builder_keeps_log_level() {
        let config: StderrLogConfig = StderrLogConfig::builder()
            .log_level(Level::Warn)
            .build();

        assert_eq!(config.log_level, Level::Warn);
    }

    #[test]
    fn file_builder_keeps_values() {
        let config = FileLogConfig::builder()
            .log_level(Level::Trace)
            .log_path("/tmp/vetis.log")
            .roll_strategy("size")
            .size(1024)
            .build();

        assert_eq!(config.log_level, Level::Trace);
        assert_eq!(config.log_path, std::path::PathBuf::from("/tmp/vetis.log"));
        assert_eq!(&*config.roll_strategy, "size");
        assert_eq!(config.size, 1024);
    }
}
