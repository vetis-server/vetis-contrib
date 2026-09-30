# vetis-log

vetis-log adds several different log strategies to vetis server, feel free to choose the one which best suit your needs!

## Features

- stdout
- stderr
- file with rolling strategies
- syslog
- journald

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
vetis-log = "0.1.0"
```

## Configuration

Below you can find instruction on how to configure many logging options availble
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

### stderr

```yaml
dest: stderr
log_level: "INFO"
```

#### Configuration Options

- **log_level**: Log level: TRACE, WARN, DEBUG, INFO, ERROR and FATAL

### file

```yaml
dest: stdout
log_level: "INFO"
```

#### Configuration Options

- **log_level**: Log level: TRACE, WARN, DEBUG, INFO, ERROR and FATAL

### syslog

```yaml
dest: syslog
log_level: "INFO"
```

#### Configuration Options

- **log_level**: Log level: TRACE, WARN, DEBUG, INFO, ERROR and FATAL

### journald

```yaml
dest: journald
log_level: "INFO"
```

#### Configuration Options

- **log_level**: Log level: TRACE, WARN, DEBUG, INFO, ERROR and FATAL

### opentelemetry

```yaml
dest: opentelemetry
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
