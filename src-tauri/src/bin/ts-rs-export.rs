use std::{
    env, fs,
    path::Path,
    process::{exit, Command},
};

use ts_rs::TS;

fn main() {
    // Set TS_RS_EXPORT_DIR environment variable
    let export_dir = "src/bindings";
    env::set_var("TS_RS_EXPORT_DIR", export_dir);

    let export_path = Path::new(export_dir);

    // Clear the `src/bindings` directory
    if export_path.exists() {
        println!("Clearing old bindings...");
        fs::remove_dir_all(export_path).expect("Failed to clear bindings directory");
    }

    fs::create_dir_all(export_path).expect("Failed to recreate bindings directory");

    // Generate TypeScript bindings
    println!("Exporting TypeScript bindings...");

    // generate bindings

    // Format the bindings with Prettier
    println!("Formatting with Prettier...");
    let status = Command::new("npx")
        .arg("prettier")
        .arg("--write")
        .arg(format!("{}/**/*.ts", export_dir))
        .status()
        .expect("Failed to run Prettier");

    if !status.success() {
        eprintln!("Prettier formatting failed");
        exit(1);
    }

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

    println!("Done!");
}
