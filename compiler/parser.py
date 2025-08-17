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
            elif self.current().type == 'IF':
                body.append(self.parse_if())
            elif self.current().type == 'WHILE':
                body.append(self.parse_while())
            elif self.current().type == 'FOR':
                body.append(self.parse_for())
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

            if self.current().type == 'INPUT':
                self.eat('INPUT')
                self.eat('LEFT_BRACKET')
                message = self.parse_expression()
                self.eat('RIGHT_BRACKET')
                value = InputNode(variable=VarReferenceNode(name_token.value), message=message)
            else:
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
        node = self.parse_unary()
        while self.current() and self.current().type in ('MULTIPLY', 'DIVIDE', 'MODULO'):
            op = self.eat(self.current().type).value
            right = self.parse_unary()
            node = ExpressionNode(left=node, operator=op, right=right)
        return node

    def parse_unary(self):
        if self.current().type == 'NOT':
            self.eat('NOT')
            operand = self.parse_unary()
            return ExpressionNode(left=None, operator='not', right=operand)
        return self.parse_atom()

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
            expr = self.parse_condition()
            self.eat('RIGHT_BRACKET')
            return expr
        elif tok == 'RIGHT_BRACKET':
            return ExpressionNode(left='', operator=None, right=None)
        else:
            raise SyntaxError(f'Unexpected token in expression {tok}')

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
                if token == 'LITERAL_INT':
                    raw_value = self.eat(token).value
                    value = ExpressionNode(left=int(raw_value), operator=None, right=None)
                elif token == 'LITERAL_FLOAT':
                    raw_value = self.eat(token).value
                    value = ExpressionNode(left=float(raw_value), operator=None, right=None)
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

    def parse_if(self):
        self.eat('IF')
        condition = self.parse_condition()

        if self.current().type == 'LEFT_BRACE':
            self.eat('LEFT_BRACE')
            body = self.parse_block()
        else:
            body = [self.parse_single_statement()]

        elif_blocks = []
        else_body = None

        while self.current() and self.current().type == 'ELSE':
            self.eat('ELSE')

            if self.current().type == 'IF':
                self.eat('IF')
                elif_condition = self.parse_condition()
                if self.current().type == 'LEFT_BRACE':
                    self.eat('LEFT_BRACE')
                    elif_body = self.parse_block()
                else:
                    elif_body = [self.parse_single_statement()]
                elif_blocks.append(ElifBlock(condition=elif_condition, body=elif_body))
            else:
                if self.current().type == 'LEFT_BRACE':
                    self.eat('LEFT_BRACE')
                    else_body = self.parse_block()
                else:
                    else_body = [self.parse_single_statement()]
                break

        return IfNode(condition=condition, body=body, elif_blocks=elif_blocks, else_body=else_body)

    def parse_while(self):
        self.eat('WHILE')
        condition = self.parse_condition()

        if self.current().type == 'LEFT_BRACE':
            self.eat('LEFT_BRACE')
            body = self.parse_block()
        else:
            body = self.parse_single_statement()

        else_body = None

        if self.current() and self.current().type == 'ELSE':
            self.eat('ELSE')
            if self.current().type == 'LEFT_BRACE':
                self.eat('LEFT_BRACE')
                else_body = self.parse_block()
            else:
                else_body = [self.parse_single_statement()]

        return WhileNode(condition=condition, body=body, else_body=else_body)

    def parse_for(self):
        pass

    def parse_block(self):
        body = []
        while self.current() and self.current().type != 'RIGHT_BRACE':
            if self.current().type in ('INT', 'FLOAT', 'BOOL', 'STR'):
                body.append(self.parse_var_declaration())
            elif self.current().type == 'PRINT':
                body.append(self.parse_print())
            elif self.current().type == 'IF':
                body.append(self.parse_if())
            elif self.current().type == 'IDENTIFIER':
                body.append(self.parse_assignment_or_expression())
            else:
                raise SyntaxError(f'Unexpected token in block: {self.current()}')

        self.eat('RIGHT_BRACE')
        return body

    def parse_single_statement(self):
        if self.current().type == 'PRINT':
            return self.parse_print()
        elif self.current().type == 'IF':
            return self.parse_if()
        elif self.current().type == 'IDENTIFIER':
            return self.parse_assignment_or_expression()
        elif self.current().type in ('INT', 'FLOAT', 'STR', 'BOOL'):
            return self.parse_var_declaration()
        else:
            raise SyntaxError(f'Unexpected token in single-line if-body: {self.current()}')

    def parse_condition(self):
        return self.parse_or()

    def parse_or(self):
        node = self.parse_and()
        while self.current() and self.current().type == 'OR':
            self.eat('OR')
            right = self.parse_and()
            node = ConditionNode(left=node, operator='or', right=right)
        return node

    def parse_and(self):
        node = self.parse_not()
        while self.current() and self.current().type == 'AND':
            self.eat('AND')
            right = self.parse_not()
            node = ConditionNode(left=node, operator='and', right=right)
        return node

    def parse_not(self):
        if self.current() and self.current().type == 'NOT':
            self.eat('NOT')
            operand = self.parse_not()
            return ConditionNode(left='not', operator='not', right=operand)
        return self.parse_comparison()

    def parse_comparison(self):
        node = self.parse_expression()
        while self.current() and self.current().type in ('EQUALS', 'NOT_EQUALS', 'LESS', 'GREATER', 'LESS_EQUALS', 'GREATER_EQUALS'):
            op = self.eat(self.current().type).value
            right = self.parse_expression()
            node = ConditionNode(left=node, operator=op, right=right)
        return node

