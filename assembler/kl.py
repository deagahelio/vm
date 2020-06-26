#!/usr/bin/env python3

import click

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

TYPES = list(TYPE_SIZES.keys())

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

def parse(code, line=1, col=1):
    ast = []
    current = ""
    mode = "normal"
    comment = False
    paren_level = 0

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
                current = ""
                continue

        if mode == "list":
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
            if char in " \t\n\r([" or i == len(code) - 1:
                if i == len(code) - 1:
                    current += char

                if current not in " \t\n\r([" and current != "":
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

                continue

            if char == ";":
                mode = "comment"

            current += char
    
    return Node(ast, "list", first_line, first_col)

class CompileError(Exception):
    def __init__(self, message, node):
        super().__init__(message)
        self.message = message
        self.node = node

class Compiler:
    def __init__(self, path="<unknown>", comment=False):
        self.code = "" # Generated assembly code
        self.funcs = {} # Dict of function declaration nodes
        self.vars = [{}] # Stores variables and scopes (first scope is global)
        self.sp_offset = 0 # Keeps track of distance from base of stack frame to store local variables

        self.path = path # Path to source code file
        self.source_code = None # Contents of source code file to generate comments. Optional, must be set manually
        self.line = 0 # Last line of code that a comment was generated for
        self.comment = comment # When set to true, will generate comments for the assembly code

        self.last_unique_id = 0 # Used for control flow labels

    @property
    def unique_id(self):
        self.last_unique_id += 1
        return self.last_unique_id
    
    def compile(self, ast):
        self.code = ""
        self.line = 0
        if self.source_code:
            self.source_code = self.source_code.split("\n")

        for node in ast:
            if node.type != "list":
                raise CompileError("top-level expression must be list", node)

            self.generate_expression(node, True)
    
    def generate_expression(self, node, root=False, r=1):
        if self.comment and node.line > self.line:
            self.line = node.line
            self.code += f"; >>> {self.path}:{node.line}"
            if self.source_code:
                self.code += f" | {self.source_code[node.line - 1]}"
            self.code += "\n"

        if node.type == "int":
            self.code += f"mov {node.value} ${r}\n"
        
        elif node.type == "word":
            found = False

            for scope in reversed(self.vars):
                if node.value in scope.keys():
                    found = True
                    var = scope[node.value]

                    if var["global"]:
                        self.code += f"mov #{node.value} ${r+1}\nld{TYPE_DIRECTIVES[var['type']][0]} ${r+1} ${r}\n"
                    else:
                        self.code += f"mov $12 ${r+1}\nsub {-var['offset']} ${r+1}\nld{TYPE_DIRECTIVES[var['type']][0]} ${r+1} ${r}\n"
            
            if not found:
                raise CompileError("undefined variable", node)

        elif node[0].value == "fn":
            if len(node) < 4:
                raise CompileError("wrong number of arguments", node)
                
            if node[1].value not in TYPES:
                raise CompileError("first argument must be type", node)

            if node[2].type != "word":
                raise CompileError("invalid function name", node)

            if node[3].type != "list":
                raise CompileError("third argument must be parameter list", node)
            
            if not root:
                raise CompileError("function should have top-level declaration", node)

            self.sp_offset = 0
            self.vars.append({})

            self.code += f".export #{node[2]}\n{node[2]}:\npush $12\nmov $15 $12\n"
            for expr in node[4:]:
                self.generate_expression(expr, r=r)
            self.code += "mov $12 $15\npop $12\nret\n"

            self.vars.pop()

            self.funcs[node[2].value] = {
                "node": node,
                "type": node[1].value,
                "args": [], # TODO: function args
            }
        
        elif node[0].value == "static":
            if len(node) not in (4, 3):
                raise CompileError("wrong number of arguments", node)

            if node[1].value not in TYPES:
                raise CompileError("first argument must be type", node)

            if node[2].type != "word":
                raise CompileError("invalid variable name", node)

            if not root:
                raise CompileError("static variable should have top-level declaration", node)

            self.code += f".export #{node[2]}\n{node[2]}:\n.{TYPE_DIRECTIVES[node[1].value]} "
            if len(node) == 3:
                self.code += "0\n"
            else:
                # TODO: only works with ints
                self.code += f"{node[3]}\n"

            self.vars[0][node[2].value] = {
                "global": True, 
                "node": node,
                "type": node[1].value,
            }

        elif node[0].value == "local":
            if len(node) not in (4, 3):
                raise CompileError("wrong number of arguments", node)

            if node[1].value not in TYPES:
                raise CompileError("first argument must be type", node)

            if node[2].type != "word":
                raise CompileError("invalid variable name", node)

            if root:
                raise CompileError("local variable cannot have top-level declaration", node)

            if len(node) == 3:
                # TODO: only works with ints
                self.code += f"mov $0 ${r}\n"
            else:
                self.generate_expression(node[3], r=r)
            self.code += f"push ${r}\n"

            self.sp_offset -= 4
            self.vars[-1][node[2].value] = {
                "global": False, 
                "offset": self.sp_offset,
                "node": node,
                "type": node[1].value,
            }
        
        elif node[0].value == "return":
            if len(node) > 2:
                raise CompileError("wrong number of arguments", node)

            if len(node) == 2:
                self.generate_expression(node[1], r=r)
                if r != 1:
                    self.code += f"mov ${r} $1\n"
            self.code += "mov $12 $15\npop $12\nret\n"
        
        elif node[0].value in ("+", "-", "*", "/", "%", "<", ">"):
            if len(node) != 3:
                raise CompileError("wrong number of arguments", node)

            self.generate_expression(node[1], r=r)
            self.code += f"push ${r}\n"
            self.generate_expression(node[2], r=r)
            self.code += f"mov ${r} ${r+1}\npop ${r}\n"

            self.code += {
                "+": f"add ${r+1} ${r}\n",
                "-": f"sub ${r+1} ${r}\n",
                "*": f"mul ${r+1} ${r}\nmov $13 ${r}\n",
                "/": f"div ${r+1} ${r}\nmov $14 ${r}\n",
                "%": f"div ${r+1} ${r}\nmov $13 ${r}\n",
                "<": f"clt ${r} ${r+1}\nmov $0 ${r}\nbf #__clt{self.unique_id}\nmov 1 ${r}\n__clt{self.last_unique_id}:\n",
                ">": f"cgt ${r} ${r+1}\nmov $0 ${r}\nbf #__cgt{self.unique_id}\nmov 1 ${r}\n__cgt{self.last_unique_id}:\n",
            }[node[0].value]

        else:
            raise CompileError("unknown expression type", node)

@click.command()
@click.argument("files", type=click.Path(exists=True), required=True, nargs=-1)
@click.option("--comment", is_flag=True, default=False)
def run(files, comment):
    compiler = Compiler(comment=comment)

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
    run(None, None)