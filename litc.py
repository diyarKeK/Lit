import os
import sys
import subprocess
import re
import random

def compile_lit_to_cpp(lit_path, cpp_path):
    with open(lit_path, 'r', encoding='utf-8') as f:
        code = f.read()

    code = re.sub(r'/\*.*?\*/', '', code, flags=re.DOTALL)

    lines = code.splitlines()

    cpp_code = [
        '#include <iostream>',
        '#include <string>',
        'using namespace std;\n',
        'int main() {'
    ]

    inside_main = False
    variables = {}

    for line in lines:
        line = re.sub(r'//.*', '', line).strip()

        if line.startswith('fun main()'):
            inside_main = True
            continue
        if inside_main and line == '}':
            inside_main = False
            continue

        if inside_main:

            if re.match(r"^(int|float|bool|str)(:[\w\d]+)?\s+\w+\s*=\s*.+", line):

                match = re.match(r"^(int|float|bool|str)(:[\w\d]+)?\s+(\w+)\s*=\s*(.+)", line)

                base_type = match.group(1)
                sub_type = match.group(2)[1:] if match.group(2) else None
                var_name = match.group(3)
                var_value = match.group(4).strip()

                if base_type == 'int' and sub_type and '#include <cstdint>' not in cpp_code:
                    cpp_code.insert(2, '#include <cstdint>')
                elif base_type == 'float' and sub_type and '#include <iomanip>' not in cpp_code:
                    cpp_code.insert(2, '#include <iomanip>')

                variables[var_name] = base_type if not sub_type else f'{base_type}:{sub_type}'

                if base_type == 'int':
                    type_map = {
                        None: 'int',
                        'i8': 'int8_t',
                        'i16': 'int16_t',
                        'i32': 'int32_t',
                        'i64': 'int64_t',
                        'u8': 'uint8_t',
                        'u16': 'uint16_t',
                        'u32': 'uint32_t',
                        'u64': 'uint64_t'
                    }
                    cpp_type = type_map.get(sub_type, 'int')
                    cpp_code.append(f'    {cpp_type} {var_name} = {var_value};')
                elif base_type == 'float':
                    if not  var_value.endswith('f'):
                        var_value += 'f'

                    type_map = {
                        None: 'float',
                        'f32': 'float',
                        'f64': 'double'
                    }

                    cpp_type = type_map.get(sub_type, 'float')
                    cpp_code.append(f'    {cpp_type} {var_name} = {var_value};')
                elif base_type == 'bool':
                    if var_value.lower() == 'true':
                        var_value = 'true'
                    elif var_value.lower() == 'false':
                        var_value = 'false'
                    cpp_code.append(f'    bool {var_name} = {var_value};')
                elif base_type == 'str':
                    if var_value.startswith("'") and var_value.endswith("'"):
                        text = var_value[1:-1].replace('"', '\\"').replace("\\'", "'")

                        if re.search(r'hello', text, re.IGNORECASE) and re.search(r'world', text, re.IGNORECASE):
                            text = random.choice([
                                text,
                                'Hello World is not enabled in Lit! :)'
                            ])

                        var_value = f'"{text}"'
                    cpp_code.append(f'    string {var_name} = {var_value};')

            elif line.startswith('print(') and line.endswith(')'):
                args = line[6:-1].strip()

                match = re.match(r"(.+?)(?:,\s*end\s*=\s*'(.*?)')?$", args)

                if match:
                    expr = match.group(1).strip()
                    end = match.group(2) if match.group(2) is not None else "\\n"

                    if expr.startswith("'") and expr.endswith("'"):
                        text = expr[1:-1].replace('"', '\\"').replace("\\'", "'")

                        if '{' in text:
                            cpp_code.append(f'    cout << {interpolate_string(text, variables)} << "{end}";')

                        else:
                            if re.search(r'hello', text, re.IGNORECASE) and re.search(r'world', text, re.IGNORECASE):
                                text = random.choice([
                                    text,
                                    'Hello World is not enabled in Lit! :)'
                                ])

                            cpp_code.append(f'    cout << "{text}" << "{end}";')

                    elif expr in variables:
                        var_type = variables[expr]
                        if var_type == 'bool':
                            cpp_code.append(f'    cout << ({expr} ? "true" : "false") << "{end}";')
                        elif var_type.startswith('int'):
                            if var_type.endswith('i8') or var_type.endswith('u8'):
                                cpp_code.append(f'    cout << (int){expr} << "{end}";')
                            else:
                                cpp_code.append(f'    cout << {expr} << "{end}";')
                        elif var_type.startswith('float'):
                            if var_type.endswith('f64'):
                                cpp_code.append(f'    cout << fixed << setprecision(12) << {expr} << "{end}";')
                            elif var_type.endswith('f32'):
                                cpp_code.append(f'    cout << fixed << setprecision(6) << {expr} << "{end}";')
                            else:
                                cpp_code.append(f'    cout << std::to_string({expr}) << "{end}";')
                        elif var_type == 'str':
                            cpp_code.append(f'    cout << {expr} << "{end}";')

                elif args in variables:
                    var_type = variables[args]
                    if var_type == 'bool':
                        cpp_code.append(f'    cout << ({args} ? "true" : "false") << "\\n";')
                    else:
                        cpp_code.append(f'    cout << {args} << "\\n";')

    cpp_code.append('    return 0;')
    cpp_code.append('}')

    os.makedirs(os.path.dirname(cpp_path), exist_ok=True)
    with open(cpp_path, 'w', encoding='utf-8') as f:
        f.write('\n'.join(cpp_code))


def interpolate_string(template, variables):
    result = []
    buffer = ''
    i = 0

    while i < len(template):
        if template[i:i+2] == '{{':
            buffer += '{'
            i += 2
        elif template[i:i+2] == '}}':
            buffer += '}'
            i += 2
        elif template[i] == '{':
            if buffer:
                result.append(f'"{buffer}"')
                buffer = ''
            j = i + 1
            while j < len(template) and template[j] != '}':
                j += 1
            expr = template[i+1:j].strip()
            if not expr:
                pass
            elif expr in variables and variables[expr] == 'bool':
                result.append(f'({expr} ? "true" : "false")')
            elif expr == 'true':
                result.append('"true"')
            elif expr == 'false':
                result.append('"false"')
            else:
                result.append(f'{expr}')
            i = j + 1
        else:
            buffer += template[i]
            i += 1
    if buffer:
        result.append(f'"{buffer}"')
    for res in result:
        if res is None:
            res = '""'
    return ' << '.join(result)

def compile_cpp_to_exe(cpp_path, exe_path):
    result = subprocess.run(['g++', cpp_path, '-o', exe_path])
    if result.returncode != 0:
        print("Error Of Compilation")
        sys.exit(1)
    else:
        print(f"Compiled to: {exe_path}")


def main():
    if len(sys.argv) != 2:
        print("InValid, Use: python litc.py file.lit")
        return

    lit_path = sys.argv[1]
    if not lit_path.endswith('.lit'):
        print("Not .lit file")
        return

    base_name = os.path.splitext(os.path.basename(lit_path))[0]
    build_dir = "build"
    cpp_path = os.path.join(build_dir, base_name + ".cpp")
    exe_path = os.path.join(build_dir, base_name + ".exe")

    compile_lit_to_cpp(lit_path, cpp_path)
    compile_cpp_to_exe(cpp_path, exe_path)

if __name__ == "__main__":
    main()