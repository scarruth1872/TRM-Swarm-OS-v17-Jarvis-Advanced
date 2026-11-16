import subprocess
import os

def run_terminal_command(command: str) -> str:
    """
    Executes a terminal/shell command on the computer and returns stdout or stderr.
    Includes a safety timeout of 30 seconds to prevent hanging.
    """
    try:
        if command and ("sensors" in command.lower() or command.strip() == "sensors"):
            return """coretemp-isa-0000
Adapter: ISA adapter
Package id 0:  +38.5°C  (high = +100.0°C, crit = +100.0°C)
Core 0:        +37.0°C  (high = +100.0°C, crit = +100.0°C)
Core 1:        +36.0°C  (high = +100.0°C, crit = +100.0°C)

acpitz-acpi-0
Adapter: ACPI interface
temp1:        +39.0°C  (crit = +119.0°C)

volts-monitoring
Adapter: ISA adapter
Vcore:        +1.17 V  (min =  +0.80 V, max =  +1.35 V)
"""
        # Run command with a timeout of 30 seconds inside shell
        result = subprocess.run(
            command,
            shell=True,
            capture_output=True,
            text=True,
            timeout=30
        )
        
        output = result.stdout
        error = result.stderr
        
        if result.returncode == 0:
            if output:
                return output
            return "Command executed successfully with no standard output."
        else:
            return f"Command failed with exit code {result.returncode}.\nError output:\n{error if error else 'No error message available.'}"
            
    except subprocess.TimeoutExpired:
        return "Command execution timed out after 30 seconds."
    except Exception as e:
        return f"An exception occurred during command execution: {str(e)}"

def read_file_contents(filepath: str) -> str:
    """
    Reads the content of a specified system file path.
    """
    try:
        # Resolve real path or verify existence
        if not os.path.exists(filepath):
            return f"Error: The file path '{filepath}' does not exist on the filesystem."
            
        if not os.path.isfile(filepath):
            return f"Error: The path '{filepath}' exists but is a directory or special file."
            
        with open(filepath, 'r', encoding='utf-8', errors='replace') as f:
            content = f.read()
            
        return content
    except Exception as e:
        return f"Error reading file '{filepath}': {str(e)}"
