use nexuslang::token;
use nexuslang::lexer;
use nexuslang::parser;
use nexuslang::compiler;
use nexuslang::vm;

use std::fs;
use std::path::Path;
use std::process::Command;
use clap::Parser;
use inkwell::context::Context;

#[derive(Parser, Debug)]
#[command(name = "nxc", about = "Nexus Language Compiler", version)]
struct Args {
    /// Source file (.nx)
    #[arg(required = true)]
    input: String,

    /// Compile to bytecode (.nxb)
    #[arg(long)]
    nxb: bool,

    /// Output file name
    #[arg(short, long)]
    output: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    
    let path = Path::new(&args.input);
    if !path.exists() {
        eprintln!("Error: File '{}' not found", args.input);
        return Ok(());
    }

    let code = fs::read_to_string(&args.input).expect("Failed to read file");

    // 1. Parse
    let mut lexer = lexer::Lexer::new(&code);
    let mut tokens = Vec::new();
    loop {
        let tok = lexer.next_token();
        if tok.kind == token::TokenType::EOF { tokens.push(tok); break; }
        tokens.push(tok);
    }

    let mut parser = parser::Parser::new(tokens);
    let program = match parser.parse() {
        Ok(p) => p,
        Err(e) => { eprintln!("Parse Error: {}", e); return Ok(()); }
    };

    // 2. Compile
    if args.nxb {
        // Bytecode mode
        println!("Compiling to Bytecode...");
        let mut bc = vm::ByteCompiler::new();
        let instructions = bc.compile_program(&program).unwrap();
        
        let out_name = args.output.clone().unwrap_or_else(|| "out.nxb".to_string());
        vm::save_bytecode(&instructions, &out_name).unwrap();
        println!("Success: Saved bytecode to {}", out_name);
    } else {
        // Native mode
        println!("Compiling to Native Binary...");
        let context = Context::create();
        let mut comp = compiler::Compiler::new(&context, "main_module");

        match comp.compile(&program) {
            Ok(_) => {
                let out_name = args.output.clone().unwrap_or_else(|| "main".to_string());
                let obj_name = format!("{}.o", out_name);
                
                comp.compile_to_object(&obj_name).map_err(|e| anyhow::anyhow!("{}", e))?;
                println!("Object file generated: {}", obj_name);

                // --- Linking ---
                let sdk_lib_path = "./sdk/target/release/libnexus_sdk.a";
                
                if !Path::new(sdk_lib_path).exists() {
                     eprintln!("Error: SDK not found at {}. Please build the SDK first.", sdk_lib_path);
                     return Ok(());
                }

                let linker = if Command::new("gcc").arg("--version").output().is_ok() { "gcc" } 
                             else if Command::new("clang").arg("--version").output().is_ok() { "clang" } 
                             else { "" };

                if linker.is_empty() {
                    eprintln!("Error: No linker (gcc/clang) found.");
                } else {
                    println!("Linking with '{}'...", linker);
                    let status = Command::new(linker)
                        .arg(&obj_name)
                        .arg("-o")
                        .arg(&out_name)
                        .arg(sdk_lib_path)
                        .arg("-lpthread")
                        .arg("-ldl")
                        .arg("-lm")
                        .status()
                        .expect("Failed to execute linker");

                    if status.success() {
                        println!("Executable '{}' created successfully!", out_name);
                    } else {
                        eprintln!("Linker failed.");
                    }
                }
            },
            Err(e) => eprintln!("Compilation Error: {}", e),
        }
    }

    Ok(())
}