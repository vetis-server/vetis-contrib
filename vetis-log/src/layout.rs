#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]

pub extern crate jiff;

use std::fmt::Write;

use jiff::Timestamp;
use jiff::tz::TimeZone;
use logforth_core::Diagnostic;
use logforth_core::Error;
use logforth_core::kv::KeyView;
use logforth_core::kv::ValueView;
use logforth_core::kv::Visitor;
use logforth_core::layout::Layout;
use logforth_core::record::Record;

/// A layout that formats log record as optionally colored text.
///
/// Output format:
///
/// ```text
/// 2024-08-11T22:44:57.172105+08:00 ERROR server: Hello error!
/// 2024-08-11T22:44:57.172219+08:00  WARN server: Hello warn!
/// 2024-08-11T22:44:57.172276+08:00  INFO server: Hello info!
/// 2024-08-11T22:44:57.172329+08:00 DEBUG server: Hello debug!
/// 2024-08-11T22:44:57.172382+08:00 TRACE server: Hello trace!
/// ```
///
/// You can customize the timezone of the timestamp by setting the `tz` field with a [`TimeZone`]
/// instance. Otherwise, the system timezone is used.
///
/// # Examples
///
/// ```
/// use logforth_layout_text::TextLayout;
///
/// let layout = TextLayout::default();
/// ```
#[derive(Debug, Clone)]
pub struct TextLayout {
    timezone: TimeZone,
    timestamp_format: Option<fn(Timestamp, &TimeZone) -> String>,
}

impl Default for TextLayout {
    fn default() -> Self {
        Self { timezone: TimeZone::system(), timestamp_format: None }
    }
}

impl TextLayout {
    /// Set the timezone for timestamps.
    ///
    /// Defaults to the system timezone if not set.
    ///
    /// # Examples
    ///
    /// ```
    /// use jiff::tz::TimeZone;
    /// use logforth_layout_text::TextLayout;
    ///
    /// let layout = TextLayout::default().timezone(TimeZone::UTC);
    /// ```
    pub fn timezone(mut self, tz: TimeZone) -> Self {
        self.timezone = tz;
        self
    }

    /// Set a user-defined timestamp format function.
    ///
    /// Default to formatting the timestamp with offset as ISO 8601. See the example below.
    ///
    /// For other formatting options, refer to the [jiff::fmt::strtime] documentation.
    ///
    /// # Examples
    ///
    /// ```
    /// use jiff::Timestamp;
    /// use jiff::tz::TimeZone;
    /// use logforth_layout_text::TextLayout;
    ///
    /// // This is equivalent to the default timestamp format.
    /// let layout = TextLayout::default()
    ///     .timestamp_format(|ts, tz| format!("{:.6}", ts.display_with_offset(tz.to_offset(ts))));
    /// ```
    pub fn timestamp_format(mut self, format: fn(Timestamp, &TimeZone) -> String) -> Self {
        self.timestamp_format = Some(format);
        self
    }
}

struct KvWriter {
    text: String,
}

impl Visitor for KvWriter {
    fn visit(&mut self, key: KeyView, value: ValueView) -> Result<(), Error> {
        use std::fmt::Write;

        // SAFETY: write to a string always succeeds
        write!(&mut self.text, " {key}={value}").unwrap();
        Ok(())
    }
}

fn default_timestamp_format(ts: Timestamp, tz: &TimeZone) -> String {
    let offset = tz.to_offset(ts);
    format!("{:.6}", ts.display_with_offset(offset))
}

impl Layout for TextLayout {
    fn format(&self, record: &Record, diags: &[Box<dyn Diagnostic>]) -> Result<Vec<u8>, Error> {
        // SAFETY: jiff::Timestamp::try_from only fails if the time is out of range, which is
        // very unlikely if the system clock is correct.
        let ts = Timestamp::try_from(record.time()).unwrap();
        let time = if let Some(format) = self.timestamp_format {
            format(ts, &self.timezone)
        } else {
            default_timestamp_format(ts, &self.timezone)
        };

        let level = record
            .level()
            .to_string();
        let target = record.target();
        let message = record.payload();

        let mut visitor = KvWriter { text: time };
        write!(&mut visitor.text, " {level:>6} {target}: {message}").unwrap();
        record
            .key_values()
            .visit(&mut visitor)?;
        for d in diags {
            d.visit(&mut visitor)?;
        }

        Ok(visitor
            .text
            .into_bytes())
    }
}
