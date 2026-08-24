from computer_control import run_terminal_command, read_file_contents

# J.A.R.V.I.S. Tools Schema declarations conforming to conversational AI definition specs
JARVIS_COGNITIVE_SCHEMA = [
    {
        "name": "run_terminal_command",
        "description": "Executes a terminal or shell command on the host computer. Jarvis can use this to execute files, compile programs, run system checks, or manage workspace setups.",
        "parameters": {
            "type": "OBJECT",
            "properties": {
                "command": {
                    "type": "STRING",
                    "description": "The command line string to run in the terminal."
                }
            },
            "required": ["command"]
        }
    },
    {
        "name": "read_file_contents",
        "description": "Reads the entire contents of any file on the local computer. Jarvis can use this to inspect system configs, read documentation, or review datasets.",
        "parameters": {
            "type": "OBJECT",
            "properties": {
                "filepath": {
                    "type": "STRING",
                    "description": "The relative or absolute file path to read."
                }
            },
            "required": ["filepath"]
        }
    }
]

# Map of accessible functions for execution
AVAILABLE_TOOLS = {
    "run_terminal_command": run_terminal_command,
    "read_file_contents": read_file_contents
}
