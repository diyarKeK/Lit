from typing import Union

from nodes import Program, Main_Function, PrintNode, VarDeclarationNode, VarReferenceNode, ExpressionNode, \
    AssignmentNode, AugmentedAssignmentNode, IncrementNode, IfNode, ConditionNode
import re
import random


def generate_cpp(ast: Program):
    lines = [
        '#include <iostream>',
        '#include <string>',
        '#include <sstream>',
        'using namespace std;',
        '',
    ]

    variables = {}
    includes = set(lines)

    for node in ast.body:
        if isinstance(node, Main_Function):
            lines.append('int main() {')

            for stmt in node.body:
                full_stmt = generate_stmt(stmt, variables, indent='    ')
                for line in full_stmt.split('\n'):
                    if line.startswith('#include') and line not in includes:
                        lines.insert(2, line)
                        includes.add(line)
                    else:
                        lines.append(line)

            lines.append('    return 0;')
            lines.append('}')

    return '\n'.join(lines).strip()


def generate_stmt(stmt, variables, indent='') -> str:
    if isinstance(stmt, VarDeclarationNode):
        line = ''
        if stmt.suffix and not stmt.suffix.startswith('f'):
            line += '#include <cstdint>\n'
        elif stmt.suffix and stmt.suffix.startswith('f'):
            line += '#include <iomanip>\n'

        var_type = cpp_type(stmt.var_type, stmt.suffix)
        suffix = ':' + stmt.suffix if stmt.suffix else ''
        variables[stmt.name] = stmt.var_type + suffix

        if stmt.value is None:
            default = {
                'int': '0',
                'float': '0.0f',
                'bool': 'false',
                'str': '""',
            }[stmt.var_type]
            line += f'{indent}{var_type} {stmt.name} = {default};'
        elif isinstance(stmt.value, list):
            line += f'{indent}{var_type} {stmt.name} = {merge_parts(stmt.value, variables, for_string=True)};'
        elif isinstance(stmt.value, ExpressionNode):
            if stmt.var_type == 'str':
                expr_list = expression_to_string_parts(stmt.value)
                line += f'{indent}{var_type} {stmt.name} = ' + merge_parts(expr_list, variables, for_string=True) + ';'
            else:
                line += f'{indent}{var_type} {stmt.name} = {generate_expr(stmt.value, variables, var_type=variables[stmt.name])};'
        elif isinstance(stmt.value, VarReferenceNode):
            line += f'{indent}{var_type} {stmt.name} = {stmt.value.name};'
        else:
            val = cpp_literal(stmt.value)
            line += f'{indent}{var_type} {stmt.name} = {val};'

        return line

    elif isinstance(stmt, AssignmentNode):
        if isinstance(stmt.value, list):
            value_code = merge_parts(stmt.value, variables, for_string=True)
        elif isinstance(stmt.value, ExpressionNode):
            value_code = generate_expr(stmt.value, variables)
        elif isinstance(stmt.value, VarReferenceNode):
            value_code = stmt.value.name
        else:
            var_type = variables.get(stmt.name)
            if var_type == 'str' and isinstance(stmt.value, (int, float, bool)):
                value_code = f'to_string({cpp_literal(stmt.value)})'
            else:
                value_code = cpp_literal(stmt.value)

        return f'{indent}{stmt.name} = {value_code};'

    elif isinstance(stmt, AugmentedAssignmentNode):
        if stmt.name not in variables:
            raise Exception(f'Variable {stmt.name} not declared')

        var_type = variables[stmt.name]

        if stmt.operator == '%' and 'float' in var_type:
            raise Exception(f"Cannot use % for float var '{stmt.name}'")
        elif stmt.operator in ('-', '*', '/', '%') and var_type == 'str':
            raise Exception(f"Cannot use -, *, /, % for str var '{stmt.name}'")

        if isinstance(stmt.value, list):
            expr = merge_parts(stmt.value, variables, for_string=True)
        elif isinstance(stmt.value, ExpressionNode):
            expr = generate_expr(stmt.value, variables)
        elif isinstance(stmt.value, VarReferenceNode):
            expr = stmt.value.name
        else:
            expr = cpp_literal(stmt.value)

        return f'{indent}{stmt.name} {stmt.operator}= {expr};'

    elif isinstance(stmt, IncrementNode):
        return f'{indent}{stmt.name}{stmt.operator};'

    elif isinstance(stmt, PrintNode):
        line = f'{indent}cout << '

        if stmt.value == '':
            stmt.value = '""'

        elif isinstance(stmt.value, list):
            line += merge_parts(stmt.value, variables, for_string=False)
        elif isinstance(stmt.value, ExpressionNode):
            expr_parts = expression_to_string_parts(stmt.value)
            line += merge_parts(expr_parts, variables, for_string=False)
        elif isinstance(stmt.value, VarReferenceNode):
            line += merge_parts(stmt.value, variables, for_string=False)
        else:
            line += merge_parts(stmt.value, variables, for_string=False)

        if isinstance(stmt.end, list):
            line += f' << {merge_parts(stmt.end, variables, for_string=False)}'
        elif isinstance(stmt.end, VarReferenceNode):
            line += f' << {merge_parts(stmt.end, variables, for_string=False)}'
        elif isinstance(stmt.end, ExpressionNode):
            expr_parts_end = expression_to_string_parts(stmt.end)
            line += f' << {merge_parts(expr_parts_end, variables, for_string=False)}'
        else:
            line += f' << {cpp_literal(stmt.end)}'

        line += ';'
        return line

    elif isinstance(stmt, IfNode):
        return '\n'.join(generate_if_node(stmt, variables, indent))

    else:
        raise Exception(f'Unknown Statement: {stmt}')



