import re

def optimize(code: str, file_name: str):
    code = remove_comments(code)
    return code

def remove_comments(code: str) -> str:
    code = re.sub(r'/\*.*?\*/', '', code, flags=re.DOTALL)
    code = re.sub(r'//.*', '', code)
    return code