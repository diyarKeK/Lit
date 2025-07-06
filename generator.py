from typing import Union

from nodes import Program, Main_Function, PrintNode, VarDeclarationNode, VarReferenceNode, ExpressionNode, \
    AssignmentNode, AugmentedAssignmentNode, IncrementNode
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

    for node in ast.body:
        if isinstance(node, Main_Function):
            lines.append('int main() {')

            for stmt in node.body:
                if isinstance(stmt, VarDeclarationNode):
                    var_type = cpp_type(stmt.var_type)
                    variables[stmt.name] = stmt.var_type

                    if isinstance(stmt.value, list):
                        line = f'{var_type} {stmt.name} = '
                        line += merge_parts(stmt.value, variables, for_string=True) + ';'
                        lines.append(f'    {line}')
                    elif isinstance(stmt.value, ExpressionNode):
                        line = f'{var_type} {stmt.name} = {generate_expr(stmt.value, variables)};'
                        lines.append(f'    {line}')
                    else:
                        val = cpp_literal(stmt.value)
                        lines.append(f'    {var_type} {stmt.name} = {val};')

                elif isinstance(stmt, AssignmentNode):
                    code = f'{stmt.name} = {merge_parts(stmt.value, variables, for_string=(variables.get(stmt.name) == 'str'))};'
                    lines.append(f'    {code}')

                elif isinstance(stmt, AugmentedAssignmentNode):
                    if stmt.name not in variables:
                        raise Exception(f'Variable {stmt.name} not declared')

                    var_type = variables[stmt.name]

                    if stmt.operator == '%' and var_type == 'float':
                        raise Exception(f"Cannot use '%' for float var '{stmt.name}'")

                    expr = generate_expr(stmt.value, variables)
                    lines.append(f'    {stmt.name} {stmt.operator}= {expr};')

                elif isinstance(stmt, IncrementNode):
                    lines.append(f'    {stmt.name}{stmt.operator};')

                elif isinstance(stmt, PrintNode):
                    output = '    cout << '
                    if stmt.value == '':
                        stmt.value = '""'
                    output += merge_parts(stmt.value, variables)

                    if isinstance(stmt.end, list):
                        output += f' << {merge_parts(stmt.end, variables)}'
                    elif isinstance(stmt.end, VarReferenceNode):
                        output += f' << {stmt.end.name}'
                    else:
                        output += f' << {cpp_literal(stmt.end)}'

                    output += ';'
                    lines.append(output)

            lines.append('    return 0;')
            lines.append('}')

    return '\n'.join(lines)

def cpp_type(var_type: str) -> str:
    return {
        'int': 'int',
        'float': 'float',
        'bool': 'bool',
        'str': 'string',
    }[var_type]

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
    else:
        return str(val)

def merge_parts(value, variables, for_string=False):
    if isinstance(value, list):
        if len(value) == 0:
            return '""'
        parts = []

        for part in value:
            if isinstance(part, VarReferenceNode):
                if part.name in variables and variables[part.name] == 'bool':
                    parts.append(f'({part.name} ? "true" : "false")')
                else:
                    parts.append(part.name)
            elif isinstance(part, ExpressionNode):
                parts.append(f'to_string({generate_expr(part, variables)})')
            elif isinstance(part, bool):
                parts.append('"true"' if part else '"false"')
            elif isinstance(part, (int, float)):
                parts.append(f'to_string({part})')
            else:
                parts.append(f'"{part}"')

        # ВАЖНО: внутри строки нужно +
        return ' + '.join(parts) if for_string else ' << '.join(parts)

    elif isinstance(value, VarReferenceNode):
        if value.name in variables and variables[value.name] == 'bool':
            return f'({value.name} ? "true" : "false")'
        else:
            return value.name

    elif isinstance(value, ExpressionNode):
        return generate_expr(value, variables)

    elif isinstance(value, bool):
        return '"true"' if value else '"false"'

    elif isinstance(value, (int, float)):
        return f'to_string({value})'

    else:
        return cpp_literal(value)

def generate_expr(expr, variables):
    if isinstance(expr, ExpressionNode):
        if expr.operator is None:
            return generate_expr(expr.left, variables)
        else:
            left = generate_expr(expr.left, variables)
            right = generate_expr(expr.right, variables)
            return f'({left} {expr.operator} {right})'

    elif isinstance(expr, VarReferenceNode):
        return expr.name

    elif isinstance(expr, bool):
        return '"true"' if expr else '"false"'

    elif isinstance(expr, (int, float)):
        return str(expr)

    else:
        raise Exception(f'Unknown expression part: {expr}')