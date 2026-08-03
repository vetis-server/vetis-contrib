# vetis-static

Static file serving support for the vetis HTTP server framework. This module provides efficient static file serving with configurable caching, support for multiple async runtimes, and flexible path configuration.

## Features

- Serve static files with configurable extensions and directories
- Built-in caching with TTL (time-to-live) and TTI (time-to-idle) support
- Support for multiple async runtimes: Tokio, Smol, and Compio
- Configurable index files for directory requests
- MIME type detection
- ETag support for cache validation
- Memory and disk-based file storage options

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
vetis-static = "0.1.0-beta.1"
```

### Runtime Features

Choose the appropriate runtime feature for your project:

- `runtime-tokio` (default): Use with Tokio runtime
- `runtime-smol`: Use with Smol runtime
- `runtime-compio`: Use with Compio runtime

Example:

```toml
[dependencies]
vetis-static = { version = "0.1.0-beta.1", features = ["runtime-tokio"] }
```

## Configuration

### Builder Pattern

Configure static file serving using the builder pattern:

```rust,ignore
use vetis_static::StaticPathConfig;
use std::time::Duration;

let config = StaticPathConfig::builder()
    .uri("/static")
    .extensions(r"\.(html|css|js|png|jpg)$")
    .directory("./public")
    .index_files(vec!["index.html".to_string()])
    .cache(
        vetis_static::StaticPathCache::builder()
            .max_file_size(10 * 1024 * 1024) // 10MB
            .ttl(Duration::from_secs(60))
            .tti(Duration::from_secs(10))
            .capacity(1000)
            .build()
    )
    .build()?;
```

### Deserialization from Configuration

Configure via deserialization (e.g., from YAML):

#### YAML Example

```yaml
type: StaticPathConfig
uri: /static
extensions: \.(html|css|js|png|jpg)$
directory: ./public
index_files: [index.html]
cache:
    max_file_size: 10485760
    ttl: 60s
    tti: 10s
    capacity: 1000
```

## Configuration Options

### StaticPathConfig

- **uri**: The URI prefix for serving static files (required)
- **extensions**: Comma-separated list of file extensions to serve (required)
- **directory**: The filesystem directory containing static files (required)
- **index_files**: Optional list of index file names to serve for directory requests
- **cache**: Optional cache configuration

### StaticPathCache

- **max_file_size**: Maximum file size to cache in bytes (default: 10MB)
- **ttl**: Time-to-live for cached entries (default: 60s)
- **tti**: Time-to-idle for cached entries (default: 10s)
- **capacity**: Maximum number of cached files (default: 1000)

Duration fields support the string format from the `duration-str` crate (e.g., "60s", "5m", "1h").

## Runtime-Specific Modules

Based on your selected runtime feature, import the appropriate module:

```rust
// For Tokio runtime
#[cfg(feature = "runtime-tokio")]
use vetis_static::tokio;

// For Smol runtime
#[cfg(feature = "runtime-smol")]
use vetis_static::smol;

// For Compio runtime
#[cfg(feature = "runtime-compio")]
use vetis_static::compio;
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
