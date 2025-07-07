import re

from lexer import tokenize
from nodes import *

class Parser:
    def __init__(self, tokens, file_name):
        self.tokens = tokens
        self.pos = 0
        self.file_name = file_name

    def current(self):
        if self.pos < len(self.tokens):
            return self.tokens[self.pos]
        return None

    def eat(self, expected_type):
        token = self.current()
        if token and token.type == expected_type:
            self.pos += 1
            return token
        elif token:
            raise SyntaxError(f'{self.file_name}:{token.line}: Expected {expected_type}, got {token.type} = {token.value}')
        else:
            raise SyntaxError(f'{self.file_name}: Unexpected end of input (expected {expected_type})')

    def parse(self):

        self.eat('FUN')
        self.eat('MAIN')
        self.eat('LEFT_BRACKET')
        self.eat('RIGHT_BRACKET')
        self.eat('LEFT_BRACE')

        body = []
        while self.current() and self.current().type != 'RIGHT_BRACE':
            if self.current().type in ('INT', 'FLOAT', 'BOOL', 'STR'):
                body.append(self.parse_var_declaration())
            elif self.current().type == 'PRINT':
                body.append(self.parse_print())
            elif self.current().type == 'IDENTIFIER':
                body.append(self.parse_assignment_or_expression())
            else:
                raise SyntaxError(f'Unexpected Token: {self.current()}')

        self.eat('RIGHT_BRACE')
        return Program(body=[Main_Function(body=body)])




    def parse_var_declaration(self):
        var_type_token = self.eat(self.current().type)
        type_suffix = None
        if self.current() and self.current().type == 'TYPE_SUFFIX':
            type_suffix = self.eat('TYPE_SUFFIX').value[1:]
        name_token = self.eat('IDENTIFIER')
        value = None

        if self.current() and self.current().type == 'ASSIGNMENT':
            self.eat('ASSIGNMENT')

            token = self.current().type
            if token == 'LITERAL_STRING':
                value = self.parse_expression()
            elif token == 'TRUE':
                self.eat('TRUE')
                value = True
            elif token == 'FALSE':
                self.eat('FALSE')
                value = False
            else:
                value = self.parse_expression()

        return VarDeclarationNode(var_type_token.type.lower(), name_token.value, value, type_suffix)



    def parse_print(self):
        self.eat('PRINT')
        self.eat('LEFT_BRACKET')


        token = self.current().type
        value = self.parse_expression()

        end = '\\n'
        if self.current() and self.current().type == 'COMMA':
            self.eat('COMMA')
            self.eat('END')
            self.eat('ASSIGNMENT')
            end = self.parse_expression()

        self.eat('RIGHT_BRACKET')
        return PrintNode(value=value, end=end)

    def parse_expression(self):
        return self.parse_term()

    def parse_term(self):
        node = self.parse_factor()
        while self.current() and self.current().type in ('PLUS', 'MINUS'):
            op = self.eat(self.current().type).value
            right = self.parse_factor()
            node = ExpressionNode(left=node, operator=op, right=right)
        return node

    def parse_factor(self):
        node = self.parse_atom()
        while self.current() and self.current().type in ('MULTIPLY', 'DIVIDE', 'MODULO'):
            op = self.eat(self.current().type).value
            right = self.parse_atom()
            node = ExpressionNode(left=node, operator=op, right=right)
        return node

    def parse_atom(self):
        if self.current().type == 'MINUS':
            self.eat('MINUS')
            atom = self.parse_atom()

            if isinstance(atom, ExpressionNode) and atom.operator is None:
                atom.left = -atom.left
                return atom
            elif isinstance(atom, VarReferenceNode):
                return ExpressionNode(left=0, operator='-', right=atom)
            else:
                raise SyntaxError('Invalid usage of unary minus')

        tok = self.current().type
        if tok == 'LITERAL_INT':
            return ExpressionNode(left=int(self.eat(tok).value), operator=None, right=None)
        elif tok == 'LITERAL_FLOAT':
            return ExpressionNode(left=float(self.eat(tok).value), operator=None, right=None)
        elif tok == 'LITERAL_STRING':
            raw = self.eat('LITERAL_STRING').value[1:-1].replace('"', '\\"').replace("\\'", "'")
            parts = self.parse_interpolated_string(raw)
            return self.build_interpolated_expr(parts)
        elif tok == 'TRUE':
            self.eat(tok)
            return ExpressionNode(left=True, operator=None, right=None)
        elif tok == 'FALSE':
            self.eat(tok)
            return ExpressionNode(left=False, operator=None, right=None)
        elif tok == 'IDENTIFIER':
            return VarReferenceNode(self.eat(tok).value)
        elif tok == 'LEFT_BRACKET':
            self.eat('LEFT_BRACKET')
            expr = self.parse_expression()
            self.eat('RIGHT_BRACKET')
            return expr
        else:
            raise SyntaxError('Unexpected token in expression')

    def build_interpolated_expr(self, parts):
        if not parts:
            return ExpressionNode(left='""', operator=None, right=None)

        expr = None
        for part in parts:
            node = part if isinstance(part, ExpressionNode) else ExpressionNode(left=part, operator=None, right=None)
            if expr is None:
                expr = node
            else:
                expr = ExpressionNode(left=expr, operator='+', right=node)
        return expr

    def parse_interpolated_string(self, text: str):
        parts = []
        cursor = 0

        text = text.replace('{{', '\x01').replace('}}', '\x02')

        for match in re.finditer(r'\{([^{}]+)\}', text):
            start, end = match.span()

            if start > cursor:
                segment = text[cursor:start]

                segment = segment.replace('\x01', '{').replace('\x02', '}')
                parts.append(segment)

            inner_code = match.group(1).strip()
            tokens = tokenize(inner_code)
            expr_parser = Parser(tokens, self.file_name)
            expr = expr_parser.parse_expression()
            parts.append(expr)

            cursor = end

        if cursor < len(text):
            tail = text[cursor:]
            tail = tail.replace('\x01', '{').replace('\x02', '}')
            parts.append(tail)

        return parts

    def parse_assignment_or_expression(self):
        name = self.eat('IDENTIFIER').value
        current_type = self.current().type

        if current_type == 'ASSIGNMENT':
            self.eat('ASSIGNMENT')

            token = self.current().type
            if token == 'LITERAL_STRING':
                value = self.parse_expression()
            elif token in ('LITERAL_INT', 'LITERAL_FLOAT', 'TRUE', 'FALSE'):
                if token == 'TRUE':
                    self.eat('TRUE')
                    value = True
                elif token == 'FALSE':
                    self.eat('FALSE')
                    value = False
                else:
                    value = self.eat(token).value
                    if isinstance(value, str):
                        value = int(value) if '.' not in value else float(value)
            else:
                value = self.parse_expression()

            return AssignmentNode(name=name, value=value)

        elif current_type == 'PLUS_ASSIGNMENT':
            self.eat('PLUS_ASSIGNMENT')
            if self.current().type == 'LITERAL_STRING':
                raw = self.eat('LITERAL_STRING').value[1:-1].replace('"', '\\"').replace("\\'", "'")
                value = self.parse_interpolated_string(raw)
            else:
                value = self.parse_expression()
            return AugmentedAssignmentNode(name=name, operator='+', value=value)

        elif current_type == 'MINUS_ASSIGNMENT':
            self.eat('MINUS_ASSIGNMENT')
            value = self.parse_expression()
            return AugmentedAssignmentNode(name=name, operator='-', value=value)

        elif current_type == 'MULTIPLY_ASSIGNMENT':
            self.eat('MULTIPLY_ASSIGNMENT')
            value = self.parse_expression()
            return AugmentedAssignmentNode(name=name, operator='*', value=value)

        elif current_type == 'DIVIDE_ASSIGNMENT':
            self.eat('DIVIDE_ASSIGNMENT')
            value = self.parse_expression()
            return AugmentedAssignmentNode(name=name, operator='/', value=value)

        elif current_type == 'MODULO_ASSIGNMENT':
            self.eat('MODULO_ASSIGNMENT')
            value = self.parse_expression()
            return AugmentedAssignmentNode(name=name, operator='%', value=value)

        elif current_type == 'INCREMENT':
            self.eat('INCREMENT')
            return IncrementNode(name=name, operator='++')

        elif current_type == 'DECREASE':
            self.eat('DECREASE')
            return IncrementNode(name=name, operator='--')

        else:
            raise SyntaxError(f'Unknown assignment or expression for {name}')