def cpp_type(var_type: str, suffix: str = None) -> str:
    INT_SUFFIX_MAP = {
        'i8': 'int8_t', 'u8': 'uint8_t',
        'i16': 'int16_t', 'u16': 'uint16_t',
        'i32': 'int32_t', 'u32': 'uint32_t',
        'i64': 'int64_t', 'u64': 'uint64_t'
    }
    FLOAT_SUFFIX_MAP = {
        'f32': 'float', 'f64': 'double'
    }

    if var_type == 'int':
        if not suffix:
            return 'int'
        if suffix not in INT_SUFFIX_MAP:
            raise Exception(f'Unknown int suffix: {suffix}')
        return INT_SUFFIX_MAP[suffix]

    if var_type == 'float':
        if not suffix:
            return 'float'
        if suffix not in FLOAT_SUFFIX_MAP:
            raise Exception(f'Unknown float suffix: {suffix}')
        return FLOAT_SUFFIX_MAP[suffix]

    if var_type == 'bool':
        return 'bool'
    if var_type == 'str':
        return 'string'

    raise Exception(f"Can't provide cpp type for var type: {var_type} and suffix: {suffix if suffix else 'NULL'}")

def cpp_literal(val: Union[str, int, float, bool]) -> str:
    if isinstance(val, str):
        if val == '':
            return '""'
        if re.search('hello', val, re.IGNORECASE) and re.search('world', val, re.IGNORECASE):
            val = random.choice([
                val,
                'Hello, World is not enabled in Lit! :)'
            ])
        return f'"{val}"'
    elif isinstance(val, bool):
        return 'true' if val else 'false'
    elif isinstance(val, int):
        return str(int(val))
    elif isinstance(val, float):
        return f'{val}f'
    else:
        return str(val)

def merge_parts(value, variables, for_string=False):
    if isinstance(value, list):
        if len(value) == 0:
            return '""'
        parts = []
        for part in value:
            if for_string and isinstance(part, str):
                parts.append(f'std::string("{part}")')
                continue

            if isinstance(part, VarReferenceNode):
                var_type = variables.get(part.name, '')
                if 'bool' in var_type:
                    parts.append(f'({part.name} ? "true" : "false")')
                elif 'i8' in var_type or 'u8' in var_type:
                    parts.append(f'to_string(static_cast<int>({part.name}))')
                elif 'f32' in var_type:
                    parts.append(f'to_string((float){part.name})')
                elif 'f64' in var_type:
                    parts.append(f'to_string((double){part.name})')
                elif 'int' in var_type:
                    parts.append(f'to_string({part.name})')
                elif var_type == 'str':
                    parts.append(f'{part.name}')
                else:
                    parts.append(f'to_string({part.name})')
            elif isinstance(part, ExpressionNode):
                if part.operator is None:
                    inner = part.left
                    if isinstance(inner, VarReferenceNode):
                        var_type = variables.get(inner.name, '')
                        if 'f32' in var_type:
                            parts.append(f'to_string((float){inner.name})')
                        elif 'f64' in var_type:
                            parts.append(f'to_string((double){inner.name})')
                        elif 'i8' in var_type or 'u8' in var_type:
                            parts.append(f'to_string(static_cast<int>({inner.name}))')
                        elif var_type.startswith('int'):
                            parts.append(f'to_string({inner.name})')
                        elif var_type == 'str':
                            parts.append(f'{inner.name}')
                        else:
                            parts.append(f'to_string({inner.name})')
                    elif isinstance(inner, str):
                        parts.append(f'"{inner}"')
                    else:
                        parts.append(f'to_string({generate_expr(inner, variables)})')
                else:
                    parts.append(f'to_string({generate_expr(part, variables)})')
            elif isinstance(part, bool):
                parts.append('"true"' if part else '"false"')
            elif isinstance(part, str):
                parts.append(f'"{part}"')
            elif isinstance(part, (int, float)):
                parts.append(f'to_string({part})')
            else:
                parts.append(f'"{str(part)}"')
        return ' + '.join(parts) if for_string else ' << '.join(parts)

    elif isinstance(value, VarReferenceNode):
        var_type = variables.get(value.name)
        if value.name in variables and variables[value.name] == 'bool':
            return f'({value.name} ? "true" : "false")'
        elif 'i8' in var_type or 'u8' in var_type:
            return f'static_cast<int>({value.name})'
        elif 'f32' in var_type:
            if for_string:
                return f'to_string((float){value.name})'
            else:
                return f'fixed << setprecision(6) << {value.name}'
        elif 'f64' in var_type:
            if for_string:
                return f'to_string((double){value.name})'
            else:
                return f'fixed << setprecision(12) << {value.name}'
        else:
            return value.name

    elif isinstance(value, ExpressionNode):
        return generate_expr(value, variables, for_string=True)

    elif isinstance(value, bool):
        return '"true"' if value else '"false"'

    elif isinstance(value, (int, float)):
        return f'to_string({value})'

    elif isinstance(value, str):
        return f'std::string("{value}")'

    else:
        return f'string("{value}")'

