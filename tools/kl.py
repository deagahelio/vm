#!/usr/bin/env python3

import click

UNSIGNED_INT_TYPES = [
    "uint8",
    "uint16",
    "uint32",
]

SIGNED_INT_TYPES = [
    "int8",
    "int16",
    "int32",
]

INT_TYPES = UNSIGNED_INT_TYPES + SIGNED_INT_TYPES

TYPE_SIZES = {
    "uint8": 1,
    "uint16": 2,
    "uint32": 4,
    "int8": 1,
    "int16": 2,
    "int32": 4,
}

SIZE_DIRECTIVES = {
    1: "byte",
    2: "word",
    4: "dword",
}

TYPE_DIRECTIVES = {t: SIZE_DIRECTIVES[TYPE_SIZES[t]] for t in TYPE_SIZES.keys()}

TYPES = list(TYPE_SIZES.keys()) + ["void"]

def flatten(l):
    for e in l:
        if isinstance(e, list):
            for e in flatten(e):
                yield e
        else:
            yield e

def chunks(l, n):
    for i in range(0, len(l), n):
        yield l[i:i + n]

class Node():
    def __init__(self, value, type, line, col):
        self.value = value
        self.type = type
        self.line = line
        self.col = col
    
    def __repr__(self):
        return f"Node({repr(self.value)}, {repr(self.type)}, {repr(self.line)}, {repr(self.col)})"

    def __str__(self):
        return str(self.lit())
    
    def __getitem__(self, index):
        return self.value[index]
    
    def __len__(self):
        return len(self.value)

    def lit(self):
        if self.type == "list":
            return [node.lit() for node in self.value]
        else:
            return self.value
    
    @property
    def id(self):
        return f"{self.line}_{self.col}"

def parse(code, line=1, col=1):
    ast = []
    current = ""
    mode = "normal"
    paren_level = 0
    comment = False
    string = False
    char_ = False
    escape = False

    first_line, first_col = old_line, old_col = line, col

    for i, char in enumerate(code):
        #print(f"{char}:{line}:{col} / {current}")

        col += 1
        if char == "\n":
            line += 1
            col = 1

        if comment:
            if char == "\n":
                comment = False
            continue

        elif string:
            if escape:
                escape = False
            elif char == "\\":
                escape = True
                continue
            elif char == "\"":
                ast.append(Node(
                    [Node(ord(char), "int", line, col - len(current) - 1) for char in current],
                    "list",
                    line, col - len(current) - 1
                ))
                string = False
                current = ""
                continue

            current += char
            continue

        elif char_:
            ast.append(Node(ord(char), "int", line, col - 1))
            char_ = False
            continue

        elif mode == "list":
            if char == "(":
                paren_level += 1

            elif char == ")":
                paren_level -= 1

                if paren_level == 0:
                    ast.append(parse(current, old_line, old_col))
                    mode = "normal"
                    current = ""
                    continue

            current += char
        
        elif mode == "normal":
            if char == "\"":
                string = True
                current = ""
                continue
            
            if char == "'":
                char_ = True
                current = ""
                continue

            elif char in " \t\n\r([;" or i == len(code) - 1:
                if i == len(code) - 1:
                    current += char

                if current not in " \t\n\r([;" and current != "":
                    node = Node(current, "word", line, col - len(current) - 1)

                    if node.value.startswith("0x"):
                        node.value = int(node.value, 16)
                        node.type = "int"
                    elif node.value.startswith("0b"):
                        node.value = int(node.value, 2)
                        node.type = "int"
                    elif node.value.startswith("0o"):
                        node.value = int(node.value, 8)
                        node.type = "int"
                    elif node.value[0] in "0123456789":
                        node.value = int(node.value)
                        node.type = "int"

                    ast.append(node)

                current = ""

                if char == "(":
                    mode = "list"
                    paren_level += 1
                    old_line, old_col = line, col
                
                if char == ";":
                    comment = True

                continue

            current += char
    
    return Node(ast, "list", first_line, first_col)

class CompileError(Exception):
    def __init__(self, message, node):
        super().__init__(message)
        self.message = message
        self.node = node

