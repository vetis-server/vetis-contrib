# vetis-wt

vetis-wt adds webtransport support to your vetis server!

## Features

- Protocol version 13

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
vetis-wt = "0.1.0"
```

## Configuration

Below you can find instruction on how to configure many logging options availble
on vetis.

### stdout

#### Using API

```rust,ignore

```

#### vetis.yaml

There's no plans to provide this kind of configuration method.

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
