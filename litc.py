import sys
import os
import subprocess
import re
import random


# TO-DO:
# Оптимизация компилятора (litc.py) построение в архитектуру
# Вывод ошибок самим компилятором для .lit файлов - Syntax_Error пока что


def compile_lit_to_c(lit_path, output_c_path):
    with open(lit_path, "r", encoding="utf-8") as f:
        code = f.read()

    code = re.sub(r'/\*.*?\*/', '', code, flags=re.DOTALL)

    lines = code.splitlines()

    c_code = [
        '#include <stdio.h>',
        '#include <stdlib.h>',
        '#include <stdint.h>',
        '#include <string.h>',
        'int main() {'
    ]

    variables = {}
    tmp_print_counter = 0

    for line in lines:
        line = re.sub(r'//.*', '', line).strip()

        if not line:
            continue

        if line.startswith(("int", "float", "bool", "str")):

            header, *rest = line.split()
            if ':' in header:
                base_type, sub_type = header.split(':')
            else:
                base_type = header
                sub_type = None

            if len(rest) >= 3 and rest[1] == "=":
                var_name = rest[0]
                var_value = " ".join(rest[2:])
                var_value = clean_number_literals(var_value)
                c_type = None

                if base_type == "int":
                    c_type = {
                        None: "int",
                        "i8": "int8_t",
                        "i16": "int16_t",
                        "i32": "int32_t",
                        "i64": "int64_t",
                        "u8": "uint8_t",
                        "u16": "uint16_t",
                        "u32": "uint32_t",
                        "u64": "uint64_t"
                    }.get(sub_type, "int")
                elif base_type == "float":
                    c_type = {
                        None: "float",
                        "f32": "float",
                        "f64": "double"
                    }.get(sub_type, "float")
                elif base_type == "bool":
                    c_type = "bool"
                    if var_value == "true":
                        var_value = "1"
                    elif var_value == "false":
                        var_value = "0"
                elif base_type == "str":
                    c_type = "char*"
                    if var_value.startswith("'") and var_value.endswith("'") and not var_value.__contains__('+'):
                        var_value = '"' + var_value[1:-1].replace('"', '\\"') + '"'
                        c_code.append(f'    {c_type} {var_name} = {var_value};')
                        variables[var_name] = base_type
                        continue
                    else:
                        express_str(c_code, variables, base_type, var_name, var_value)
                        continue

                if c_type == "float" and not var_value.endswith("f"):
                    var_value += "f"

                c_code.append(f'    {c_type} {var_name} = {var_value};')
                variables[var_name] = base_type if sub_type is None else f"{base_type}:{sub_type}"
            continue

        elif line.startswith("print(") and line.endswith(")"):
            inner = line[6:-1].strip()

            if inner == "":
                c_code.append('    printf("\\n");')
                continue

            if inner.startswith("'") and inner.endswith("'"):
                content = inner[1:-1].strip()

                if re.search(r'hello', content, re.IGNORECASE) and re.search(r'world', content, re.IGNORECASE):
                    content = random.choice([
                        content,
                        "Hello World is not enabled in Lit! :)"
                    ])

                text = content.replace('"', '\\"').replace("\\'", "'")
                c_code.append(f'    printf("{text}\\n");')
                continue

            if is_string_expression(inner, variables):
                tmp_var = f"__tmp_print_{tmp_print_counter}"
                tmp_print_counter += 1
                generate_str_concat(c_code, variables, tmp_var, inner)
                c_code.append(f'    printf("%s\\n", {tmp_var});')
                continue

            if inner == "true":
                c_code.append(f'    printf("true\\n");')

            elif inner == "false":
                c_code.append(f'    printf("false\\n");')


            elif re.match(r'^\d+$', inner):
                c_code.append(f'    printf("%d\\n", {inner});')

            elif re.match(r'^\d+\.\d*$', inner):
                c_code.append(f'    printf("%f\\n", {inner});')


            elif inner in variables:
                var_type = variables[inner]

                if var_type.startswith('int'):
                    if ':i64' in var_type or ':u32' in var_type or ':u64' in var_type:
                        c_code.insert(4, '#include <inttypes.h>')
                        if ':i64' in var_type:
                            c_code.append(f'    printf("%" PRId64 "\\n", {inner});')
                        elif ':u32' in var_type:
                            c_code.append(f'    printf("%" PRIu32 "\\n", {inner});')
                        elif ':u64' in var_type:
                            c_code.append(f'    printf("%" PRIu64 "\\n", {inner});')
                    else:
                        c_code.append(f'    printf("%d\\n", {inner});')
                elif var_type.startswith('float'):
                    if ':f64' in var_type:
                        c_code.append(f'    printf("%.12f\\n", {inner});')
                    else:
                        c_code.append(f'    printf("%f\\n", {inner});')
                elif var_type == 'bool':
                    c_code.append(f'    printf("%s\\n", {inner} ? "true" : "false");')
                elif var_type == 'str':
                    c_code.append(f'    printf("%s\\n", {inner});')
                else:
                    c_code.append('    printf("unknown type\\n");')

            else:
                if re.search(r'\d+\.\d*', inner) or any(var in inner for var in variables if variables[var] == "float"):
                    c_code.append(f'    printf("%f\\n", (float)({inner}));')
                else:
                    c_code.append(f'    printf("%d\\n", ({inner}));')

    c_code.append('    return 0;')
    c_code.append('}')

    os.makedirs(os.path.dirname(output_c_path), exist_ok=True)
    with open(output_c_path, "w", encoding="utf-8") as f:
        f.write("\n".join(c_code))




