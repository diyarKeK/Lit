import re

from lexer import tokenize
from nodes import *

class Parser:
    def __init__(self, tokens):
        self.tokens = tokens
        self.pos = 0

    def current(self):
        if self.pos < len(self.tokens):
            return self.tokens[self.pos]
        return None

    def eat(self, expected_type):
        token = self.current()
        if token and token.type == expected_type:
            self.pos += 1
            return token
        raise SyntaxError(f'Expected {expected_type}, got {token}')

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
            else:
                raise SyntaxError(f'Unexpected Token: {self.current()}')

        self.eat('RIGHT_BRACE')
        return Program(body=[Main_Function(body=body)])




    def parse_var_declaration(self):
        var_type_token = self.eat(self.current().type)
        name_token = self.eat('IDENTIFIER')
        self.eat('ASSIGNMENT')
        value = None

        if var_type_token.type == 'INT':
            value = self.parse_expression()
        elif var_type_token.type == 'FLOAT':
            value = self.parse_expression()
        elif var_type_token.type == 'BOOL':
            token = self.eat(self.current().type)
            value = True if token.type == 'TRUE' else False
        elif var_type_token.type == 'STR':
            token = self.eat('LITERAL_STRING').value[1:-1].replace('"', '\\"').replace("\\'", "'")
            value = self.parse_interpolated_string(token)
        else:
            raise SyntaxError(f'Invalid Variable Type: {var_type_token}')

        return VarDeclarationNode(var_type_token.type.lower(), name_token.value, value)



    def parse_print(self):
        self.eat('PRINT')
        self.eat('LEFT_BRACKET')


        token = self.current().type

        if token == 'LITERAL_STRING':
            raw = self.eat('LITERAL_STRING').value[1:-1].replace('"', '\\"').replace("\\'", "'")
            value = self.parse_interpolated_string(raw)

        elif token == 'IDENTIFIER':
            value = VarReferenceNode(self.eat('IDENTIFIER').value)

        else:
            value = self.parse_expression()

        end = '\\n'
        if self.current() and self.current().type == 'COMMA':
            self.eat('COMMA')
            self.eat('END')
            self.eat('ASSIGNMENT')

            if self.current().type == 'LITERAL_STRING':
                raw = self.eat('LITERAL_STRING').value[1:-1].replace('"', '\\"').replace("\\'", "'")
                end = self.parse_interpolated_string(raw)
            elif self.current().type == 'IDENTIFIER':
                name = self.eat('IDENTIFIER').value
                end = VarReferenceNode(name)
            elif self.current().type in ('LITERAL_INT', 'LITERAL_FLOAT', 'TRUE', 'FALSE'):
                val = self.eat(self.current().type).value
                if self.current().type == 'TRUE':
                    end = True
                elif self.current().type == 'FALSE':
                    end = False
                elif self.current().type == 'LITERAL_INT':
                    end = int(val)
                elif self.current().type == 'LITERAL_FLOAT':
                    end = float(val)
            else:
                raise SyntaxError(f'What did you write for end= ?')

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
        tok = self.current().type
        if tok == 'LITERAL_INT':
            return ExpressionNode(left=int(self.eat(tok).value), operator=None, right=None)
        elif tok == 'LITERAL_FLOAT':
            return ExpressionNode(left=float(self.eat(tok).value), operator=None, right=None)
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

    def parse_interpolated_string(self, text: str):
        parts = []
        cursor = 0

        for match in re.finditer(r'\{([^{}]+)\}', text):
            start, end = match.span()

            if start > cursor:
                parts.append(text[cursor:start])

            inner_code = match.group(1).strip()
            tokens = tokenize(inner_code)
            expr_parser = Parser(tokens)
            expr = expr_parser.parse_expression()
            parts.append(expr)

            cursor = end

        if cursor < len(text):
            parts.append(text[cursor:])

        return parts