#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
use log::Level;
use logforth::{
    Diagnostic, Filter,
    append::{
        self,
        file::FileBuilder,
        syslog::{
            SyslogBuilder,
            fasyslog::sender::{SyslogSender::UnixStream, UnixStreamSender},
        },
    },
    core::DispatchBuilder,
    filter::FilterResult,
    layout::TextLayout,
    record::{self, FilterCriteria, Record},
};
use serde::{Deserialize, Serialize};
use std::{num::NonZeroUsize, ops::Deref, path::PathBuf, sync::Arc};
use vetis::log::LogConfig;

#[derive(Debug)]
/// TargetFilter is used to dispatch by target
pub struct TargetFilter {
    target: String,
    level: Level,
}

impl TargetFilter {
    /// Create a net TargetFilter
    pub fn new(target: &str, level: Level) -> Self {
        Self { target: target.into(), level }
    }
}

impl Filter for TargetFilter {
    fn enabled(&self, criteria: &FilterCriteria, _: &[Box<dyn Diagnostic>]) -> FilterResult {
        let log_level = match self.level {
            Level::Error => record::Level::Error,
            Level::Warn => record::Level::Warn,
            Level::Info => record::Level::Info,
            Level::Debug => record::Level::Debug,
            Level::Trace => record::Level::Trace,
        };

        if criteria.target() == self.target && criteria.level() >= log_level {
            FilterResult::Accept
        } else {
            FilterResult::Reject
        }
    }

    fn matches(&self, record: &Record, diags: &[Box<dyn Diagnostic>]) -> FilterResult {
        let criteria = FilterCriteria::builder()
            .level(record.level())
            .target(record.target())
            .build();

        self.enabled(&criteria, diags)
    }
}

/// Builder for creating `StdoutLogConfig` instances.
pub struct StdoutLogConfigBuilder {
    log_level: Level,
}

impl StdoutLogConfigBuilder {
    /// Allow set the log level.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn log_level(mut self, level: Level) -> Self {
        self.log_level = level;
        self
    }

    /// Build StdoutLogConfig instance
    pub fn build(self) -> StdoutLogConfig {
        StdoutLogConfig { log_level: self.log_level }
    }
}

/// Configuration for stdout logging.
#[derive(Clone, Serialize, Deserialize)]
pub struct StdoutLogConfig {
    log_level: Level,
}

impl Default for StdoutLogConfig {
    fn default() -> Self {
        Self { log_level: Level::Info }
    }
}

#[typetag::serde(name = "stdout")]
impl LogConfig for StdoutLogConfig {
    fn log_level(&mut self, log_level: Level) {
        self.log_level = log_level;
    }

    fn boxed_clone(&self) -> Box<dyn LogConfig> {
        Box::new(self.clone())
    }

    fn into_builder(&self, target: &str, input: DispatchBuilder<false>) -> DispatchBuilder<true> {
        input
            .filter(TargetFilter::new(target, self.log_level))
            .append(append::Stdout::default().with_layout(TextLayout::default()))
    }
}

impl StdoutLogConfig {
    /// Allow create a new `StdoutLogConfigBuilder` with default settings.
    ///
    /// # Returns
    ///
    /// * `StdoutLogConfigBuilder` - The builder.
    pub fn builder() -> StdoutLogConfigBuilder {
        StdoutLogConfigBuilder { log_level: Level::Info }
    }
}

/// Builder for creating `StderrLogConfig` instances.
pub struct StderrLogConfigBuilder {
    log_level: Level,
}

impl StderrLogConfigBuilder {
    /// Allow set the log level.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn log_level(mut self, level: Level) -> Self {
        self.log_level = level;
        self
    }

    /// Build StderrLogConfig instance
    pub fn build(self) -> StderrLogConfig {
        StderrLogConfig { log_level: self.log_level }
    }
}

/// Configuration for stderr logging.
#[derive(Clone, Serialize, Deserialize)]
pub struct StderrLogConfig {
    log_level: Level,
}

impl Default for StderrLogConfig {
    fn default() -> Self {
        Self { log_level: Level::Info }
    }
}

#[typetag::serde(name = "stderr")]
impl LogConfig for StderrLogConfig {
    fn log_level(&mut self, log_level: Level) {
        self.log_level = log_level;
    }

    fn boxed_clone(&self) -> Box<dyn LogConfig> {
        Box::new(self.clone())
    }

    fn into_builder(&self, target: &str, input: DispatchBuilder<false>) -> DispatchBuilder<true> {
        input
            .filter(TargetFilter::new(target, self.log_level))
            .append(append::Stdout::default().with_layout(TextLayout::default()))
    }
}

impl StderrLogConfig {
    /// Allow create a new `StderrLogConfigBuilder` with default settings.
    ///
    /// # Returns
    ///
    /// * `StderrLogConfigBuilder` - The builder.
    pub fn builder() -> StdoutLogConfigBuilder {
        StdoutLogConfigBuilder { log_level: Level::Info }
    }
}

