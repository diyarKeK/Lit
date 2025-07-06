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
    ('TRUE', r'\btrue\b'),
    ('FALSE', r'\bfalse\b'),
    ('STR', r'\bstr\b'),

    ('ASSIGNMENT', r'='),

    ('LITERAL_FLOAT', r'\d+\.\d+'),
    ('LITERAL_INT', r'\d+'),
    ('LITERAL_STRING', r"'[^']*'"),

    ('PLUS', r'\+'),
    ('MINUS', r'-'),
    ('MULTIPLY', r'\*'),
    ('DIVIDE', r'/'),
    ('MODULO', r'%'),

    ('PRINT', r'\bprint\b'),
    ('END', r'\bend\b'),
    ('COMMA', r','),

    ('IDENTIFIER', r'\b[a-zA-Z_][a-zA-Z0-9_]*\b'),
    ('UNKNOWN', r'.'),
]

class Token:
    def __init__(self, type_, value):
        self.type = type_
        self.value = value

    def __repr__(self):
        return f'Token({self.type}) = {self.value}'

def tokenize(code: str):
    tokens = []
    i = 0
    unknown_statements_count = 0

    while i < len(code):
        for tok_type, pattern in TOKEN_TYPES:
            regex = re.compile(pattern)
            match = regex.match(code, i)

            if match:
                text = match.group(0)
                if tok_type != 'SKIP' and tok_type != 'NEW_LINE':
                    tokens.append(Token(tok_type, text))
                    print(f'Token({tok_type}) = {text}')
                elif tok_type == 'UNKNOWN':
                    unknown_statements_count += 1

                i += len(text)
                break

    print(f'Unknown Statements Count = {unknown_statements_count}')
    return tokens