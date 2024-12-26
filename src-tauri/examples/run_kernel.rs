//! Shell example of running a kernel from Rust code.

use std::io::Write;

use jute::backend::{
    local::{environment, LocalKernel},
    wire_protocol::{
        ErrorReply, ExecuteRequest, ExecuteResult, KernelInfoReply, KernelInfoRequest,
        KernelMessage, KernelMessageType, KernelStatus, Reply, Status, Stream,
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

    {
        let conn = kernel.conn();
        let mut req = conn
            .call_shell(KernelMessage::new(
                KernelMessageType::KernelInfoRequest,
                KernelInfoRequest {},
            ))
            .await
            .unwrap();
        let msg = req.get_reply::<KernelInfoReply>().await.unwrap();
        if let Reply::Ok(info) = &msg.content {
            println!("{}", info.banner);
        }
    }

    while kernel.is_alive() {
        print!("> ");
        std::io::stdout().flush().unwrap();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        let conn = kernel.conn();
        while conn.try_recv_iopub().is_some() {}

        conn.call_shell(KernelMessage::new(
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
                    let msg = msg.into_typed::<Status>().unwrap();
                    // println!("Kernel status: {:?}", msg.content.execution_state);
                    status = msg.content.execution_state;
                }
                KernelMessageType::Stream => {
                    let msg = msg.into_typed::<Stream>().unwrap();
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
                    let msg = msg.into_typed::<ExecuteResult>().unwrap();
                    println!("-> {}", msg.content.data["text/plain"].as_str().unwrap());
                }
                KernelMessageType::Error => {
                    let msg = msg.into_typed::<ErrorReply>().unwrap();
                    for line in &msg.content.traceback {
                        println!("{line}");
                    }
                }
                _ => (),
            }
        }
    }
}