def split_concat_expr(expr):
    parts = []
    current = ''
    depth = 0

    i = 0
    while i < len(expr):
        ch = expr[i]

        if ch == '(':
            if depth == 0 and current.strip():
                parts.append(current.strip())
                current = ''
            depth += 1
            current += ch
        elif ch == ')':
            depth -= 1
            current += ch
            if depth == 0:
                parts.append(current.strip())
                current = ''
        elif ch == '+' and depth == 0:
            if current.strip():
                parts.append(current.strip())
                current = ''
        else:
            current += ch
        i += 1

    if current.strip():
        parts.append(current.strip())

    return parts




def estimate_str_length(expr, variables):
    parts = split_concat_expr(expr)
    total = 0

    for part in parts:
        part = part.strip()
        if part.startswith("'") and part.endswith("'"):
            total += len(part[1:-1])
        elif part == "true":
            total += 4
        elif part == "false":
            total += 5
        elif re.match(r'^\d+\.\d*$', part):
            total += 16
        elif re.match(r'^\d+$', part):
            total += 12
        elif part in variables:
            var_type = variables[part]
            if var_type == "str":
                total += 64
            elif var_type == "int":
                total += 12
            elif var_type == "float":
                total += 24
            elif var_type == "bool":
                total += 5
        elif part.startswith("(") and part.endswith(")"):
            total += estimate_str_length(part[1:-1], variables)
        else:
            total += 16  # fallback
    return total




def generate_str_concat(c_code, variables, var_name, expr, is_top_level=True):
    parts = split_concat_expr(expr)

    if is_top_level:
        est_len = estimate_str_length(expr, variables)
        c_code.append(f'    int __len_{var_name} = {est_len} + 1;')
        c_code.append(f'    char* {var_name} = malloc(__len_{var_name});')
        c_code.append(f'    {var_name}[0] = \'\\0\';')

    for part in parts:
        if part.startswith("'") and part.endswith("'"):
            val = '"' + part[1:-1].replace('"', '\\"') + '"'
            c_code.append(f'    strcat({var_name}, {val});')
        elif part.startswith('(') and part.endswith(')'):
            inner_expr = part[1:-1].strip()
            if is_string_expression(inner_expr, variables):
                generate_str_concat(c_code, variables, var_name, inner_expr, is_top_level=False)
            else:
                if re.search(r'\d+\.\d*', inner_expr):
                    c_code.append(f'    {{ char buf[32]; sprintf(buf, "%g", (double)({inner_expr})); strcat({var_name}, buf); }}')
                else:
                    c_code.append(f'    {{ char buf[32]; sprintf(buf, "%d", (int)({inner_expr})); strcat({var_name}, buf); }}')
            continue
        elif part == "true":
            c_code.append(f'    strcat({var_name}, "true");')
        elif part == "false":
            c_code.append(f'    strcat({var_name}, "false");')
        elif re.search('[0-9]+.', part):
            c_code.append(f'    {{ char buf[32]; sprintf(buf, "%g", (double)({part})); strcat({var_name}, buf); }}')
        elif re.search('[0-9]+', part):
            c_code.append(f'    {{ char buf[32]; sprintf(buf, "%d", (int)({part})); strcat({var_name}, buf); }}')
        elif part in variables:
            if variables[part] == "str":
                c_code.append(f'    strcat({var_name}, {part});')
            elif variables[part].startswith("int"):
                c_code.append(f'    {{ char buf[32]; sprintf(buf, "%d", {part}); strcat({var_name}, buf); }}')
            elif variables[part].startswith("float"):
                c_code.append(f'    {{ char buf[32]; sprintf(buf, "%f", {part}); strcat({var_name}, buf); }}')
            elif variables[part] == "bool":
                c_code.append(f'    {{ char buf[32]; sprintf(buf, "%s", {part} ? "true" : "false"); strcat({var_name}, buf); }}')




def express_str(c_code, variables, base_type, var_name, var_value):
    generate_str_concat(c_code, variables, var_name, var_value)
    variables[var_name] = base_type




def is_string_expression(expr, variables):
    parts = split_concat_expr(expr)
    for part in parts:
        part = part.strip()
        if part.startswith("'") and part.endswith("'"):
            return True
        if part in variables and variables[part] == "str":
            return True
        if part.startswith("(") and part.endswith(")"):
            if is_string_expression(part[1:-1], variables):
                return True

    return False




def clean_number_literals(expr):
    return re.sub(r'(?<=\d)_(?=\d)', '', expr)




def compile_c_to_exe(c_path, exe_path):
    result = subprocess.run(["gcc", c_path, "-o", exe_path])
    if result.returncode != 0:
        print("Error Of Compilation C Code")
        sys.exit(1)
    else:
        print(f"Compiled to: {exe_path}")




def main():
    if len(sys.argv) != 2:
        print("Use: python litc.py path/to/file.lit")
        return

    lit_path = sys.argv[1]
    if not lit_path.endswith(".lit"):
        print("Not .lit file")
        return

    base_name = os.path.splitext(os.path.basename(lit_path))[0]
    build_dir = "build"
    c_path = os.path.join(build_dir, base_name + ".c")
    exe_path = os.path.join(build_dir, base_name + ".exe")

    compile_lit_to_c(lit_path, c_path)
    compile_c_to_exe(c_path, exe_path)

if __name__ == "__main__":
    main()