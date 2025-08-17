import time
from pprint import pprint
import shlex
import sys


class LVM:
    def __init__(self, bytecode, path):
        self.bytecode = bytecode
        self.call_stack = []
        self.classes = {}
        self.class_positions = {}
        self.current_class = None
        self.frame_stack = [{}]
        self.ip = 0
        self.labels = {}
        self.path = path
        self.stack = []
        self.this = None
        self.try_stack = []

    def collect_labels_and_classes(self):
        for idx, line in enumerate(self.bytecode):
            parts = self.parse_line(line)

            if len(parts) == 0:
                continue

            op = parts[0].upper()

            if op == 'LABEL':
                label_name = parts[1]
                self.labels[label_name] = idx
            elif op == 'CLASS':
                class_name = parts[1]
                self.class_positions[class_name] = idx

    def load_class_if_needed(self, class_name, from_class=None):
        if class_name in self.classes:
            return

        if class_name not in self.class_positions:
            print(f'Class: {class_name} is not found')
            exit(1)

        start_index = self.class_positions[class_name]
        idx = start_index
        self.current_class = None

        while idx < len(self.bytecode):
            raw_line = self.bytecode[idx]
            if not raw_line.strip():
                continue

            instr = self.parse_line(raw_line)
            op = instr[0].upper()

            if op == 'CLASS':
                current_class = instr[1]

                self.classes[current_class] = {
                    "fields": {},
                    "methods": {},
                    "static_fields": {},
                    "static_init": None,
                    "static_initialized": False,
                    "static_methods": {},
                    "super_class": None,
                    "interfaces": [],
                    "generics": []
                }

                self.current_class = current_class

            elif op == 'EXTENDS':
                super_class = instr[1]

                self.load_class_if_needed(super_class, from_class=f'{self.current_class}')

                self.classes[self.current_class]["super_class"] = super_class

            elif op == 'IMPLEMENTS':
                values = " ".join(instr[1:])
                interfaces = [p.strip() for p in values.split(',') if p.strip()]

                for interface in interfaces:
                    self.load_class_if_needed(interface, from_class=f'{self.current_class}')

                self.classes[self.current_class]["interfaces"] = interfaces

            elif op == 'GENERIC':
                g_name = instr[1]

                self.classes[self.current_class]["generics"].append(g_name)

            elif op == 'FIELD':
                f_type = instr[1]
                f_name = instr[2]

                self.classes[self.current_class]["fields"][f_name] = f_type

            elif op == 'STATIC_FIELD':

                f_type = instr[1]
                f_name = instr[2]

                self.classes[self.current_class]["static_fields"][f_name] = (f_type, None)

            elif op == 'STATIC_INIT':

                label = instr[1]
                self.classes[self.current_class]["static_init"] = label
                self.classes[self.current_class]["static_initialized"] = False

            elif op == 'METHOD':

                m_name = instr[1]
                m_label = instr[2]

                self.classes[self.current_class]["methods"][m_name] = m_label

            elif op == 'STATIC_METHOD':

                m_name = instr[1]
                m_label = instr[2]

                self.classes[self.current_class]["static_methods"][m_name] = m_label

            elif op == 'END_CLASS':
                super_class = self.classes[self.current_class]["super_class"]

                if super_class:
                    parent = self.classes[super_class]

                    for f_name, f_type in parent["fields"].items():
                        self.classes[self.current_class]["fields"][f_name] = f_type

                    for m_name, m_label in parent["methods"].items():
                        self.classes[self.current_class]["methods"][m_name] = m_label

                    for static_f_name, static_f_type in parent["static_fields"].items():
                        self.classes[self.current_class]["static_fields"][static_f_name] = (static_f_type, None)

                    for static_m_name, static_m_label in parent["static_methods"].items():
                        self.classes[self.current_class]["static_methods"][static_m_name] = static_m_label

                for interface in (self.classes[self.current_class]["interfaces"] or []):
                    iface = self.classes[interface]
                    for m_name, m_label in iface["methods"].items():
                        self.classes[self.current_class]["methods"][m_name] = m_label

                self.current_class = from_class
                break
            else:
                print(f'Not Class Instruction: {op}, at {self.path}:{idx}:\n    {raw_line}')
                exit(1)

            idx += 1

    def init_static_fields_if_needed(self, class_name):
        if not self.classes[class_name]["static_init"] or self.classes[class_name]["static_initialized"]:
            return

        label = self.classes[class_name]["static_init"]
        self.call_stack.append(self.ip)
        self.frame_stack.append({})
        self.classes[class_name]["static_initialized"] = True
        self.ip = self.labels[label] + 1

        for i in range(1000):
            raw_line = self.bytecode[self.ip]

            if not raw_line.strip():
                self.ip += 1
                continue

            instr = self.parse_line(raw_line)
            self.execute()
            if instr[0].upper() == 'RET':
                break
        else:
            print('To Big Code!!! Your static initializer has more than 1000 line of code!')
            exit(1)

    def current_frame(self):
        return self.frame_stack[-1]

    def parse_line(self, line):
        parts = shlex.split(line.strip())
        return tuple(parts)

    def run(self):
        self.collect_labels_and_classes()
        self.ip = self.labels["main"] + 1

        while self.ip < len(self.bytecode):
            self.execute()

    def execute(self):
        raw_line: str = self.bytecode[self.ip].strip()
        self.ip += 1

        if not raw_line:
            return

        if raw_line.startswith(';') or raw_line.startswith('#'):
            return

        instr = self.parse_line(raw_line)
        op = instr[0].upper()

        if op == 'PUSH_CONST':
            dtype = instr[1]
            raw_value: str = instr[2]
            value = ''

            if dtype == 'int':
                value = int(raw_value)
            elif dtype == 'float':
                value = float(raw_value)
            elif dtype == 'bool':
                value = raw_value.lower() == 'true'
            elif dtype == 'str':
                value = raw_value.strip('"').replace('\\n', '\n')
            elif dtype == 'lambda':
                value = raw_value
            elif dtype == 'object':
                if raw_value == 'null':
                    value = None
                else:
                    print(f'Unsupported object constant: {raw_value}, at {self.path}:{self.ip}:\n    {raw_line}')
                    exit(1)

            self.stack.append((dtype, value))

        elif op in ('INC', 'DEC'):
            dtype, value = self.stack.pop()

            if dtype not in ('int', 'float'):
                print(f'Cannot increment or decrement non-numeric value: {value}, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            self.stack.append((dtype, value + 1 if op == 'INC' else value - 1))

        elif op in ('ADD_VAR', 'SUB_VAR', 'MUL_VAR', 'DIV_VAR', 'MOD_VAR'):
            var_name = instr[1]

            if var_name not in self.current_frame():
                print(f'Undefined variable: {var_name}, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            t2, b = self.stack.pop()
            t1, a = self.current_frame()[var_name]

            if t1 == 'str':
                if op != 'ADD_VAR':
                    print(f'Cannot use {op} on str, at {self.path}:{self.ip}:\n    {raw_line}')
                    exit(1)

                if t2 != 'str':
                    b = str(b)

                self.current_frame()[var_name] = ('str', a + b)

            else:
                if t1 not in ('int', 'float') or t2 not in ('int', 'float'):
                    print(f'Type Error: {t1} {op} {t2}, at {self.path}:{self.ip}:\n    {raw_line}')
                    exit(1)

                if op == 'ADD_VAR':
                    res = a + b
                elif op == 'SUB_VAR':
                    res = a - b
                elif op == 'MUL_VAR':
                    res = a * b
                elif op == 'DIV_VAR':
                    res = a / b
                elif op == 'MOD_VAR':
                    if t1 == 'float' and t2 == 'float':
                        print(f'Cannot use %= with float, at {self.path}:{self.ip}:\n    {raw_line}')
                        exit(1)

                    res = a % b
                else:
                    res = None

                new_type = 'float' if t1 == 'float' or t2 == 'float' else 'int'
                self.current_frame()[var_name] = (new_type, res)

        elif op in ('ADD', 'SUB', 'MUL', 'DIV', 'MOD'):
            t2, b = self.stack.pop()
            t1, a = self.stack.pop()

            if t1 not in ('int', 'float') or t2 not in ('int', 'float'):
                print(f'Type Error: {t1} {op} {t2}, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            res = ''

            if op == 'ADD':
                res = a + b
            elif op == 'SUB':
                res = a - b
            elif op == 'MUL':
                res = a * b
            elif op == 'DIV':
                res = a / b
            elif op == 'MOD':
                res = a % b

            result_type = 'float' if t1 == 'float' or t2 == 'float' else 'int'
            self.stack.append((result_type, res))

        elif op == 'ADD_STR':
            t2, b = self.stack.pop()
            t1, a = self.stack.pop()

            if t1 != 'str':
                a = str(a)
            if t2 != 'str':
                b = str(b)

            self.stack.append(('str', a + b))

        elif op == 'STORE_VAR':
            if not self.stack:
                print(f'Error: empty stack before STORE_VAR {instr[1]}, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            var_name = instr[1]
            dtype, value = self.stack.pop()

            self.current_frame()[var_name] = (dtype, value)

        elif op == 'LOAD_VAR':
            var_name = instr[1]

            if var_name not in self.current_frame():
                print(f'Undefined variable: {var_name}, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            self.stack.append(self.current_frame()[var_name])

        elif op == 'PRINT':
            if len(self.stack) > 0:
                dtype, value = self.stack.pop()
            else:
                dtype, value = 'str', '\n'

            if dtype == 'array':
                elem_type, data = value
                print(data)

            elif dtype == 'object':
                if not value:
                    print('null')
                else:
                    pprint(value, indent=2)

            elif dtype == 'tuple':
                print('(', end='')

                for i in range(len(value)):
                    _, v = value[i]

                    if i < len(value) - 1:
                        print(v, end=', ')
                    else:
                        print(v, end='')

                print(')')

            elif dtype == 'bool':
                print('true' if value else 'false')

            else:
                print(value)

        elif op == 'INPUT':
            dtype = instr[1]
            prompt = instr[2] if len(instr) > 2 else ""

            user_input = input(prompt)
            value = ''

            if dtype == 'int':
                try:
                    value = int(user_input)
                except ValueError:
                    print(f'Invalid int input, at {self.path}:{self.ip}:\n    {raw_line}')
                    exit(1)
            elif dtype == 'float':
                try:
                    value = float(user_input)
                except ValueError:
                    print(f'Invalid float input, at {self.path}:{self.ip}:\n    {raw_line}')
                    exit(1)

            elif dtype == 'bool':
                value = user_input.lower() in ('true', '1', 'y', 'yes')
            else:
                value = user_input

            self.stack.append((dtype, value))

        elif op == 'TRY':
            catch_class = instr[1]
            catch_label = instr[2]

            self.load_class_if_needed(catch_class)

            if catch_label not in self.labels:
                print(f'Catch label: {catch_label} is not found, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            saved_frame = [frame.copy() for frame in self.frame_stack]

            self.try_stack.append((
                self.labels[catch_label] + 1,
                catch_class,
                saved_frame
            ))

        elif op == 'END_TRY':
            if self.try_stack:
                self.try_stack.pop()
            else:
                print('END_TRY used without TRY')
                exit(1)

        elif op == 'CALL':
            func_name = instr[1]

            if func_name not in self.labels:
                print(f'Function: {func_name} is not found, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            self.call_stack.append(self.ip)
            self.frame_stack.append({})

            self.ip = self.labels[func_name] + 1

        elif op == 'CALL_DYNAMIC':
            dtype, label_name = self.stack.pop()
                
            if dtype != 'lambda':
                print(f'Expected lambda, got {dtype}, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            if label_name not in self.labels:
                print(f'Lambda {label_name} not found, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            self.call_stack.append(self.ip)
            self.frame_stack.append({})

            self.ip = self.labels[label_name] + 1

        elif op == 'RET':
            if not self.call_stack:
                print(f'RET without calling to him, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            self.frame_stack.pop()

            self.ip = self.call_stack.pop()

        elif op == 'THROW':
            t_obj, obj = self.stack.pop()

            if t_obj != 'object':
                print(f'Expected object for THROW, got {t_obj}, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            t_msg, msg = obj["fields"].get("description", (None, None))
            if not (t_msg and msg):
                print(f'Class: {obj["_class"]} is not Exception class, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            exception_class = obj["_class"]

            handled = False

            while self.try_stack:
                catch_ip, catch_class, saved_frame = self.try_stack.pop()

                if exception_class == catch_class:
                    self.frame_stack = [frame.copy() for frame in saved_frame]
                    self.ip = catch_ip
                    handled = True
                    break

            if handled:
                self.this = (t_obj, obj)
                return

            print(f'Error at {self.path}:{self.ip}:\n    Exception: {obj["_class"]}\n    Description: {msg}\n    In The Code: {self.path}:{self.ip}')
            exit(1)

        elif op == 'NEW':
            class_name = instr[1]
            init_label = instr[2]

            self.load_class_if_needed(class_name)

            if init_label not in self.labels:
                print(f'Init Label: {init_label} is not found, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            field_map = self.classes[class_name]["fields"]

            obj = {
                "_class": class_name,
                "fields": {n: (t, None) for n, t in field_map.items()}
            }

            self.call_stack.append(self.ip)
            self.frame_stack.append({})
            self.this = ('object', obj)

            self.ip = self.labels[init_label] + 1

        elif op == 'NEW_GENERIC_OBJ':
            class_name = instr[1]
            init_label = instr[2]
            generic_args = instr[3:]

            self.load_class_if_needed(class_name)

            class_info = self.classes[class_name]
            generic_names = class_info["generics"]

            if len(generic_names) != len(generic_args):
                print(f'Generic argument count mismatch for {class_name}, '
                      f'expected {len(generic_names)}, got {len(generic_args)}, '
                      f'at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            generic_map = dict(zip(generic_names, generic_args))

            field_map = {}
            for f_name, f_type in class_info["fields"].items():
                real_type = generic_map.get(f_type, f_type)
                field_map[f_name] = (real_type, None)

            obj = {
                "_class": class_name,
                "generics": generic_map,
                "fields": field_map
            }

            self.call_stack.append(self.ip)
            self.frame_stack.append({})
            self.this = ('object', obj)

            self.ip = self.labels[init_label] + 1

        elif op == 'INIT_FIELD':
            field_name = instr[1]

            t_obj, obj = self.stack.pop()
            t_val, val = self.stack.pop()

            if t_obj != 'object':
                print(f'Expected object for INIT_FIELD, got {t_obj}, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            if field_name not in obj["fields"]:
                print(f'Field: {field_name} not found in object of class {obj["_class"]}, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            if t_val != obj["fields"][field_name][0]:
                print(f'Field: {field_name} is {obj["fields"][field_name][0]}, got {t_val}, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            if obj["fields"][field_name][1]:
                print(f'Field: {field_name} already initialized, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            obj["fields"][field_name] = (t_val, val)

        elif op == 'UPDATE_FIELD':
            field_name = instr[1]

            t_obj, obj = self.stack.pop()
            t_val, val = self.stack.pop()

            if t_obj != 'object':
                print(f'Expected object for UPDATE_FIELD, got {t_obj}, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            if field_name not in obj["fields"]:
                print(f'Field: {field_name} is not found in object of class {obj["_class"]}, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            if t_val != obj["fields"][field_name][0]:
                print(f'Field: {field_name} is {obj["fields"][field_name][0]}, got {t_val}, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            obj["fields"][field_name] = (t_val, val)

        elif op == 'LOAD_FIELD':
            field_name = instr[1]
            t_obj, obj = self.stack.pop()

            if t_obj != 'object':
                print(f'Expected object for LOAD_FIELD, got {t_obj}, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            if field_name not in obj["fields"]:
                print(f'Field: {field_name} not found in object of class {obj["_class"]}, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            self.stack.append(obj["fields"][field_name])

        elif op == 'LOAD_THIS':
            if not self.this:
                print(f'LOAD_THIS used outside object context, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            self.stack.append(self.this)

        elif op == 'SET_STATIC_FIELD':
            class_name = instr[1]
            field_name = instr[2]
            t_val, val = self.stack.pop()

            self.load_class_if_needed(class_name)
            self.init_static_fields_if_needed(class_name)

            if field_name not in self.classes[class_name]["static_fields"]:
                print(f'Static field: {field_name} is not found, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            expected_type = self.classes[class_name]["static_fields"][field_name][0]
            if t_val != expected_type:
                print(f'Static field: {field_name} is {expected_type}, got {t_val}, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            self.classes[class_name]["static_fields"][field_name] = (t_val, val)

        elif op == 'LOAD_STATIC_FIELD':
            class_name = instr[1]
            field_name = instr[2]

            self.load_class_if_needed(class_name)
            self.init_static_fields_if_needed(class_name)

            if field_name not in self.classes[class_name]["static_fields"]:
                print(f'Static field: {field_name} is not found, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            t_val, val = self.classes[class_name]["static_fields"][field_name]
            if not val:
                print(f'Static field: {field_name} is uninitialized, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            self.stack.append((t_val, val))

        elif op == 'CALL_METHOD':
            method_name = instr[1]
            t_obj, obj = self.stack.pop()

            if t_obj != 'object':
                print(f'Expected object for CALL_METHOD, got {t_obj}, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            class_name = obj["_class"]
            methods = self.classes[class_name]["methods"]

            if method_name not in methods:
                print(f'Method: {method_name} is not found in class: {class_name}, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            label = methods[method_name]

            self.call_stack.append(self.ip)
            self.frame_stack.append({})
            self.this = (t_obj, obj)

            self.ip = self.labels[label] + 1

        elif op == 'CALL_STATIC_METHOD':
            class_name = instr[1]
            method_name = instr[2]

            self.load_class_if_needed(class_name)

            if method_name not in self.classes[class_name]["static_methods"]:
                print(f'Static method: {method_name} is not found, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            label = self.classes[class_name]["static_methods"][method_name]

            self.call_stack.append(self.ip)
            self.frame_stack.append({})

            self.ip = self.labels[label] + 1

        elif op == 'CALL_SUPER_METHOD':
            method_name = instr[1]
            t_obj, obj = self.stack.pop()

            if t_obj != 'object':
                print(f'Expected object for CALL_SUPER_METHOD, got {t_obj}, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            class_name = obj["_class"]
            super_class = self.classes[class_name]["super_class"]

            self.load_class_if_needed(super_class)

            if method_name not in self.classes[super_class]["methods"]:
                print(f'Method: {method_name} is not found in super class, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            label = self.classes[super_class]["methods"][method_name]

            self.call_stack.append(self.ip)
            self.frame_stack.append({})
            self.this = (t_obj, obj)

            self.ip = self.labels[label] + 1

        elif op == 'SLEEP':
            t_val, val = self.stack.pop()

            if t_val != 'int':
                print(f'Illegal Argument: {val} for SLEEP')
                exit(1)

            time.sleep(val / 1000)

        elif op == 'DUMP':
            print(f'[IP={self.ip}]: {raw_line.strip()}')

            print('[STACK]')
            pprint(self.stack, indent=2)

            print(f'[FRAME_STACK]')
            for frame in self.frame_stack:
                pprint(frame, indent=2)

            print('[TRY_STACK]')
            pprint(self.try_stack, indent=2)

            print('[CLASSES]')
            pprint(self.classes, indent=2)

        elif op == 'NEW_TUPLE':
            taking_size = int(instr[1])
            items = []

            for _ in range(taking_size):
                items.append(self.stack.pop())
            items.reverse()

            self.stack.append(('tuple', tuple(items)))

        elif op == 'TUPLE_GET':
            idx = int(instr[1])
            dtype, val = self.stack.pop()

            if dtype != 'tuple':
                print(f'Expected tuple for TUPLE_GET, got {dtype}, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            if idx >= len(val) or idx < 0:
                print(f'Index out of range: {idx}, length is {len(val)}, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            self.stack.append(val[idx])

        elif op == 'UNPACK_TUPLE':
            t_items, items = self.stack.pop()

            if t_items != 'tuple':
                print(f'Cannot unpack not tuple: {t_items}, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            for t_val, val in reversed(items):
                self.stack.append((t_val, val))

        elif op == 'NEW_ARRAY':
            elem_type = instr[1]
            t_size, size = self.stack.pop()

            if t_size != 'int':
                print(f'Expected int for size in ARRAY_INIT, got {t_size}, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            self.stack.append(('array', [elem_type, [None] * size]))

        elif op == 'INIT_ARRAY':
            elem_type = instr[1]
            t_size, size = self.stack.pop()

            if t_size != 'int':
                print(f'Expected int for size in ARRAY_INIT, got {t_size}, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            values = []
            for v in instr[4:]:
                if len(values) <= size:
                    if elem_type == 'int':
                        values.append(int(v))
                    elif elem_type == 'float':
                        values.append(float(v))
                    elif elem_type == 'bool':
                        values.append(v.lower() == 'true')
                    else:
                        values.append(v.strip('"'))
                else:
                    print(f'Found more elements than expected: {len(values)}, at {self.path}:{self.ip}:\n    {raw_line}')
                    exit(1)

            self.stack.append(('array', [elem_type, values]))

        elif op == 'NEW_GENERIC_ARRAY':
            generic_name = instr[1]

            t_obj, obj = self.stack.pop()

            if t_obj != 'object':
                print(f'Expected LOAD_THIS before using ARRAY_NEW_GENERIC, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            t_size, size = self.stack.pop()

            if t_size != 'int':
                print(f'Expected int for size in ARRAY_INIT, got {t_size}, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            elem_type = obj["generics"][generic_name]

            self.stack.append(('array', [elem_type, [None] * size]))

        elif op == 'ARRAY_GET':
            dtype, arr = self.stack.pop()
            t_idx, idx = self.stack.pop()

            if t_idx != 'int':
                print(f'Index must be int, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            if dtype != 'array':
                print(f'Expected array for ARRAY_GET, got {dtype}, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            elem_type, data = arr
            if idx < 0 or idx >= len(data):
                print(f'Index out of range: {idx}, length of array: {len(data)}, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            self.stack.append((elem_type, data[idx]))

        elif op == 'ARRAY_SET':
            dtype, arr = self.stack.pop()
            t_val, val = self.stack.pop()
            t_idx, idx = self.stack.pop()

            if t_idx != 'int':
                print(f'Index must be int, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            if dtype != 'array':
                print(f'Expected array for ARRAY_SET, got {dtype}, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            elem_type, data = arr
            if t_val != elem_type:
                print(f'Type mismatch: expected {elem_type}, got {t_val}, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            if idx < 0 or idx >= len(data):
                print(f'Index out of range: {idx}, length of array: {len(data)}, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            data[idx] = val

        elif op == 'ARRAY_LEN':
            dtype, arr = self.stack.pop()

            if dtype != 'array':
                print(f'Expected array for ARRAY_LENGTH, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            elem_type, data = arr
            self.stack.append(('int', len(data)))

        elif op in ('EQ', 'NEQ', 'LT', 'GT', 'LTE', 'GTE'):
            t2, b = self.stack.pop()
            t1, a = self.stack.pop()

            if t1 != t2:
                print(f'Type mismatch in compare: {t1} vs {t2}, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            res = False

            if op == 'EQ':
                res = a == b
            elif op == 'NEQ':
                res = a != b
            elif op == 'LT':
                res = a < b
            elif op == 'GT':
                res = a > b
            elif op == 'LTE':
                res = a <= b
            elif op == 'GTE':
                res = a >= b

            self.stack.append(('bool', res))

        elif op == 'AND':
            t2, b = self.stack.pop()
            t1, a = self.stack.pop()

            if t1 != 'bool' or t2 != 'bool':
                print(f"Type Error: AND only supports bool, got {t1} and {t2}, at {self.path}:{self.ip}:\n    {raw_line}")
                exit(1)

            self.stack.append(('bool', a and b))

        elif op == 'OR':
            t2, b = self.stack.pop()
            t1, a = self.stack.pop()

            if t1 != 'bool' or t2 != 'bool':
                print(f"Type Error: OR only supports bool, got {t1} and {t2}, at {self.path}:{self.ip}:\n    {raw_line}")
                exit(1)

            self.stack.append(('bool', a or b))

        elif op == 'NOT':
            t, val = self.stack.pop()

            if t != 'bool':
                print(f"Type Error: NOT only supports bool, got {t}, at {self.path}:{self.ip}:\n    {raw_line}")
                exit(1)

            self.stack.append(('bool', not val))

        elif op == 'TYPE_OF':
            target_type = instr[1]

            t_val, val = self.stack.pop()

            if t_val in ('int', 'float', 'bool', 'str', 'lambda'):
                self.stack.append(('bool', True if t_val == target_type else False))
            else:
                self.stack.append(('bool', False))

        elif op == 'INSTANCE_OF':
            target_class = instr[1]
            t_obj, obj = self.stack.pop()

            if t_obj != 'object':
                print(f'Expected object for INSTANCE_OF, got {t_obj}, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            self.load_class_if_needed(target_class)

            if not obj:
                self.stack.append(('bool', False))
                return

            obj_class = obj["_class"]

            if obj_class == target_class:
                self.stack.append(('bool', True))
                return

            current = obj_class
            while current:
                if current == target_class:
                    self.stack.append(('bool', True))
                    return
                if target_class in self.classes[current]["interfaces"]:
                    self.stack.append(('bool', True))
                    return
                current = self.classes[current]["super_class"]

            self.stack.append(('bool', False))

        elif op == 'JUMP':
            label = instr[1]

            if label not in self.labels:
                print(f'Cannot find label: {label}, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            self.ip = self.labels[label] + 1

        elif op == 'JUMP_IF_FALSE':
            label = instr[1]

            if label not in self.labels:
                print(f'Cannot find label: {label}, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            t, cond = self.stack.pop()
            if t != 'bool':
                print(f'Expected bool for JUMP_IF_FALSE, got: {t}, at {self.path}:{self.ip}:\n    {raw_line}')
                exit(1)

            if not cond:
                self.ip = self.labels[label] + 1

        elif op == 'LABEL':
            return

        elif op == 'HALT':
            value = int(instr[1]) if len(instr) > 1 else 0

            print('[STACK]')
            if self.stack:
                pprint(self.stack, indent=2)
            else:
                print('Stack is empty')

            print('[FRAMES]')
            if self.frame_stack:
                for frame in self.frame_stack:
                    pprint(frame, indent=2)
            else:
                print('Frames are empty')

            sys.exit(value)

        else:
            print(f'Not a statement: {raw_line}, at {self.path}:{self.ip}')
            exit(1)

def repl():
    print('Repl in development. Sorry for that')
    pass

def read(path):
    with open(path, 'r', encoding='utf-8') as f:
        return f.readlines()

def main():
    if len(sys.argv) < 2:
        print("Usage: python lvm.py <file.lbc>")
        return

    if sys.argv[1] == '--repl':
        repl()
        return

    lbc_path = sys.argv[1]

    if not lbc_path.endswith('.lbc'):
        print("Not .lbc file")
        return

    file = read(lbc_path)

    lvm = LVM(file, lbc_path)
    lvm.run()

if __name__ == "__main__":
    main()