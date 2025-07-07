import re

def optimize(code: str, file_name: str):
    code = remove_comments(code)
    code = remove_underscores_in_numbers(code)
    return code

def remove_comments(code: str) -> str:
    code = re.sub(r'/\*.*?\*/', '', code, flags=re.DOTALL)
    code = re.sub(r'//.*', '', code)
    return code

def remove_underscores_in_numbers(code: str) -> str:
    def clean_number(match):
        number = match.group(0)
        return number.replace('_', '')

    return re.sub(r'\b\d[\d_]*(?:\.[\d_]+)?\b', clean_number, code)