//! An example that lists all available kernels.

use jute::server::environment;

#[tokio::main]
async fn main() {
    println!("Available kernels:");
    for (path, kernel_spec) in environment::list_kernels(None).await {
        println!("  {:20} {}", kernel_spec.display_name, path.display());
    }
}
