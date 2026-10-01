use clap::Parser;
use std::{
    error::Error,
    net::{IpAddr, Ipv4Addr},
};
use tracing_subscriber::{filter::LevelFilter, fmt::format};
use vetis::{VetisServer, host::HostConfig};
use vetis_static::{StaticPathConfig, tokio::StaticPath};
use vetis_tokio::{Vetis, host::Host};

#[derive(Parser)]
#[command(
    name = "front",
    about = "front - a very tiny frontend server",
    long_about = r#"
front - a very tiny frontend server

Usage:
    front [OPTIONS]

Options:
    -h, --help       Print help information
    -V, --version    Print version information
    -i, --interface  <INTERFACE>
                     Interface to bind to
                     Default: 0.0.0.0
    -p, --port       <PORT>
                     Port to bind to
                     Default: 4444
    -r, --root       <ROOT>
                     Root directory to serve
                     Default: .
"#
)]
struct Args {
    #[arg(short, long, required = false, help = "Root directory to serve.")]
    root: Option<String>,
    #[arg(short, long, required = false, help = "Interface to bind to.")]
    interface: Option<String>,
    #[arg(short, long, required = false, help = "Port to bind to.")]
    port: Option<u16>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(LevelFilter::DEBUG)
        .event_format(format().compact())
        .with_target(false)
        .init();

    let root = &args
        .root
        .unwrap_or(".".to_string());
    let port = args
        .port
        .unwrap_or(4444);

    let host_config = HostConfig::builder()
        .hostname("localhost")
        .bind_addresses(&[(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port)])
        .root_directory(root)
        .build()?;

    let mut virtual_host = Host::new(host_config).await?;
    virtual_host.add_path(StaticPath::new(
        StaticPathConfig::builder()
            .uri("/")
            .directory(root)
            .extensions("\\.(html|js|css|svg|png|ico|jsx|ts|tsx|json|mjs)$")
            .index_files(vec!["index.html".into()])
            .build()?,
    ));

    let mut server = Vetis::builder()
        .add_host(virtual_host)
        .await?
        .build();

    server.run().await?;

    Ok(())
}