def generate_expr(expr, variables, for_string=False, var_type=''):
    if isinstance(expr, ExpressionNode):
        if expr.operator is None:
            return generate_expr(expr.left, variables, for_string, var_type)
        elif expr.operator == 'not':
            right = generate_expr(expr.right, variables, for_string=False, var_type=var_type)
            return f'!({right})'
        else:
            left = generate_expr(expr.left, variables, for_string, var_type)
            right = generate_expr(expr.right, variables, for_string, var_type)
            return f'({left} {expr.operator} {right})'

    elif isinstance(expr, VarReferenceNode):
        var_type = variables.get(expr.name)
        if var_type == 'bool':
            return f'({expr.name} ? "true" : "false")'
        elif 'i8' in var_type or 'u8' in var_type:
            return f'static_cast<int>({expr.name})'
        elif 'f32' in var_type:
            return f'to_string((float){expr.name})' if for_string else expr.name
        elif 'f64' in var_type:
            return f'to_string((double){expr.name})' if for_string else expr.name
        else:
            return expr.name

    elif isinstance(expr, str):
        return f'"{expr}"'

    elif isinstance(expr, bool):
        if for_string:
            return '"true"' if expr else '"false"'
        return 'true' if expr else 'false'

    elif isinstance(expr, int):
        return str(expr)

    elif isinstance(expr, float):
        if 'f32' in var_type:
            return f'{expr}f'
        elif 'f64' in var_type:
            return f'{expr}'
        return f'{expr}f'

    else:
        raise Exception(f'Unknown expression part: {expr}')



def expression_to_string_parts(expr):
    parts = []

    def flatten(node):
        if isinstance(node, ExpressionNode) and node.operator == '+':
            flatten(node.left)
            flatten(node.right)
        else:
            parts.append(node)

    flatten(expr)
    return parts




def generate_if_node(node: IfNode, variables: dict, indent=''):
    lines = []

    def generate_condition(condition):
        if isinstance(condition, ConditionNode):
            if condition.operator == 'not':
                inner = generate_condition(condition.right)
                return f'!({inner})'
            elif condition.operator in ('and', 'or'):
                left = generate_condition(condition.left)
                right = generate_condition(condition.right)
                op = '&&' if condition.operator == 'and' else '||'
                return f'({left} {op} {right})'
            else:
                left = generate_expr(condition.left, variables)
                right = generate_expr(condition.right, variables)
                return f'{left} {condition.operator} {right}'
        elif isinstance(condition, VarReferenceNode):
            return condition.name
        elif isinstance(condition, ExpressionNode):
            return generate_expr(condition, variables)
        elif isinstance(condition, bool):
            return 'true' if condition else 'false'
        else:
            raise Exception(f'Unknown condition type: {type(condition)}')

    condition = generate_condition(node.condition)
    lines.append(f'{indent}if ({condition}) {{')

    for stmt in node.body:
        lines.append(generate_stmt(stmt, variables, indent + '    '))

    lines.append(f'{indent}}}')

    for elif_block in node.elif_blocks:
        condition = generate_condition(elif_block.condition)
        lines.append(f'{indent}else if ({condition}) {{')

        for stmt in elif_block.body:
            lines.append(generate_stmt(stmt, variables, indent + '    '))

        lines.append(f'{indent}}}')

    if node.else_body:
        lines.append(f'{indent}else {{')
        for stmt in node.else_body:
            lines.append(generate_stmt(stmt, variables, indent + '    '))

        lines.append(f'{indent}}}')

    return lines
