import os
import sys
import subprocess
from generator import generate_cpp
from lexer import tokenize
from optimizer import optimize
from parser import Parser

def compile_cpp_to_exe(cpp_path, exe_path):
    result = subprocess.run(['g++', cpp_path, '-o', exe_path])
    if result.returncode != 0:
        print("Error Of Compilation")
        sys.exit(1)
    else:
        print(f"Compiled to: {exe_path}")

def main():
    if len(sys.argv) != 2:
        print('Use: python litc.py file.lit')
        return

    lit_path = sys.argv[1]
    if not lit_path.endswith('.lit'):
        print(f'Not .lit file: {lit_path}')
        return
    with open(lit_path, 'r', encoding='utf-8') as f:
        code = f.read()

    code = optimize(code, lit_path)
    tokens = tokenize(code)
    parser = Parser(tokens, lit_path)
    ast = parser.parse()
    cpp_code = generate_cpp(ast)

    base_name = os.path.splitext(os.path.basename(lit_path))[0]
    build_dir = 'build'
    cpp_path = os.path.join(build_dir, base_name + ".cpp")
    exe_path = os.path.join(build_dir, base_name + ".exe")

    with open(cpp_path, 'w', encoding='utf-8') as f:
        f.write(cpp_code)

    compile_cpp_to_exe(cpp_path, exe_path)

# TO-DO
# 1. if-else с операторами: and, or, not, ==, >, <, >=, <=, !=

if __name__ == "__main__":
    main()