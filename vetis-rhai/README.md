# vetis-rhai

vetis-rhai added rhai scripting support to vetis server.

## Features

- Serve static files with configurable extensions and directories

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
vetis-rhai = "0.1.0"
```

### Runtime Features

Choose the appropriate runtime feature for your project:

- `runtime-tokio` (default): Use with Tokio runtime

## Configuration

### Using API

Configure static file serving using the builder pattern:

```rust,ignore
use vetis_rhai::RhaiPathConfig;

let config = RhaiPathConfig::builder()
    .uri("/script")
    .script_path("index.rhai")
    .build()?;
```

### vetis.yaml

```yaml
type: rhai_path
uri: /hello
script_path: index.rhai
```

### Configuration Options

#### rhai_path

- **script_path**: The path for rhai script file

## License

Licensed under either of

- Apache License, Version 2.0
  (LICENSE-APACHE or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license
  (LICENSE-MIT or <https://opensource.org/licenses/MIT>)

at your option.

## Author

Rogerio Pereira Araujo <rogerio.araujo@gmail.com>
