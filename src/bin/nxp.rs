// src/bin/nxp.rs
use clap::Parser;
use std::fs;
use std::path::Path;
use std::process::Command as ProcessCommand;
use anyhow::Result;

#[derive(Parser, Debug)]
#[command(name = "nxp", about = "Nexus Package Manager & Build System", version)]
struct Args {
    #[arg(short = 'S', long)]
    sync: bool,
    #[arg(short = 'u', long)]
    upgrade: bool,
    #[arg(required = false)]
    package: Option<String>,
    #[arg(short = 'B', long)]
    build: bool,
    #[arg(short = 'R', long)]
    run: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.build {
        execute_script("build")?;
    } else if args.run {
        execute_script("start")?;
    } else if args.sync {
        if args.upgrade {
            println!("Updating dependencies...");
        } else if let Some(pkg) = args.package {
            println!("Installing {}...", pkg);
        }
    } else {
        println!("Usage: nxp -B (build) | nxp -R (run) | nxp -S <pkg>");
    }
    Ok(())
}

fn execute_script(name: &str) -> Result<()> {
    println!("Trying to run script: {}", name);
    
    let path = Path::new("nxp.kdl");
    if !path.exists() {
        eprintln!("Error: nxp.kdl not found");
        return Ok(());
    }

    let content = fs::read_to_string(path)?;
    let doc = kdl::KdlDocument::parse(&content).map_err(|e| anyhow::anyhow!("KDL Parse Error: {:?}", e))?;

    if let Some(scripts_node) = doc.get("scripts") {
        if let Some(children) = scripts_node.children() {
            // Ищем ноду с именем скрипта (например "start")
            if let Some(script_node) = children.nodes().iter().find(|n| n.name().value() == name) {
                
                // Ищем аргумент "command"
                let cmd = script_node.entries().iter()
                    .find(|e| e.name().map(|n| n.value() == "command").unwrap_or(false))
                    .map(|e| e.value().to_string().trim_matches('"').to_string());

                if let Some(cmd_str) = cmd {
                    println!("> {}", cmd_str);
                    
                    let (shell, flag) = if cfg!(target_os = "windows") {
                        ("cmd", "/C")
                    } else {
                        ("sh", "-c")
                    };

                    ProcessCommand::new(shell)
                        .arg(flag)
                        .arg(&cmd_str)
                        .status()
                        .expect("Failed to run");
                } else {
                    eprintln!("No 'command' found in script '{}'", name);
                }
                return Ok(());
            }
        }
    }

    eprintln!("Script '{}' not found in nxp.kdl", name);
    Ok(())
}