class Compiler:
    def __init__(self, path="<unknown>", comment=False, type_checking="loose", definitions_mode=False):
        self.code = "" # Generated assembly code
        self.funcs = {} # Dict of function declaration nodes
        self.vars = [{}] # Stores variables and scopes (first scope is global)
        self.sp_offset = 0 # Keeps track of distance from base of stack frame to store local variables

        self.path = path # Path to source code file
        self.source_code = None # Contents of source code file to generate comments. Optional, must be set manually
        self.line = 0 # Last line of code that a comment was generated for
        self.comment = comment # When set to true, will generate comments for the assembly code

        self.type_checking = type_checking # Type checking mode. [strict/loose/off]
        self.definitions_mode = definitions_mode # When in definitions mode, compiler doesn't generate any code
    
    def warning(self, message, node):
        click.echo(f"WARNING: {message} ({self.path}:{node.line}:{node.col})", err=True)

        click.echo(self.source_code[node.line - 1], err=True)
        click.echo(" " * (node.col - 1) + "^", err=True)

    def merge_types(self, l, r, node):
        if self.type_checking == "off":
            return

        if l == r:
            return l
        
        # TODO: this breaks for some cases with signed ints
        elif (l == "int" and r in INT_TYPES):
            return r
        
        elif (l in INT_TYPES and r == "int"):
            return l
        
        elif (self.type_checking == "loose" and l in UNSIGNED_INT_TYPES and r in UNSIGNED_INT_TYPES):
            return max(l, r, key=UNSIGNED_INT_TYPES.index)

        elif (self.type_checking == "loose" and l in SIGNED_INT_TYPES and r in SIGNED_INT_TYPES):
            return max(l, r, key=SIGNED_INT_TYPES.index)

        else:
            raise CompileError(f"cannot merge types '{l}' and '{r}'", node)

    def compile(self, ast):
        if not self.definitions_mode:
            self.definitions_mode = True
            self.compile(ast)
            self.definitions_mode = False

        else:
            self.code = ""
            self.funcs = {}
            self.vars = [{}]
            self.sp_offset = 0
            self.line = 0

            if self.source_code:
                self.source_code = self.source_code.split("\n")

        for node in ast:
            self.generate_expression(node, root=True)
    
    def generate_expression(self, node, root=False, statement=False, r=1):
        top_level = ["fn", "static", "array", "import"]

        if root:
            if node.type != "list":
                raise CompileError("top-level expression must be list", node)
        
            if node[0].value not in top_level + ["asm"]:
                raise CompileError("invalid top-level expression", node)

        elif node.type == "list" and node[0].value in top_level:
            raise CompileError("expression must be top-level", node)

        if self.comment and not self.definitions_mode and node.line > self.line:
            self.line = node.line

            self.code += f"; >>> {self.path}:{node.line}"
            if self.source_code:
                self.code += f" | {self.source_code[node.line - 1]}"

            self.code += "\n"

        if node.type == "int":
            self.code += f"mov {node.value} ${r}\n"
            
            return "int"
        
        elif node.type == "word":
            for scope in reversed(self.vars):
                if node.value in scope.keys():
                    var = scope[node.value]

                    if var["global"]:
                        self.code += f"mov #{node.value} ${r+1}\nld{TYPE_DIRECTIVES[var['type']][0]} ${r+1} ${r}\n"
                    else:
                        self.code += f"mov $12 ${r+1}\n"
                        if var["offset"] < 0:
                            self.code += f"sub {-var['offset']} ${r+1}\n"
                        else:
                            self.code += f"add {var['offset']} ${r+1}\n"
                        self.code += f"ld{TYPE_DIRECTIVES[var['type']][0]} ${r+1} ${r}\n"
                    
                    return var["type"]
            
            raise CompileError("undefined variable", node)

        elif node[0].value == "import":
            if len(node) != 2:
                raise CompileError("wrong number of arguments", node)
                
            if node[1].type != "list":
                raise CompileError("file name must be string or list of bytes", node)

            if self.definitions_mode:
                return

            path = "".join([chr(val.value) for val in node[1].value])
            with open(path, "r") as f:
                code = f.read()
                compiler = Compiler(path=path, definitions_mode=True)
                compiler.source_code = code
                compiler.compile(parse(code))

                self.funcs = {**self.funcs, **compiler.funcs}
                self.vars[0] = {**self.vars[0], **compiler.vars[0]}

                for symbol in {**self.funcs, **self.vars[0]}.keys():
                    self.code += f".import #{symbol}\n"

        elif node[0].value == "fn":
            if len(node) < 4:
                raise CompileError("wrong number of arguments", node)
                
            if node[1].value not in TYPES:
                raise CompileError("first argument must be type", node)

            if node[2].type != "word":
                raise CompileError("invalid function name", node)

            if node[3].type != "list":
                raise CompileError("third argument must be parameter list", node)
            
            if self.definitions_mode:
                if node[2].value in self.funcs:
                    raise CompileError("cannot declare function twice", node)

                arg_offset = 8
                for arg in node[3]:
                    if arg.type != "list":
                        raise CompileError("invalid parameter definition", node)

                    if len(arg) != 2:
                        raise CompileError("wrong number of arguments", node)

                    if arg[0].value not in TYPES:
                        raise CompileError("first argument must be type", node)

                    if arg[1].type != "word":
                        raise CompileError("invalid parameter name", node)

                    self.vars[-1][arg[1].value] = {
                        "global": False, 
                        "offset": arg_offset,
                        "node": arg,
                        "type": arg[0].value,
                        "length": 1,
                    }
                    arg_offset += 4

                self.funcs[node[2].value] = {
                    "node": node,
                    "type": node[1].value,
                    "args": [arg[0].value for arg in node[3]],
                }

            else:
                self.sp_offset = 0
                self.vars.append({})

                self.code += f".export #{node[2]}\n#{node[2]}:\npush $12\nmov $15 $12\n"
                for expr in node[4:]:
                    self.generate_expression(expr, statement=True, r=r)
                self.code += "mov $12 $15\npop $12\nret\n"

                self.vars.pop()
        
        elif node[0].value == "while":
            if len(node) == 1:
                raise CompileError("wrong number of arguments", node)

            if not statement:
                raise CompileError("while loop cannot be used in expression", node)

            self.code += f"#__while_{node.id}:\n"
            self.generate_expression(node[1], r=r)
            self.code += f"bf #__while_{node.id}_end\n"
            for expr in node[2:]:
                self.generate_expression(expr, statement=True, r=r)
            self.code += f"b #__while_{node.id}\n#__while_{node.id}_end:\n"
        
        elif node[0].value == "cond":
            if len(node) == 1:
                raise CompileError("wrong number of arguments", node)

            if not statement:
                raise CompileError("cond statement cannot be used in expression", node)

            for i, block in enumerate(node[1:]):
                if len(block) == 0:
                    raise CompileError("cond branch cannot be empty", node)

                self.generate_expression(block[0], r=r)
                self.code += f"bf #__cond_{node.id}_{i}\n"
                for expr in block[1:]:
                    self.generate_expression(expr, statement=True, r=r)
                self.code += f"#__cond_{node.id}_{i}:\n"
            
        elif node[0].value == "static":
            if len(node) not in (4, 3):
                raise CompileError("wrong number of arguments", node)

            if node[1].value not in TYPES:
                raise CompileError("first argument must be type", node)

            if node[2].type != "word":
                raise CompileError("invalid variable name", node)

            if len(node) == 4 and node[3].type not in ("int", "list"):
                raise CompileError("static variable must be integer or array of integers", node)

            if self.definitions_mode:
                if node[2].value in self.vars[0]:
                    raise CompileError("cannot declare variable twice", node)

                self.vars[0][node[2].value] = {
                    "global": True, 
                    "node": node,
                    "type": node[1].value,
                    "length": len(node[2]) if node[2].type == "list" else 1,
                }

            else:
                if len(node) == 3:
                    self.code += f".export #{node[2]}\n#{node[2]}:\n.{TYPE_DIRECTIVES[node[1].value]} 0\n"

                else:
                    if node[3].type == "int":
                        self.code += f".export #{node[2]}\n#{node[2]}:\n.{TYPE_DIRECTIVES[node[1].value]} "
                        if len(node) == 3:
                            self.code += "0\n"
                        else:
                            self.code += f"{node[3]}\n"

                    else:
                        self.code += f".export #{node[2]}\n#{node[2]}:\n" 

                        for expr in node[3]:
                            if expr.type != "int":
                                raise CompileError("array element must be integer literal", node)

                            self.code += f".{TYPE_DIRECTIVES[node[1].value]} {expr}\n"
        
        elif node[0].value == "local":
            if len(node) not in (4, 3):
                raise CompileError("wrong number of arguments", node)

            if node[1].value not in TYPES:
                raise CompileError("first argument must be type", node)

            if node[2].type != "word":
                raise CompileError("invalid variable name", node)

            if not statement:
                raise CompileError("local variable cannot be declared in expression", node)

            if node[2].value in self.vars[-1]:
                raise CompileError("cannot declare variable twice", node)

            if len(node) == 3:
                # TODO: only works with ints
                self.code += f"mov $0 ${r}\n"
            else:
                type = self.generate_expression(node[3], r=r)
                self.merge_types(node[1].value, type, node)
            self.code += f"push ${r}\n"

            self.sp_offset -= 4
            self.vars[-1][node[2].value] = {
                "global": False, 
                "offset": self.sp_offset,
                "node": node,
                "type": node[1].value,
                "length": 1,
            }
        
        elif node[0].value == "return":
            if len(node) > 2:
                raise CompileError("wrong number of arguments", node)

            if not statement:
                raise CompileError("return cannot be used in expression", node)

            if len(node) == 2:
                self.generate_expression(node[1], r=r)
                if r != 1:
                    self.code += f"mov ${r} $1\n"
            self.code += "mov $12 $15\npop $12\nret\n"
        
        elif node[0].value in ("+", "-", "*", "/", "%", "<", ">", ">=", "<=", "==", "!=", "&", "|", "<<", ">>"):
            if len(node) != 3:
                raise CompileError("wrong number of arguments", node)

            type_r = self.generate_expression(node[2], r=r)
            self.code += f"push ${r}\n"
            type_l = self.generate_expression(node[1], r=r)
            self.code += f"pop ${r+1}\n"

            self.code += {
                "+": f"add ${r+1} ${r}\n",
                "-": f"sub ${r+1} ${r}\n",
                "*": f"mul ${r+1} ${r}\nmov $13 ${r}\n",
                "/": f"div ${r+1} ${r}\nmov $14 ${r}\n",
                "%": f"div ${r+1} ${r}\nmov $13 ${r}\n",
                # TODO: clt and cgt don't work for signed ints
                "<": f"clt ${r} ${r+1}\n",
                ">": f"cgt ${r} ${r+1}\n",
                "<=": f"cltq ${r} ${r+1}\n",
                ">=": f"cgtq ${r} ${r+1}\n",
                "==": f"ceq ${r} ${r+1}\n",
                "!=": f"cnq ${r} ${r+1}\n",
                "&": f"and ${r+1} ${r}\n",
                "|": f"or ${r+1} ${r}\n",
                "<<": f"shl ${r+1} ${r}\n",
                ">>": f"shr ${r+1} ${r}\n",
            }[node[0].value]
            
            return self.merge_types(type_l, type_r, node)
        
        elif node[0].value == "set-var":
            if len(node) != 3:
                raise CompileError("wrong number of arguments", node)
            
            if node[1].type != "word":
                raise CompileError("first argument must be variable name", node)

            type_l = self.generate_expression(node[1], r=r)
            self.code += f"push ${r+1}\n"
            type_r = self.generate_expression(node[2], r=r)
            self.code += f"pop ${r+1}\nst{TYPE_DIRECTIVES[type_l][0]} ${r} ${r+1}\n"

            self.merge_types(type_l, type_r, node)

            return type_l
        
        elif node[0].value in ("set-8", "set-16", "set-32"):
            if len(node) != 3:
                raise CompileError("wrong number of arguments", node)

            size = {
                "set-8": "b",
                "set-16": "w",
                "set-32": "d",
            }[node[0].value]

            self.generate_expression(node[1], r=r)
            self.code += f"push ${r}\n"
            type = self.generate_expression(node[2], r=r)
            self.code += f"pop ${r+1}\nst{size} ${r} ${r+1}\n"

            return type
        
        elif node[0].value in ("get-8", "get-16", "get-32"):
            if len(node) != 2:
                raise CompileError("wrong number of arguments", node)

            size = {
                "get-8": "b",
                "get-16": "w",
                "get-32": "d",
            }[node[0].value]

            self.generate_expression(node[1], r=r)
            self.code += f"ld{size} ${r} ${r}\n"

            return {
                "b": "uint8",
                "w": "uint16",
                "d": "uint32",
            }[size]

        elif node[0].value == "cast":
            if len(node) != 3:
                raise CompileError("wrong number of arguments", node)

            if node[1].value not in TYPES:
                raise CompileError("first argument must be type", node)

            self.generate_expression(node[2], r=r)

            return node[1].value
        
        elif node[0].value == "addr":
            if len(node) != 2:
                raise CompileError("wrong number of arguments", node)

            self.generate_expression(node[1], r=r)
            self.code += f"mov ${r+1} ${r}\n"

            return "uint32"
        
        elif node[0].value == "bool":
            if len(node) > 2:
                raise CompileError("wrong number of arguments", node)
                
            if len(node) == 2:
                self.generate_expression(node[1], r=r)
            self.code += f"mov $0 ${r}\nbf #__bool_{node.id}_1\nmov 1 ${r}\n#__bool_{node.id}_1:\n"

            return "uint8"
        
        elif node[0].value == "elem-var":
            if len(node) != 3:
                raise CompileError("wrong number of arguments", node)

            if node[1].type != "word":
                raise CompileError("first argument must be variable name", node)

            type_l = self.generate_expression(node[1], r=r)
            self.code += f"push ${r+1}\n"
            type_r = self.generate_expression(node[2], r=r)
            self.code += f"pop ${r+1}\n"
            if TYPE_SIZES[type_l] != 1:
                self.code += f"mul {TYPE_SIZES[type_l]} ${r}\nadd $13 ${r+1}\n"
            else:
                self.code += f"add ${r} ${r+1}\n"
            self.code += f"ld{TYPE_DIRECTIVES[type_l][0]} ${r+1} ${r}\n"

            return type_l
        
        elif node[0].value == "len-var":
            if len(node) != 2:
                raise CompileError("wrong number of arguments", node)

            if node[1].type != "word":
                raise CompileError("first argument must be variable name", node)

            if node[1].value not in self.vars[0]:
                raise CompileError("undefined static variable", node)

            self.code += f"mov {self.vars[0][node[1].value]['length']} ${r}\n"

            return "uint32"

        elif node[0].value == "asm":
            if len(node) == 1:
                raise CompileError("wrong number of arguments", node)
            
            if not self.definitions_mode:
                for arg in node[1:]:
                    if arg.type != "list" or set([val.type for val in arg.value]) != set(["int"]):
                        raise CompileError("inline assembly must be string or list of bytes", arg)

                    self.code += "".join([chr(val.value) for val in arg.value]) + "\n"

        elif node[0].value == "data":
            if len(node) != 3:
                raise CompileError("wrong number of arguments", node)
            
            if node[1].value not in TYPES:
                raise CompileError("first argument must be type", node)
            
            if node[2].type == "int":
                self.code = f"#__data_{node.id}:\n.{TYPE_DIRECTIVES[node[1].value]} {node[2]}\n" + self.code
            elif node[2].type == "list" and set([val.type for val in node[2].value]) == set(["int"]):
                code = "\n".join([f".{TYPE_DIRECTIVES[node[1].value]} {i}" for i in node[2].value])
                self.code = f"#__data_{node.id}:\n{code}\n" + self.code
            else:
                raise CompileError("invalid data type", node)
            self.code += f"mov #__data_{node.id} ${r+1}\nld{TYPE_DIRECTIVES[node[1].value][0]} ${r+1} ${r}\n"

        elif node[0].value in self.funcs:
            func = self.funcs[node[0].value]

            if len(node) - 1 != len(func["args"]):
                raise CompileError("wrong number of arguments", node)

            for (arg, param) in zip(reversed(node[1:]), reversed(func["args"])):
                type = self.generate_expression(arg, r=r)
                self.merge_types(type, param, arg)
                self.code += f"push ${r}\n"
            self.code += f"call #{node[0]}\n"
            if r != 1:
                self.code += f"mov $1 ${r}\n"
            for _ in node[1:]:
                self.code += "pop $0\n"
            
            return func["type"]

        else:
            raise CompileError("undefined function", node)

@click.command()
@click.argument("files", type=click.Path(exists=True), required=True, nargs=-1)
@click.option("--comment", is_flag=True, default=False, help="Adds comment lines to the generated assembly code")
@click.option("--type-checking", default="loose", help="Type checking mode [strict/loose/off]")
def run(files, comment, type_checking):
    compiler = Compiler(comment=comment, type_checking=type_checking)

    for file in files:
        with open(file, "r") as f:
            code = f.read()
            ast = parse(code)

        compiler.path = file
        compiler.source_code = code

        try:
            compiler.compile(ast)
        except CompileError as e:
            click.echo(f"ERROR: {e.message} ({file}:{e.node.line}:{e.node.col})", err=True)

            with open(file, "r") as f:
                click.echo(f.readlines()[e.node.line - 1][:-1], err=True)
                click.echo(" " * (e.node.col - 1) + "^", err=True)

            exit(1)

        with open(file + ".out", "w") as f:
            f.write(compiler.code)

if __name__ == "__main__":
    run(None, None, None)