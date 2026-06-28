use std::process::Command;
use std::fs;
use std::path::Path;

fn verify_tool(name: &str, program: &str, args: &[&str]) -> bool {
    print!("Checking for {}... ", name);
    match Command::new(program).args(args).output() {
        Ok(output) if output.status.success() => {
            println!("Found.");
            true
        }
        _ => {
            println!("NOT FOUND.");
            false
        }
    }
}

fn main() {
    println!("==========================================================");
    println!("         🔒 MCPC ENTERPRISE INSTALLATION WIZARD           ");
    println!("==========================================================");
    println!("Target: Native Production Installer");
    println!();

    // 1. Verify Prerequisites
    let mut prereqs = true;
    prereqs &= verify_tool("Rust/Cargo", "cargo", &["--version"]);
    prereqs &= verify_tool("Node.js", "node", &["--version"]);
    prereqs &= verify_tool("npm", "npm", &["--version"]);
    prereqs &= verify_tool("Docker", "docker", &["--version"]);

    if !prereqs {
        eprintln!("\n[Error] Prerequisites check failed. Please install missing tools and try again.");
        std::process::exit(1);
    }

    // 2. Build Release orchestrator
    println!("\n[1/4] Compiling Rust backend in Release Mode...");
    let status = Command::new("cargo")
        .args(&["build", "--release"])
        .status();
    
    match status {
        Ok(s) if s.success() => println!("Rust backend successfully compiled."),
        _ => {
            eprintln!("[Error] Rust compilation failed.");
            std::process::exit(1);
        }
    }

    // 3. Build GUI
    println!("\n[2/4] Initializing React/Tauri GUI...");
    println!("Installing GUI dependencies...");
    let status = Command::new("npm")
        .args(&["install"])
        .current_dir("mcpc-gui")
        .status();
    if !status.map(|s| s.success()).unwrap_or(false) {
        eprintln!("[Error] GUI dependencies installation failed.");
        std::process::exit(1);
    }

    println!("Building GUI client...");
    let status = Command::new("npm")
        .args(&["run", "build"])
        .current_dir("mcpc-gui")
        .status();
    if !status.map(|s| s.success()).unwrap_or(false) {
        eprintln!("[Error] GUI compilation failed.");
        std::process::exit(1);
    }
    println!("GUI successfully compiled.");

    // 4. Install Binary
    println!("\n[3/4] Installing binary to user home directory...");
    let home = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")).expect("Failed to locate home directory");
    let install_dir = Path::new(&home).join(".mcpc").join("bin");
    fs::create_dir_all(&install_dir).expect("Failed to create installation directory");

    #[cfg(windows)]
    let binary_name = "mcpc.exe";
    #[cfg(not(windows))]
    let binary_name = "mcpc";

    let src_path = Path::new("target").join("release").join(binary_name);
    let dest_path = install_dir.join(binary_name);

    if src_path.exists() {
        if let Err(e) = fs::copy(&src_path, &dest_path) {
            eprintln!("[Error] Failed to copy binary: {}", e);
            std::process::exit(1);
        }
        println!("Binary copied to {}", dest_path.display());
    } else {
        eprintln!("[Error] Compiled binary not found at {}", src_path.display());
        std::process::exit(1);
    }

    // 5. Update Path (Windows specific)
    #[cfg(windows)]
    {
        println!("\n[4/4] Registering binary path in system Environment...");
        let path_str = install_dir.to_string_lossy().to_string();
        let ps_cmd = format!(
            "$current = [Environment]::GetEnvironmentVariable('Path', [EnvironmentVariableTarget]::User); if ($current -notlike '*{0}*') {{ [Environment]::SetEnvironmentVariable('Path', $current + ';{0}', [EnvironmentVariableTarget]::User) }}",
            path_str
        );
        let status = Command::new("powershell")
            .args(&["-NoProfile", "-Command", &ps_cmd])
            .status();
        if status.map(|s| s.success()).unwrap_or(false) {
            println!("Path successfully updated. Please restart your shell to apply changes.");
        } else {
            println!("Failed to update Path variable via PowerShell wrapper.");
        }
    }
    #[cfg(not(windows))]
    {
        println!("\n[4/4] Registering binary path in shell profile...");
        println!("Please ensure {} is added to your PATH variable.", install_dir.display());
    }

    // Create directories
    let _ = fs::create_dir_all(Path::new(&home).join(".mcpc").join("plugins"));
    let _ = fs::create_dir_all(Path::new(&home).join(".mcpc").join("cache"));

    println!("\n==========================================================");
    println!("🔒 MCPC INSTALLATION COMPLETED SUCCESSFULLY!");
    println!("==========================================================");
}