/// Builder for creating `SysLogConfig` instances.
pub struct SysLogConfigBuilder {
    log_level: Level,
}

impl SysLogConfigBuilder {
    /// Allow set the log level.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn log_level(mut self, level: Level) -> Self {
        self.log_level = level;
        self
    }

    /// Build SysLogConfig instance
    pub fn build(self) -> SysLogConfig {
        SysLogConfig { log_level: self.log_level, socket_path: "/var/log/syslog".into() }
    }
}

/// Configuration for stderr logging.
#[derive(Clone, Serialize, Deserialize)]
pub struct SysLogConfig {
    log_level: Level,
    // TODO: Expand to beyond local syslog
    socket_path: String,
}

impl Default for SysLogConfig {
    fn default() -> Self {
        Self { log_level: Level::Info, socket_path: "/var/log/syslog".into() }
    }
}

#[typetag::serde(name = "syslog")]
impl LogConfig for SysLogConfig {
    fn log_level(&mut self, log_level: Level) {
        self.log_level = log_level;
    }

    fn boxed_clone(&self) -> Box<dyn LogConfig> {
        Box::new(self.clone())
    }

    fn into_builder(&self, target: &str, input: DispatchBuilder<false>) -> DispatchBuilder<true> {
        let sender =
            UnixStreamSender::connect(&self.socket_path).expect("Couldt not connect to socket!");
        let syslog = SyslogBuilder::new(UnixStream(sender))
            .layout(TextLayout::default())
            .build();
        input
            .filter(TargetFilter::new(target, self.log_level))
            .append(syslog)
    }
}

impl SysLogConfig {
    /// Allow create a new `SysLogConfig` with default settings.
    ///
    /// # Returns
    ///
    /// * `SysLogConfig` - The builder.
    pub fn builder() -> SysLogConfigBuilder {
        SysLogConfigBuilder { log_level: Level::Info }
    }
}

/// Builder for creating `FileLogConfig` instances.
pub struct FileLogConfigBuilder {
    log_level: Level,
    log_path: PathBuf,
    roll_strategy: Arc<str>,
    size: usize,
}

impl FileLogConfigBuilder {
    /// Allow set the log level.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn log_level(mut self, level: Level) -> Self {
        self.log_level = level;
        self
    }

    /// Allow set log path.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn log_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.log_path = path.into();
        self
    }

    /// Allow set roll strategy.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn roll_strategy(mut self, strategy: &str) -> Self {
        self.roll_strategy = strategy.into();
        self
    }

    /// Allow set the lo size for size based roll strategy.
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn size(mut self, size: usize) -> Self {
        self.size = size;
        self
    }

    /// Build SysLogConfig instance
    pub fn build(self) -> FileLogConfig {
        FileLogConfig {
            log_level: self.log_level,
            log_path: self.log_path,
            roll_strategy: self.roll_strategy,
            size: self.size,
        }
    }
}

/// Configuration for file logging.
#[derive(Clone, Serialize, Deserialize)]
pub struct FileLogConfig {
    log_level: Level,
    log_path: PathBuf,
    roll_strategy: Arc<str>,
    size: usize,
}

impl Default for FileLogConfig {
    fn default() -> Self {
        Self {
            log_level: Level::Info,
            log_path: ".".into(),
            roll_strategy: "daily".into(),
            size: 0,
        }
    }
}

#[typetag::serde(name = "file")]
impl LogConfig for FileLogConfig {
    fn log_level(&mut self, log_level: Level) {
        self.log_level = log_level;
    }

    fn boxed_clone(&self) -> Box<dyn LogConfig> {
        Box::new(self.clone())
    }

    fn into_builder(&self, target: &str, input: DispatchBuilder<false>) -> DispatchBuilder<true> {
        let mut builder = FileBuilder::new(&self.log_path, target)
            .layout(TextLayout::default().no_color())
            .filename_suffix("log");

        builder = match self
            .roll_strategy
            .deref()
        {
            "daily" => builder.rollover_daily(),
            "hourly" => builder.rollover_hourly(),
            "minutelly" => builder.rollover_minutely(),
            "size" if let Some(size) = NonZeroUsize::new(self.size) => builder.rollover_size(size),
            &_ => panic!("Invalid roll strategy"),
        };

        match builder.build() {
            Ok(file) => input
                .filter(TargetFilter::new(target, self.log_level))
                .append(file),
            Err(e) => panic!("Could not initialize file appender: {}", e.to_string()),
        }
    }
}

impl FileLogConfig {
    /// Allow create a new `FileLogConfig` with default settings.
    ///
    /// # Returns
    ///
    /// * `SysLogConfig` - The builder.
    pub fn builder() -> FileLogConfigBuilder {
        FileLogConfigBuilder {
            log_level: Level::Info,
            log_path: ".".into(),
            roll_strategy: "daily".into(),
            size: 0,
        }
    }
}
