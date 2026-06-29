// src/bin/nxvm.rs
use nexuslang::vm;
use clap::Parser;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(name = "nxvm", about = "Nexus Virtual Machine", version)]
struct Args {
    /// Bytecode file to run (.nxb)
    #[arg(required = true)]
    input: String,
    
    /// Run all tests in the file
    #[arg(short, long)]
    utest: bool,
}

fn main() {
    let args = Args::parse();
    
    if args.utest {
        println!("Running tests in {}...", args.input);
        // Logic for testing would go here
        return;
    }

    let path = Path::new(&args.input);
    if !path.exists() {
        eprintln!("Error: File '{}' not found", args.input);
        return;
    }

    println!("Running {} in Nexus VM...", args.input);

    match vm::load_bytecode(&args.input) {
        Ok(instructions) => {
            let mut vm_instance = vm::VM::new();
            match vm_instance.run(&instructions) {
                Ok(code) => println!("Program finished with exit code: {}", code),
                Err(e) => eprintln!("Runtime Error: {}", e),
            }
        },
        Err(e) => eprintln!("Failed to load bytecode: {}", e),
    }
}