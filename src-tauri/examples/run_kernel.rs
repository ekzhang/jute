//! An example that lists all available kernels.

use std::io::Write;

use jute::{
    server::{environment, kernel::LocalKernel},
    wire_protocol::{
        ErrorReply, ExecuteRequest, ExecuteResult, KernelMessage, KernelMessageType, KernelStatus,
        Status, Stream,
    },
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

    while kernel.is_alive() {
        print!("> ");
        std::io::stdout().flush().unwrap();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        let conn = kernel.conn();

        conn.send_shell(KernelMessage::new(
            KernelMessageType::ExecuteRequest,
            ExecuteRequest {
                code: input,
                silent: false,
                store_history: true,
                user_expressions: Default::default(),
                allow_stdin: false,
                stop_on_error: true,
            },
        ))
        .await
        .unwrap();

        let mut status = KernelStatus::Busy;
        while status != KernelStatus::Idle {
            let msg = conn.recv_iopub().await.unwrap();
            match msg.header.msg_type {
                KernelMessageType::Status => {
                    let msg: KernelMessage<Status> = msg.into_typed().unwrap();
                    // println!("Kernel status: {:?}", msg.content.execution_state);
                    status = msg.content.execution_state;
                }
                KernelMessageType::Stream => {
                    let msg: KernelMessage<Stream> = msg.into_typed().unwrap();
                    if msg.content.name == "stdout" {
                        print!("{}", msg.content.text);
                    } else {
                        eprint!("{}", msg.content.text);
                    }
                }
                // KernelMessageType::ExecuteInput => {
                //     let msg: KernelMessage<ExecuteInput> = msg.into_typed().unwrap();
                //     println!("Kernel is executing: {}", msg.content.code);
                // }
                KernelMessageType::ExecuteResult => {
                    let msg: KernelMessage<ExecuteResult> = msg.into_typed().unwrap();
                    println!("-> {}", msg.content.data["text/plain"].as_str().unwrap());
                }
                KernelMessageType::Error => {
                    let msg: KernelMessage<ErrorReply> = msg.into_typed().unwrap();
                    for line in &msg.content.traceback {
                        println!("{line}");
                    }
                }
                _ => (),
            }
        }
    }
}
