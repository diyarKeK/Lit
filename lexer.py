import  re

TOKEN_TYPES = [
    ('FUN', r'\bfun\b'),
    ('MAIN', r'\bmain\b'),
    ('LEFT_BRACKET', r'\('),
    ('RIGHT_BRACKET', r'\)'),
    ('LEFT_BRACE', r'\{'),
    ('RIGHT_BRACE', r'\}'),

    ('NEW_LINE', r'\n'),
    ('SKIP', r'[ \t]+'),

    ('INT', r'\bint\b'),
    ('FLOAT', r'\bfloat\b'),
    ('BOOL', r'\bbool\b'),
    ('STR', r'\bstr\b'),
    ('TYPE_SUFFIX', r':[iu](8|16|32|64)|:f(32|64)'),

    ('PLUS_ASSIGNMENT', r'\+='),
    ('MINUS_ASSIGNMENT', r'-='),
    ('MULTIPLY_ASSIGNMENT', r'\*='),
    ('DIVIDE_ASSIGNMENT', r'/='),
    ('MODULO_ASSIGNMENT', r'%='),
    ('INCREMENT', r'\+\+'),
    ('DECREASE', r'--'),

    ('LITERAL_FLOAT', r'\d+\.\d+'),
    ('LITERAL_INT', r'\d+'),
    ('TRUE', r'\btrue\b'),
    ('FALSE', r'\bfalse\b'),
    ('LITERAL_STRING', r"'([^'\\]|\\.)*'"),

    ('PLUS', r'\+'),
    ('MINUS', r'-'),
    ('MULTIPLY', r'\*'),
    ('DIVIDE', r'/'),
    ('MODULO', r'%'),

    ('PRINT', r'\bprint\b'),
    ('END', r'\bend\b'),
    ('COMMA', r','),

    ('INPUT', r'\binput\b'),

    ('IF', r'\bif\b'),
    ('ELSE', r'\belse\b'),
    ('AND', r'\band\b'),
    ('OR', r'\bor\b'),
    ('NOT', r'\bnot\b'),
    ('EQUALS', r'=='),
    ('NOT_EQUALS', r'!='),
    ('LESS_EQUALS', r'<='),
    ('GREATER_EQUALS', r'>='),
    ('LESS', r'<'),
    ('GREATER', r'>'),
    ('ASSIGNMENT', r'='),

    ('IDENTIFIER', r'\b[a-zA-Z_][a-zA-Z0-9_]*\b'),
    ('UNKNOWN', r'.'),
]

class Token:
    def __init__(self, type_, value, line, column):
        self.type = type_
        self.value = value
        self.line = line
        self.column = column

    def __repr__(self):
        return f'{self.line}:{self.column}: Token({self.type}) = {self.value}'

def tokenize(code: str):
    tokens = []
    i = 0
    line = 1
    column = 1
    unknown_statements_count = 0

    while i < len(code):
        for tok_type, pattern in TOKEN_TYPES:
            regex = re.compile(pattern)
            match = regex.match(code, i)

            if match:
                text = match.group(0)

                if tok_type == 'NEW_LINE':
                    line += 1
                    column += 1
                else:
                    token = Token(tok_type, text, line, column)
                    if tok_type == 'UNKNOWN':
                        unknown_statements_count += 1
                        tokens.append(token)
                        print(f'{token.__repr__()} <- IS UNKNOWN')
                    elif tok_type != 'SKIP' and tok_type != 'NEW_LINE':
                        tokens.append(token)
                        print(f'{token.__repr__()}')

                    column += len(text)

                i += len(text)
                break

    print(f'Unknown Statements Count = {unknown_statements_count}')
    return tokens