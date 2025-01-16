use colored::*;
use std::env;
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, exit};
use ctrlc;

// Handle interrupt signal gracefully
fn handle_interrupt() {
    println!("{}", "\nOperation interrupted by user. Exiting...".yellow());
    exit(1);
}

// Self-update mechanism
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

// Relaunch the current script
fn relaunch_script() {
    let args: Vec<String> = env::args().collect();
    let current_executable = env::current_exe().expect("Failed to get the current executable path");

    Command::new(current_executable)
        .args(&args[1..])
        .spawn()
        .expect("Failed to relaunch the script");

    exit(0);
}

// Function to install Git using winget on Windows
fn install_git_windows() {
    println!("{}", "Git is not installed. Attempting to install Git using winget...".blue());
    let winget_install = Command::new("winget")
        .args(["install", "--id", "Git.Git", "-e"])
        .status();

    match winget_install {
        Ok(status) if status.success() => {
            println!("{}", "Git installed successfully. Please restart the script.".green());
            exit(0);
        }
        _ => {
            println!("{}", "Failed to install Git. Winget may not be installed or is misconfigured.".red());
            exit(1);
        }
    }
}

fn main() {
    // Initialize interrupt handling
    ctrlc::set_handler(move || handle_interrupt()).expect("Error setting Ctrl-C handler");

    // Self-update
    let s2l_dir = "S2L";
    println!("{}", "Would you like to check for updates in the S2L folder? (y/n):".blue());
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    if input.trim().eq_ignore_ascii_case("y") {
        if self_update(s2l_dir) {
            println!("{}", "Restarting the script to use the updated repository...".green());
            relaunch_script();
        }
    } else {
        println!("{}", "Skipping updates for the S2L repository.".yellow());
    }

    // Default environment name
    let env_name = env::args().nth(1).unwrap_or_else(|| "S2L".to_string());

    // Paths and environment variables
    let user_name = whoami::username();
    let executable_path = format!("C:\\Users\\{}\\miniconda3\\envs\\{}\\python.exe", user_name, env_name);
    let conda_path = if cfg!(target_os = "windows") {
        format!("C:\\Users\\{}\\Miniconda3\\condabin\\conda.bat", user_name)
    } else {
        "~/miniconda3/condabin/conda".to_string()
    };

    // Check if Git is installed
    if Command::new("git").arg("--version").output().is_err() {
        if cfg!(target_os = "windows") {
            install_git_windows();
        } else {
            println!("{}", "Git is not installed. Please install Git manually to proceed.".red());
            exit(1);
        }
    }

    // Check if Conda is installed
    if Command::new(&conda_path).arg("--version").output().is_err() {
        if cfg!(target_os = "windows") {
            println!("{}", "Conda is not installed. Attempting to install it using winget...".blue());
            let winget_install = Command::new("winget")
                .args(["install", "--id", "Anaconda.Miniconda3", "-e"])
                .status();

            match winget_install {
                Ok(status) if status.success() => {
                    println!("{}", "Conda installed successfully. Please restart the script.".green());
                    exit(0);
                }
                _ => {
                    println!("{}", "Failed to install Conda. Winget may not be installed or is misconfigured.".red());
                    exit(1);
                }
            }
        } else {
            println!("{}", "Conda is not installed. Please install it manually.".red());
            exit(1);
        }
    }

    // Check if the target environment exists
    let env_list_output = Command::new(&conda_path)
        .args(["env", "list"])
        .output()
        .expect("Failed to list Conda environments");
    let env_list = String::from_utf8_lossy(&env_list_output.stdout);

    if env_list.contains(&env_name) {
        println!("{}", format!("The environment '{}' already exists. Skipping environment creation.", env_name).blue());
    } else {
        Command::new(&conda_path)
            .args(["create", "-n", &env_name, "python=3.11.9", "-y"])
            .status()
            .expect("Failed to create Conda environment");
    }

    // Clone the repository if it doesn't exist
    if !Path::new("S2L").exists() {
        Command::new("git")
            .args(["clone", "https://github.com/python313again/S2L"])
            .status()
            .expect("Failed to clone the repository");
    }

    // Install dependencies
    Command::new(&executable_path)
        .args(["-m", "pip", "install", "-r", "requirements.txt"])
        .current_dir("S2L")
        .status()
        .expect("Failed to install Python dependencies");

    // CUDA installation prompt
    println!("{}", "Do you want to use the CUDA variant of PyTorch (huge performance boost on NVIDIA GPUs)? (y/n):".blue());
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
    }

    // Launch the application
    println!("{}", "Installation and setup are complete. Launching the application...".green());
    Command::new(&executable_path)
        .arg("main.py")
        .current_dir("S2L")
        .spawn()
        .expect("Failed to launch the application");
}
