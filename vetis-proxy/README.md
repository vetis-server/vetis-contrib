# vetis-proxy

Reverse proxy support for the vetis HTTP server framework. This module provides HTTP reverse proxy functionality to forward requests to backend servers.

## Features

- Forward HTTP requests to backend servers
- Configurable URI prefix and target URL
- Support for Tokio async runtime
- Integration with vetis virtual host configuration
- TLS support via rustls

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
vetis-proxy = "0.1.0"
```

### Runtime Features

Currently only Tokio runtime is supported:

- `runtime-tokio` (default): Use with Tokio runtime

Example:

```toml
[dependencies]
vetis-proxy = { version = "0.1.0", features = ["runtime-tokio"] }
```

## Configuration

### Builder Pattern

Configure reverse proxy using the builder pattern:

```rust,ignore
use vetis_proxy::ProxyPathConfig;

let config = ProxyPathConfig::builder()
    .uri("/api")
    .target("http://localhost:8080")
    .build()?;
```

### Deserialization from Configuration

Configure via deserialization (e.g., from YAML):

#### YAML Example

```yaml
type: ProxyPathConfig
uri: /api
target: http://localhost:8080
```

## Configuration Options

### ProxyPathConfig

- **uri**: The URI prefix for the proxy path (required)
- **target**: The target URL to forward requests to (required)

## Runtime-Specific Modules

Import the Tokio runtime module:

```rust,ignore
// For Tokio runtime
#[cfg(feature = "runtime-tokio")]
use vetis_proxy::tokio;
```

## License

Licensed under either of

- Apache License, Version 2.0
  (LICENSE-APACHE or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license
  (LICENSE-MIT or <https://opensource.org/licenses/MIT>)

at your option.

## Author

Rogerio Pereira Araujo <rogerio.araujo@gmail.com>
