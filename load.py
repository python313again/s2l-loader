import os
import subprocess
from colorama import init as colorINIT
from colorama import Fore
import sys
import signal
import platform

# Initialize Colorama
colorINIT()

# Handle keyboard interrupts gracefully
def handle_interrupt(sig, frame):
    print(Fore.YELLOW + "\nOperation interrupted by user. Exiting...")
    exit(1)

signal.signal(signal.SIGINT, handle_interrupt)

# Self-update mechanism
def self_update():
    """
    Checks for updates in the S2L folder by pulling the latest version from the Git repository.
    """
    s2l_dir = os.path.join(os.getcwd(), "S2L")

    if not os.path.exists(s2l_dir):
        print(Fore.YELLOW + "S2L folder does not exist. Skipping self-update.")
        return False

    print(Fore.BLUE + "Checking for updates in the S2L folder...")
    try:
        result = subprocess.run(
            ["git", "pull", "--rebase"],
            cwd=s2l_dir,
            text=True,
            capture_output=True,
            check=True,
        )
        output = result.stdout

        if "Already up to date" in output:
            print(Fore.GREEN + "No updates available in the S2L repository.")
            return False

        print(Fore.GREEN + "Updates have been applied to the S2L repository.")
        return True

    except subprocess.CalledProcessError as e:
        print(Fore.RED + "Failed to check for updates in the S2L folder. Error details:")
        print(Fore.YELLOW + e.stderr)
        return False

# Relaunch the current script
def relaunch_script():
    python_executable = sys.executable
    script_path = os.path.abspath(__file__)
    os.execv(python_executable, [python_executable, script_path] + sys.argv[1:])

# Self-update before proceeding
if os.path.exists("S2L"):
    if "y" in input(Fore.BLUE + "Would you like to check for updates in the S2L folder? (y/n): ").strip().lower():
        updated = self_update()
        if updated:
            print(Fore.GREEN + "Restarting the script to use the updated repository...")
            relaunch_script()
    else:
        print(Fore.YELLOW + "Skipping updates for the S2L repository.")
else:
    print(Fore.YELLOW + "S2L folder not found. Skipping repository updates.")

# Default environment name
env_name = sys.argv[1] if len(sys.argv) > 1 else "S2L"

# Paths and environment variables
if os.name == "nt":
    executable_path = f"C:\\Users\\{os.getlogin()}\\miniconda3\\envs\\{env_name}\\python.exe"
    conda_path = f"C:\\Users\\{os.getlogin()}\\Miniconda3\\condabin\\conda.bat"
else:
    executable_path = f"~/miniconda3/envs/{env_name}/bin/python"
    conda_path = "~/miniconda3/bin/conda"

# Function to install Miniconda3
def install_miniconda():
    if os.name == "nt":
        print(Fore.RED + "Miniconda auto-installation is not supported on Windows via this function.")
        exit(1)

    url = "https://repo.anaconda.com/miniconda/Miniconda3-latest-Linux-x86_64.sh"
    if platform.system() == "Darwin":
        url = "https://repo.anaconda.com/miniconda/Miniconda3-latest-MacOSX-x86_64.sh"

    print(Fore.BLUE + "Downloading Miniconda installer...")
    installer_path = "/tmp/miniconda_installer.sh"
    subprocess.run(["curl", "-o", installer_path, "-L", url], check=True)

    print(Fore.BLUE + "Installing Miniconda...")
    subprocess.run(["bash", installer_path, "-b"], check=True)
    os.environ["PATH"] += os.pathsep + "~/miniconda3/bin"

# Check if Git is installed
try:
    subprocess.run(["git", "--version"], stdout=subprocess.DEVNULL, check=True)
except subprocess.CalledProcessError:
    print(Fore.RED + "Git is not installed. Attempting to install Git...")
    if os.name == "posix":
        if platform.system() == "Linux":
            subprocess.run(["sudo", "apt-get", "install", "-y", "git"], check=True)
        elif platform.system() == "Darwin":
            subprocess.run(["brew", "install", "git"], check=True)
    else:
        print(Fore.RED + "Please install Git manually for your system.")
        exit(1)

# Check if Conda is installed
try:
    subprocess.run([conda_path, "--version"], shell=True, text=True, check=True, stdout=subprocess.DEVNULL)
except subprocess.CalledProcessError:
    print(Fore.BLUE + "Conda is not installed. Installing Miniconda...")
    install_miniconda()
    print(Fore.GREEN + "Miniconda installed successfully. Please restart the script.")
    exit(0)

# Check if the target environment exists
env_list = subprocess.run([conda_path, "env", "list"], shell=True, capture_output=True, text=True)
if env_name in env_list.stdout:
    print(Fore.BLUE + f"The environment '{env_name}' already exists. Skipping environment creation.")
else:
    subprocess.run([conda_path, "create", "-n", env_name, "python=3.11.9", "-y"], shell=True)

# Clone the repository if it doesn't exist
if not os.path.exists("S2L"):
    subprocess.run(["git", "clone", "https://github.com/python313again/S2L"], check=True)

os.chdir("S2L")
subprocess.run([executable_path, "-m", "pip", "install", "-r", "requirements.txt"], check=True)

if os.path.exists("libs/roi_visualizer.py") and os.path.exists("libs/roi_visualizer.cp311-win_amd64.pyd"):
    if platform.system() != "Darwin":
        if "y" in input(Fore.BLUE + "Do you want to use the CUDA variant of PyTorch (huge performance boost on NVIDIA GPUs)? y/n: ").strip().lower():
            subprocess.run([executable_path, "-m", "pip", "uninstall", "torch", "-y"], check=True)
            subprocess.run([executable_path, "-m", "pip", "install", "torch", "--index-url", "https://download.pytorch.org/whl/cu124"], check=True)
            os.remove("libs/roi_visualizer.py")
        else:
            os.remove("libs/roi_visualizer.cp311-win_amd64.pyd")
    else:
        print(Fore.BLUE + "Your system is not compatible with CUDA anyway, using non-CUDA version.")
        os.remove("libs/roi_visualizer.cp311-win_amd64.pyd")

# Launch the application
print(Fore.GREEN + "Installation and setup are complete. Launching the application...")
subprocess.Popen([executable_path, "main.py"], shell=True)
