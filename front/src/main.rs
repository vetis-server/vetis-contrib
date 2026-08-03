use clap::Parser;
use http::Version;
use std::error::Error;
use terminal_link::Link;
use vetis::{
    listener::ListenerConfig, server::ServerConfig, virtual_host::VirtualHostConfig, VetisServer,
};
use vetis_static::{tokio::StaticPath, StaticPathConfig};
use vetis_tokio::{virtual_host::VirtualHostImpl, Vetis};

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

    env_logger::Builder::from_env(env_logger::Env::default().filter_or("RUST_LOG", "info"))
        .format_module_path(false)
        .init();

    let root = &args
        .root
        .unwrap_or(".".to_string());
    let interface = args
        .interface
        .unwrap_or("0.0.0.0".to_string());
    let port = args
        .port
        .unwrap_or(4444);

    let listener = ListenerConfig::builder()
        .port(port)
        .protocol_version(Version::HTTP_11)
        .interface(&interface)
        .build()?;

    let config = ServerConfig::builder()
        .add_listener(listener)
        .build()?;

    let host_config = VirtualHostConfig::builder()
        .hostname("localhost")
        .port(port)
        .root_directory(root)
        .build()?;

    let mut virtual_host = VirtualHostImpl::new(host_config);
    virtual_host.add_path(StaticPath::new(
        StaticPathConfig::builder()
            .uri("/")
            .directory(root)
            .extensions("\\.(html|js|css|svg|png|ico|jsx|ts|tsx|json|mjs)$")
            .index_files(vec!["index.html".to_string()])
            .build()?,
    ));

    let mut server = Vetis::new(config);
    server
        .add_virtual_host(virtual_host)
        .await;

    println!(
        "front is serving {} on {}\n",
        root,
        Link::new(&format!("http://localhost:{}", port), &format!("http://localhost:{}", port))
    );

    server.run().await?;

    Ok(())
}
