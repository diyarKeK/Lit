from dataclasses import dataclass
from typing import Union, List

@dataclass
class Program:
    body: List

@dataclass
class Main_Function:
    body: List


@dataclass
class VarDeclarationNode:
    var_type: str
    name: str
    value: Union[str, int, float, bool, List[Union[str, 'VarReferenceNode']]]

@dataclass
class ExpressionNode:
    left: Union['ExpressionNode', 'VarReferenceNode', int, float]
    operator: str
    right: Union['ExpressionNode', 'VarReferenceNode', int, float]

@dataclass
class VarReferenceNode:
    name: str

@dataclass
class PrintNode:
    value: Union[str, int, float, bool, VarReferenceNode, List[Union[str, VarReferenceNode]]]
    end: Union[str, int, float, bool, VarReferenceNode] = '\\n'