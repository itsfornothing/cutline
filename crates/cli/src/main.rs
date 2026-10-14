//! Cutline CLI entrypoint

#[tokio::main]
async fn main() {
    println!("Cutline CLI v{}", env!("CARGO_PKG_VERSION"));
}
