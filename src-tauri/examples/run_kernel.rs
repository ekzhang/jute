//! Shell example of running a kernel from Rust code.

use std::io::Write;

use jute::backend::{
    commands::{self, RunCellEvent},
    local::{environment, LocalKernel},
};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    println!("Available kernels:");

    let kernels = environment::list_kernels(None).await;
    for (path, kernel_spec) in &kernels {
        println!("  {:20} {}", kernel_spec.display_name, path.display());
    }

    let mut kernel_spec = loop {
        print!("\nPick a kernel: ");
        std::io::stdout().flush().unwrap();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        match kernels.iter().find(|(_, spec)| spec.display_name == input) {
            Some((_, kernel_spec)) => break kernel_spec.clone(),
            None => println!("Invalid kernel name, try again"),
        }
    };

    if kernel_spec.argv[0] == "python" {
        // Temporary hack
        kernel_spec.argv[0] = "python3.11".into();
    }

    let mut kernel = LocalKernel::start(&kernel_spec).await.unwrap();

    println!("\nStarted kernel.");

    let info = commands::kernel_info(kernel.conn()).await.unwrap();
    println!("{}", info.banner);

    while kernel.is_alive() {
        print!("> ");
        std::io::stdout().flush().unwrap();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        let rx = commands::run_cell(kernel.conn(), &input).await.unwrap();

        while let Ok(event) = rx.recv().await {
            match event {
                RunCellEvent::Stdout(text) => print!("{}", text),
                RunCellEvent::Stderr(text) => eprint!("{}", text),
                RunCellEvent::ExecuteResult(msg) => {
                    println!("-> {}", msg.data["text/plain"].as_str().unwrap())
                }
                RunCellEvent::Error(msg) => {
                    for line in &msg.traceback {
                        eprintln!("{line}");
                    }
                }
                RunCellEvent::Disconnect(msg) => {
                    eprintln!("Kernel disconnected abnormally: {}", msg);
                    break;
                }
            }
        }
    }
}
