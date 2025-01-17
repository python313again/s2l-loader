use colored::*;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, exit};
use ctrlc;

// Handle interrupt signal gracefully
fn handle_interrupt() {
    println!("{}", "\nOperation interrupted by user. Exiting...".yellow());
    exit(1);
}

// Install Git and Conda using winget (Windows) or download (macOS)
fn install_with_winget(package: &str) {
    println!("{}", format!("Installing {} using winget...", package).blue());
    let status = Command::new("winget")
        .args(["install", "--id", package, "-e", "--silent"])
        .status();

    match status {
        Ok(status) if status.success() => {
            println!("{}", format!("{} installed successfully.", package).green());
        }
        _ => {
            println!("{}", format!("Failed to install {}. Please install it manually and retry.", package).red());
            exit(1);
        }
    }
}

fn install_conda() {
    let user_name = whoami::username();
    if cfg!(target_os = "windows") {
        install_with_winget("Continuum.Miniconda3");
    } else if cfg!(target_os = "macos") {
        // macOS Conda installation via Homebrew
        println!("{}", "Installing Miniconda3 on macOS via Homebrew...".blue());
        let status = Command::new("brew")
            .args(["install", "miniconda"])
            .status();

        match status {
            Ok(status) if status.success() => {
                println!("{}", "Miniconda3 installed successfully on macOS.".green());
            }
            _ => {
                println!("{}", "Failed to install Miniconda3. Please install it manually and retry.".red());
                exit(1);
            }
        }
    }
}

// Check if Conda is installed and available
fn is_conda_installed(conda_path: &str) -> bool {
    Command::new(conda_path)
        .arg("--version")
        .output()
        .is_ok()
}

fn self_update(s2l_dir: &str) -> bool {
    if !Path::new(s2l_dir).exists() {
        println!("{}", "S2L folder does not exist. Skipping self-update.".yellow());
        return false;
    }

    println!("{}", "Checking for updates in the S2L folder...".blue());
    let output = Command::new("git")
        .arg("pull")
        .arg("--rebase")
        .current_dir(s2l_dir)
        .output();

    match output {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            if stdout.contains("Already up to date") {
                println!("{}", "No updates available in the S2L repository.".green());
                false
            } else {
                println!("{}", "Updates have been applied to the S2L repository.".green());
                true
            }
        }
        Err(err) => {
            println!("{}\n{}", "Failed to check for updates in the S2L folder. Error details:".red(), err);
            false
        }
    }
}

fn relaunch_script() {
    let args: Vec<String> = env::args().collect();
    let current_executable = env::current_exe().expect("Failed to get the current executable path");

    Command::new(current_executable)
        .args(&args[1..])
        .spawn()
        .expect("Failed to relaunch the script");

    exit(0);
}

fn main() {
    // Initialize interrupt handling
    ctrlc::set_handler(move || handle_interrupt()).expect("Error setting Ctrl-C handler");

    let s2l_dir = "S2L";
    let env_name = env::args().nth(1).unwrap_or_else(|| "S2L".to_string());
    let user_name = whoami::username();
    let executable_path = format!("C:\\Users\\{}\\miniconda3\\envs\\{}\\python.exe", user_name, env_name);
    let conda_path = if cfg!(target_os = "windows") {
        format!("C:\\Users\\{}\\Miniconda3\\condabin\\conda.bat", user_name)
    } else {
        "~/miniconda3/condabin/conda".to_string()
    };

    // Check if Conda is installed
    if !is_conda_installed(&conda_path) {
        println!("{}", "Conda is not installed. Installing Conda...".yellow());
        install_conda();
    }

    // Windows-specific setup for missing Git and Conda
    if cfg!(target_os = "windows") {
        if Command::new("git").arg("--version").output().is_err() {
            install_with_winget("Git.Git");
        }
    }

    // Check if the repository already exists
    if Path::new(s2l_dir).exists() {
        println!("{}", "The S2L repository already exists. Skipping setup steps.".green());
        println!("{}", "Would you like to check for updates in the S2L folder? (y/n):".blue());
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        if input.trim().eq_ignore_ascii_case("y") {
            if self_update(s2l_dir) {
                println!("{}", "Restarting the script to use the updated repository...".green());
                relaunch_script();
            }
        }
    } else {
        // Clone and setup S2L repository
        // Check if the target environment exists
        let env_list_output = Command::new(&conda_path)
            .args(["env", "list"])
            .output()
            .expect("Failed to list Conda environments");
        let env_list = String::from_utf8_lossy(&env_list_output.stdout);

        if !env_list.contains(&env_name) {
            Command::new(&conda_path)
                .args(["create", "-n", &env_name, "python=3.11.9", "-y"])
                .status()
                .expect("Failed to create Conda environment");
        }

        // Clone the repository
        Command::new("git")
            .args(["clone", "https://github.com/python313again/S2L"])
            .status()
            .expect("Failed to clone the repository");

        // Install dependencies
        Command::new(&executable_path)
            .args(["-m", "pip", "install", "-r", "requirements.txt"])
            .current_dir(s2l_dir)
            .status()
            .expect("Failed to install Python dependencies");

        // CUDA installation logic
        if cfg!(target_os = "windows") || cfg!(target_os = "linux") {
            println!("{}", "Do you want to use the CUDA variant of PyTorch? (y/n):".blue());
            let mut cuda_input = String::new();
            io::stdin().read_line(&mut cuda_input).unwrap();

            if cuda_input.trim().eq_ignore_ascii_case("y") {
                Command::new(&executable_path)
                    .args(["-m", "pip", "uninstall", "torch", "-y"])
                    .status()
                    .expect("Failed to uninstall Torch");

                Command::new(&executable_path)
                    .args(["-m", "pip", "install", "torch", "--index-url", "https://download.pytorch.org/whl/cu124"])
                    .status()
                    .expect("Failed to install CUDA variant of PyTorch");
                fs::remove_file("S2L\\libs\\roi_visualizer.py").unwrap_or_else(|err| {
                    println!("{}", format!("Failed to delete roi_visualizer.cp311-win_amd64.pyd: {}", err).red());
                });
            } else {
                fs::remove_file("S2L\\libs\\roi_visualizer.cp311-win_amd64.pyd").unwrap_or_else(|err| {
                    println!("{}", format!("Failed to delete roi_visualizer.cp311-win_amd64.pyd: {}", err).red());
                });
            }
        } else if cfg!(target_os = "macos") {
            fs::remove_file("S2L\\libs\\roi_visualizer.cp311-win_amd64.pyd").unwrap_or_else(|err| {
                println!("{}", format!("Failed to delete roi_visualizer.cp311-win_amd64.pyd: {}", err).red());
            });
        }
    }

    // Launch the application
    println!("{}", "Launching the application...".green());
    Command::new(&executable_path)
        .arg("main.py")
        .current_dir(s2l_dir)
        .spawn()
        .expect("Failed to launch the application");
}
