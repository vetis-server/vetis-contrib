# vetis-otel

[![Crates.io downloads](https://img.shields.io/crates/d/vetis-log)](https://crates.io/crates/vetis-log) [![crates.io](https://img.shields.io/crates/v/vetis-log?style=flat-square)](https://crates.io/crates/vetis-log) [![Build Status](https://github.com/vetis-server/vetis-log/actions/workflows/rust.yml/badge.svg?event=push)](https://github.com/vetis-server/vetis-log/actions/workflows/rust.yml) ![Crates.io MSRV](https://img.shields.io/crates/msrv/vetis-log) [![Documentation](https://docs.rs/vetis-log/badge.svg)](https://docs.rs/vetis-log/latest/vetis-log) [![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/vetis-server/vetis-log/blob/main/LICENSE.md)  [![codecov](https://codecov.io/gh/vetis-server/vetis-log/graph/badge.svg?token=T0HSBAPVSI)](https://codecov.io/gh/vetis-server/vetis-contrib)

vetis-otel adds opentelemetry support to your vetis server!

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
vetis-otel = "0.1.0"
```

## Configuration

Below you can find instruction on how to configure many logging options availble
on vetis.

### Using API

```rust,ignore
use vetis_log::StdoutLogConfig;
use log::Level;

let config = StdoutLogConfig::builder()
    .log_leve(Level::Info)
    .build()?;
```

### vetis.yaml

```yaml
dest: stdout
log_level: "INFO"
```

### Configuration Options

- **log_level**: Log level: TRACE, WARN, DEBUG, INFO, ERROR and FATAL

### Runtime Features

Choose the appropriate runtime feature for your project:

- `runtime-tokio` (default): Use with Tokio runtime

## License

Licensed under either of

- Apache License, Version 2.0
  (LICENSE-APACHE or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license
  (LICENSE-MIT or <https://opensource.org/licenses/MIT>)

at your option.

## Author

Rogerio Pereira Araujo <rogerio.araujo@gmail.com>
