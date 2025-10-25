#[cfg(feature = "cli")]
use clap::{Parser, Subcommand};
#[cfg(feature = "cli")]
use wasmasc::{AscPlugin, BuildConfig, OptimizationLevel, Plugin, WasmBuilder};

#[cfg(feature = "cli")]
#[derive(Parser)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(about = env!("CARGO_PKG_DESCRIPTION"))]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(author = env!("CARGO_PKG_AUTHORS"))]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[cfg(feature = "cli")]
#[derive(Subcommand)]
enum Commands {
    #[command(alias = "c")]
    Compile {
        #[arg(short, long, default_value = ".", value_name = "PATH")]
        project: String,

        #[arg(short, long, default_value = "./dist", value_name = "DIR")]
        output: String,

        #[arg(long, value_enum, default_value = "release")]
        optimization: CliOptimization,

        #[arg(short, long)]
        verbose: bool,
    },

    CanHandle {
        #[arg(value_name = "PATH")]
        project: String,
    },

    CheckDeps,

    Info,
}

#[cfg(feature = "cli")]
#[derive(clap::ValueEnum, Clone, Debug)]
enum CliOptimization {
    Debug,
    Release,
    Size,
}

#[cfg(feature = "cli")]
impl From<CliOptimization> for OptimizationLevel {
    fn from(opt: CliOptimization) -> Self {
        match opt {
            CliOptimization::Debug => OptimizationLevel::Debug,
            CliOptimization::Release => OptimizationLevel::Release,
            CliOptimization::Size => OptimizationLevel::Size,
        }
    }
}

#[cfg(feature = "cli")]
fn print_header() {
    println!(
        "🔧 {} v{}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    println!("   {}", env!("CARGO_PKG_DESCRIPTION"));
    println!();
}

#[cfg(feature = "cli")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let plugin = AscPlugin::new();
    let builder = plugin.get_builder();

    let command = cli.command.unwrap_or(Commands::Info);

    match command {
        Commands::Compile {
            project,
            output,
            optimization,
            verbose,
        } => {
            if verbose {
                print_header();
                println!("🔨 Compiling AssemblyScript project...");
                println!("📁 Project: {project}");
                println!("📦 Output: {output}");
                println!();
            }

            let config = BuildConfig {
                project_path: project,
                output_dir: output,
                optimization_level: optimization.into(),
                verbose,
                watch: false,
            };

            match builder.build(&config) {
                Ok(result) => {
                    println!("✅ Compilation completed successfully!");
                    println!("🎯 WASM file: {}", result.wasm_path);
                }
                Err(e) => {
                    eprintln!("❌ Compilation failed: {e}");
                    std::process::exit(1);
                }
            }
        }

        Commands::CanHandle { project } => {
            if plugin.can_handle_project(&project) {
                println!("✅ Yes, wasmasc can handle this project");
            } else {
                println!("❌ No, wasmasc cannot handle this project");
                std::process::exit(1);
            }
        }

        Commands::CheckDeps => {
            print_header();
            println!("🔍 Checking system dependencies...");
            println!();

            let missing = builder.check_dependencies();

            if missing.is_empty() {
                println!("✅ All required dependencies are available!");
            } else {
                println!("❌ Missing required dependencies:");
                for dep in &missing {
                    println!("   • {dep}");
                }
                std::process::exit(1);
            }
        }

        Commands::Info => {
            print_header();
            println!("🔧 Plugin Information");
            println!("═════════════════════");
            println!("Name: {}", env!("CARGO_PKG_NAME"));
            println!("Version: {}", env!("CARGO_PKG_VERSION"));
            println!("Description: {}", env!("CARGO_PKG_DESCRIPTION"));
            println!("Author(s): {}", env!("CARGO_PKG_AUTHORS"));
            println!();
            println!("🎯 Capabilities");
            println!("═══════════════");
            println!("✅ AssemblyScript to WebAssembly compilation");
            println!("✅ npm/yarn build support");
            println!("✅ Multiple optimization levels");
            println!();
            println!("📄 Supported Extensions: ts");
            println!("📄 Entry Files: assembly/index.ts, index.ts, package.json");
        }
    }

    Ok(())
}

#[cfg(not(feature = "cli"))]
fn main() {
    eprintln!("❌ CLI feature not enabled");
    eprintln!();
    eprintln!("This library is designed as a plugin for Wasmrun:");
    eprintln!("   wasmrun plugin install wasmasc");
    eprintln!("   wasmrun run ./my-asc-project");

    std::process::exit(1);
}
