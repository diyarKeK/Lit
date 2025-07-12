from dataclasses import dataclass
from typing import Union, List, Optional


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
    value: Union[str, int, float, bool, 'InputNode', 'VarReferenceNode', List[Union[str, 'VarReferenceNode']]]
    suffix: str = None

@dataclass
class AssignmentNode:
    name: str
    value: Union[str, int, float, bool, 'VarReferenceNode', 'ExpressionNode', List]


@dataclass
class AugmentedAssignmentNode:
    name: str
    operator: str
    value: Union[int, float, 'VarReferenceNode', 'ExpressionNode']

@dataclass
class IncrementNode:
    name: str
    operator: str

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

@dataclass
class InputNode:
    variable: VarReferenceNode
    message: Union[str, VarReferenceNode, List[Union[str, VarReferenceNode]]]

@dataclass
class IfNode:
    condition: 'ConditionNode'
    body: List
    elif_blocks: List['ElifBlock'] = None
    else_body: Optional[List] = None

@dataclass
class ElifBlock:
    condition: 'ConditionNode'
    body: List

@dataclass
class ConditionNode:
    left: Union['ConditionNode', VarReferenceNode, int, float, bool, str]
    operator: str = None
    right: Union['ConditionNode', VarReferenceNode, int, float, bool, str] = None
    negate: bool = False
