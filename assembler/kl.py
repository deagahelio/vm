#!/usr/bin/env python3

import click

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

    def lit(self):
        if self.type == "list":
            return [node.lit() for node in self.value]
        else:
            return self.value

def parse(code, line=1, col=1):
    ast = []
    current = ""
    mode = "normal"
    paren_level = 0

    for i, char in enumerate(code):
        #print(f"{char}:{line}:{col} / {current}")

        col += 1
        if char == "\n":
            line += 1
            col = 1

        if mode == "list":
            if char == "(":
                paren_level += 1

            elif char == ")":
                paren_level -= 1

                if paren_level == 0:
                    ast.append(parse(current, line, col))
                    mode = "normal"
                    current = ""
                    continue

            current += char
        
        elif mode == "array":
            if char == "[":
                paren_level += 1

            elif char == "]":
                paren_level -= 1

                if paren_level == 0:
                    node = parse(current, line, col)
                    node.type = "array"
                    ast.append(node)
                    mode = "normal"
                    current = ""
                    continue

            current += char

        elif mode == "normal":
            if char in (" ", "\t", "\n", "\r", "(") or i == len(code) - 1:
                if i == len(code) - 1:
                    current += char

                if current not in ("", " ", "\t", "\n", "\r", "("):
                    node = Node(current, "word", line, col - len(current) - 1)

                    try:
                        node.value = int(node.value)
                        node.type = "int"
                    except ValueError:
                        pass

                    ast.append(node)

                current = ""

                if char == "(":
                    mode = "list"
                    paren_level += 1

                continue

            current += char
    
    return Node(ast, "list", 1, 1)

class CompileError(Exception):
    def __init__(self, message, node):
        super().__init__(message)
        self.message = message
        self.node = node

class Compiler:
    def __init__(self, comment=False):
        self.code = "" # Generated assembly code
        self.funcs = {} # Dict of function declaration nodes
        self.vars = [{}] # Stores variables and scopes (first scope is global)
        self.sp_offset = 0 # Keeps track of distance from base of stack frame to store local variables

        self.comment = comment # When set to true, will generate comments for the assembly code

        self._unique_id = 0 # Used for control flow labels
    
    def compile(self, ast):
        for node in ast:
            if node.type != "list":
                raise CompileError("top-level expression must be list", node)
            self.generate_expression(node)
    
    def generate_expression(self, node):
        pass

@click.command()
@click.argument("files", type=click.Path(exists=True), required=True, nargs=-1)
@click.option("--comment", is_flag=True, default=False)
def run(files, comment):
    compiler = Compiler(comment=comment)

    for file in files:
        with open(file, "r") as f:
            ast = parse(f.read())

        try:
            compiler.compile(ast)
        except CompileError as e:
            click.echo(f"ERROR: {e.message} ({e.node.line}:{e.node.col})", err=True)

            with open(file, "r") as f:
                click.echo(f.readlines()[e.node.line - 1][:-1], err=True)
                click.echo(" " * (e.node.col - 1) + "^", err=True)

            exit(1)

        with open(file + ".out", "w") as f:
            f.write(compiler.code)

if __name__ == "__main__":
    run(None, None)