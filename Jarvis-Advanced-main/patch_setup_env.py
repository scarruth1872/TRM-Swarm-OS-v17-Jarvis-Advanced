import os
import sys

# 1. Patch setup_env.py
setup_path = "setup_env.py"
if os.path.exists(setup_path):
    with open(setup_path, "r", encoding="utf-8") as f:
        code = f.read()

    target = """            except subprocess.CalledProcessError as e:
                logging.error(f"Error occurred while running command: {e}, check details in {log_file}")
                sys.exit(1)"""

    replacement = """            except subprocess.CalledProcessError as e:
                logging.error(f"Error occurred while running command: {e}, check details in {log_file}")
                import sys
                if os.path.exists(log_file):
                    print(f"=== START OF {log_file} ===", file=sys.stderr)
                    with open(log_file, "r") as lf:
                        print(lf.read(), file=sys.stderr)
                    print(f"=== END OF {log_file} ===", file=sys.stderr)
                sys.exit(1)"""

    if target in code:
        with open(setup_path, "w", encoding="utf-8") as f:
            f.write(code.replace(target, replacement))
        print("Successfully patched setup_env.py")
    else:
        print("Warning: Target code block not found in setup_env.py")
else:
    print(f"Error: {setup_path} not found.")
    sys.exit(1)

# 2. Patch src/ggml-bitnet-mad.cpp to fix strict const-correctness error in Clang 18
cpp_path = "src/ggml-bitnet-mad.cpp"
if os.path.exists(cpp_path):
    with open(cpp_path, "r", encoding="utf-8") as f:
        cpp_code = f.read()

    target_cpp = "int8_t * y_col = y + col * by;"
    replacement_cpp = "const int8_t * y_col = y + col * by;"

    if target_cpp in cpp_code:
        with open(cpp_path, "w", encoding="utf-8") as f:
            f.write(cpp_code.replace(target_cpp, replacement_cpp))
        print("Successfully patched src/ggml-bitnet-mad.cpp")
    else:
        print("Warning: Target line not found in src/ggml-bitnet-mad.cpp")
else:
    print(f"Error: {cpp_path} not found.")
    sys.exit(1)
