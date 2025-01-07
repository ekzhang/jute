use std::{
    fs,
    path::Path,
    process::{exit, Command},
};

use jute::backend::{commands::RunCellEvent, notebook::NotebookRoot};
use ts_rs::TS;

fn main() {
    let export_path = Path::new("../src/bindings");

    // print the full path of the export directory
    println!(
        "Exporting TypeScript bindings to `{:?}`",
        fs::canonicalize(export_path).expect("Failed to get full path of export directory")
    );

    // Clear the `src/bindings` directory
    if export_path.exists() {
        println!("Clearing old bindings...");
        fs::remove_dir_all(export_path).expect("Failed to clear bindings directory");
    }

    fs::create_dir_all(export_path).expect("Failed to recreate bindings directory");

    // Generate TypeScript bindings
    println!("Exporting TypeScript bindings...");

    NotebookRoot::export_all_to(export_path).unwrap();
    RunCellEvent::export_all_to(export_path).unwrap();

    // Generate `index.ts` file
    println!("Generating index.ts...");
    let mut index_file = String::new();
    for entry in fs::read_dir(export_path).expect("Failed to read bindings directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        if let Some(extension) = path.extension() {
            if extension == "ts" {
                let file_name = path.file_stem().unwrap().to_string_lossy();
                index_file.push_str(&format!("export * from './{}';\n", file_name));
            }
        }
    }

    fs::write(export_path.join("index.ts"), index_file).expect("Failed to write index.ts");

    // Format the bindings with Prettier
    println!("Formatting with Prettier...");
    let status = Command::new("npx")
        .arg("prettier")
        .arg("--write")
        .arg(format!("{}/**/*.ts", export_path.display()))
        .status()
        .expect("Failed to run Prettier");

    if !status.success() {
        eprintln!("Prettier formatting failed");
        exit(1);
    }

    println!("Done!");
}
