# vetis-auth

[![Crates.io downloads](https://img.shields.io/crates/d/vetis-auth)](https://crates.io/crates/vetis-auth) [![crates.io](https://img.shields.io/crates/v/vetis-auth?style=flat-square)](https://crates.io/crates/vetis-auth) [![Build Status](https://github.com/vetis-server/vetis-auth/actions/workflows/rust.yml/badge.svg?event=push)](https://github.com/vetis-server/vetis-auth/actions/workflows/rust.yml) ![Crates.io MSRV](https://img.shields.io/crates/msrv/vetis-auth) [![Documentation](https://docs.rs/vetis-auth/badge.svg)](https://docs.rs/vetis-auth/latest/vetis-auth) [![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/vetis-server/vetis-auth/blob/main/LICENSE.md)  [![codecov](https://codecov.io/gh/vetis-server/vetis-auth/graph/badge.svg?token=T0HSBAPVSI)](https://codecov.io/gh/vetis-server/vetis-auth)

vetis-auth adds diffent auth types for your host path, choose the most appropriate one for your usecase!

## Features

- basic

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
vetis-auth = "0.1.0"
```

## Configuration

Below you can find instruction on how to configure many auth options availble
on vetis.

### stdout

#### Using API

```rust,ignore
use vetis_log::StdoutLogConfig;
use log::Level;

let config = StdoutLogConfig::builder()
    .log_leve(Level::Info)
    .build()?;
```

#### vetis.yaml

```yaml
dest: stdout
log_level: "INFO"
```

#### Configuration Options

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
