#!/usr/bin/env python3

import pycparser
from pycparser import c_ast
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

class CompileError(Exception):
    def __init__(self, message, node):
        super().__init__(message)
        self.message = message
        self.node = node

class Compiler:
    def __init__(self, comment=False):
        self.code = "" # Generated assembly code
        self.funcs = [] # List of function declaration nodes
        self.vars = [{}] # Stores variables and scopes (first scope is global)
        self.sp_offset = 0 # Keeps track of distance from base of stack frame to store local variables

        self.comment = comment # When set to true, will generate comments for the assembly code

        self._unique_id = 0 # Used for control flow labels

    @staticmethod
    def is_lvalue(node):
        return isinstance(node, (c_ast.ID, c_ast.ArrayRef)) or (isinstance(node, c_ast.UnaryOp) and node.op == "*")
    
    @staticmethod
    def type_size(node):
        if isinstance(node, c_ast.TypeDecl):
            if isinstance(node.type, c_ast.IdentifierType):
                if len(node.type.names) == 1:
                    return {
                        "char": 1,
                        "short": 2,
                        "int": 4,
                    }[node.type.names[0]]
                else:
                    raise CompileError(f"too many type names", node.type)
            else:
                raise CompileError(f"invalid type node `{node.type.__class__.__name__}`", node.type)
        elif isinstance(node, c_ast.PtrDecl):
            return 4
        elif isinstance(node, c_ast.ArrayDecl):
            if node.dim_quals != []:
                raise CompileError(f"invalid qualifier", node)
            # Assumes array size is constant TODO
            # `dim` should probably be replaced directly in AST node
            return int(node.dim.value) * Compiler.type_size(node.type)
        else:
            raise CompileError(f"invalid type node `{node.__class__.__name__}`", node)

    @staticmethod
    def type_base(node):
        while not isinstance(node, c_ast.TypeDecl):
            node = node.type
        return node

    @staticmethod
    def size_directive(node):
        if isinstance(node, c_ast.TypeDecl):
            size = Compiler.type_size(node)
        elif isinstance(node, (c_ast.ArrayDecl, c_ast.PtrDecl)):
            size = 4
        elif isinstance(node, c_ast.Decl):
            return Compiler.size_directive(node.type)
        else:
            raise CompileError(f"invalid type node `{node.__class__.__name__}`", node)
        return {
            1: "byte",
            2: "word",
            4: "dword"
        }[size]
    
    @staticmethod
    def type_string(node):
        # TODO remove this function?
        if isinstance(node, c_ast.TypeDecl):
            if isinstance(node.type, c_ast.IdentifierType):
                if len(node.type.names) == 1:
                    return node.type.names[0]
                else:
                    raise CompileError(f"too many type names", node.type)
            else:
                raise CompileError(f"invalid type node `{node.type.__class__.__name__}`", node.type)
        elif isinstance(node, c_ast.ArrayDecl):
            if node.dim_quals != []:
                raise CompileError(f"invalid qualifier", node)
            return Compiler.type_string(node.type) + "*"
        elif isinstance(node, c_ast.PtrDecl):
            if node.quals != []:
                raise CompileError(f"invalid qualifier", node)
            return Compiler.type_string(node.type) + "*"
        else:
            raise CompileError(f"invalid type node `{node.__class__.__name__}`", node)
    
    @staticmethod
    def make_array(node, init):
        # Makes a python list from an ArrayDecl and InitList node
        if isinstance(node.dim, c_ast.Constant):
            size = int(node.dim.value)
        else:
            raise CompileError(f"invalid array dim type `{node.dim.__class__.__name__}`", node)

        final = not isinstance(node.type, c_ast.ArrayDecl)
        array = []

        if init != None:
            for expr in init.exprs[:size]:
                if isinstance(expr, c_ast.InitList):
                    if final:
                        raise CompileError(f"init list nested too deep", node)
                    else:
                        array.append(Compiler.make_array(node.type, expr))
                else:
                    if final:
                        array.append(expr)
                    else:
                        raise CompileError(f"expected init list", node)
        
        return array + [None] * (size - len(array))

    @property
    def unique_id(self):
        self._unique_id += 1
        return self._unique_id
    
    def get_var(self, var):
        for scope in self.vars[::-1]:
            if var in scope:
                return scope[var]
        return None

    def compile(self, ast):
        self.code = ""
        for node in ast.ext:
            self.generate_decl(node)
    
    def generate_decl(self, node):
        if isinstance(node, c_ast.FuncDef):
            self.funcs.append(node.decl)
            self.sp_offset = 0

            # TODO: add support for static/inline functions (and variables)
            # $12 is used to store base pointer of function stack frame
            # push $12 - store old value of $12 in case the caller is using it
            # mov $15 $12 - move stack pointer to $12, stack frame starts here
            # From now on, the stack will be used to store local variables and temporary values
            self.code += f".export #{node.decl.name}\n{node.decl.name}:\npush $12\nmov $15 $12\n"
            self.generate_expression(node.body)
            if self.comment:
                self.code += "; default return\n"
            if node.decl.name == "main":
                # If no return value is specified for the main function, it should return 0
                self.code += "mov $0 $1\n"
            # mov $12 $15 - set stack pointer back to the start of stack frame
            # pop $12 - pop the old $12 value that was left in the stack
            self.code += "mov $12 $15\npop $12\nret\n"
        elif isinstance(node, c_ast.Decl):
            self.code += f".export #{node.name}\n{node.name}:\n"
            if isinstance(node.type, c_ast.TypeDecl):
                if node.init != None:
                    if isinstance(node.init, c_ast.Constant):
                        value = node.init.value
                    else:
                        raise CompileError(f"invalid global initializer type `{node.init.__class__.__name__}`", node)

                self.code += f".{self.size_directive(node.type)} {value}\n"
                self.vars[0][node.name] = (-1, node.type) # -1 since it's a global variable and not stored in the stack
            elif isinstance(node.type, c_ast.ArrayDecl):
                directive = self.size_directive(self.type_base(node.type))
                array = self.make_array(node.type, node.init)

                for item in flatten(array):
                    if item == None:
                        self.code += f".{directive} 0\n"
                    elif isinstance(item, c_ast.Constant):
                        self.code += f".{directive} {item.value}\n"
                    else:
                        raise CompileError(f"invalid global initializer type `{node.init.__class__.__name__}`", node)

                self.vars[0][node.name] = (-1, node.type)
            else:
                raise CompileError(f"invalid global declaration type `{node.type.__class__.__name__}`", node)
        else:
            raise CompileError(f"invalid external declaration `{node.__class__.__name__}`", node)
    
    def generate_expression(self, node, register=1):
        # This function is recursive
        # The generated code will store the value of the expression in $register
        # and the address of the value (if there is one) in $register+1
        if isinstance(node, c_ast.Compound):
            # Push a new local variable scope (stack frame doesn't change)
            self.vars.append({})

            if node.block_items != None:
                for item in node.block_items:
                    self.generate_expression(item, register=register)

            self.vars.pop()
        elif isinstance(node, c_ast.Return):
            if self.comment:
                self.code += "; return\n"

            # Return value is stored in $1
            self.generate_expression(node.expr)
            self.code += "mov $12 $15\npop $12\nret\n"
        elif isinstance(node, c_ast.Decl):
            if isinstance(node.type, (c_ast.TypeDecl, c_ast.PtrDecl)):
                if self.comment:
                    self.code += f"; variable {node.name}\n"

                if node.init == None:
                    # Moving $0 uses less space than using an immediate value
                    self.code += f"mov $0 ${register}\n"
                else:
                    self.generate_expression(node.init, register=register)

                self.code += f"push ${register}\n"
                self.sp_offset -= 4
                self.vars[-1][node.name] = (self.sp_offset, node.type)
            elif isinstance(node.type, c_ast.ArrayDecl):
                if self.comment:
                    self.code += f"; variable (array) {node.name}\n"

                array = list(flatten(self.make_array(node.type, node.init)))
                
                # Encode array into a sequence of dwords to be pushed
                size = self.type_size(self.type_base(node.type.type)) * 8
                in_dword = 32 // size
                array += [0] * (len(array) % in_dword)
                pushed = 0

                for chunk in chunks(array[::-1], in_dword):
                    dword = 0
                    for shift in reversed(range(in_dword)):
                        # Assumes array dim is constant TODO
                        if chunk[shift] != None:
                            value = int(chunk[shift].value)
                        else:
                            value = 0
                        dword |= value << (shift * size)
                    if dword == 0:
                        self.code += f"push $0\n"
                    else:
                        self.code += f"push {dword}\n"
                    pushed += 1

                self.sp_offset -= 4 * pushed
                self.vars[-1][node.name] = (self.sp_offset, node.type)
            else:
                raise CompileError(f"unknown declaration type `{node.type.__class__.__name__}`", node)
        elif isinstance(node, c_ast.Assignment):
            if self.comment:
                self.code += "; assignment\n"

            if node.op == "=":
                if not self.is_lvalue(node.lvalue):
                    raise CompileError(f"invalid lvalue", node)

                type = self.generate_expression(node.lvalue, register=register)
                self.code += f"push ${register+1}\n"
                self.generate_expression(node.rvalue, register=register)
                self.code += f"pop ${register+1}\nst{self.size_directive(type)[0]} ${register} ${register+1}\n"
                return type
            else:
                raise CompileError(f"invalid assignment operator `{node.op}`", node)
        elif isinstance(node, c_ast.ArrayRef):
            if self.comment:
                self.code += "; array ref\n"

            if not self.is_lvalue(node.name):
                raise CompileError(f"invalid lvalue", node)

            type = self.generate_expression(node.name, register=register)
            if not isinstance(type, (c_ast.ArrayDecl, c_ast.PtrDecl)):
                raise CompileError(f"can't index non-pointer value", node)

            # We want to work with the value of the pointer variable, not a pointer to it
            if isinstance(type, c_ast.PtrDecl):
                self.code += f"push ${register}\n"
            else:
                self.code += f"push ${register+1}\n"
            self.generate_expression(node.subscript, register=register)
            self.code += f"mul {self.type_size(type.type)} ${register}\nmov $13 ${register}\npop ${register+1}\nadd ${register} ${register+1}\n"
            # Workaround for multidimensional arrays: pointers to sub-arrays aren't actually created in memory
            if isinstance(type.type, c_ast.ArrayDecl):
                self.code += f"mov ${register+1} ${register}\n"
            else:
                self.code += f"ld{self.size_directive(type.type)[0]} ${register+1} ${register}\n"
            return type.type
        elif isinstance(node, c_ast.For):
            if self.comment:
                self.code += "; for loop\n"

            # Generating a compound expression already makes a new scope so this one
            # only contains variables defined in the initial loop statement
            self.vars.append({})
            id = self.unique_id
            for decl in node.init:
                self.generate_expression(decl, register=register)
            self.code += f"_for{id}:\n"
            self.generate_expression(node.cond, register=register)
            self.code += f"bf #_for{id}end\n"
            self.generate_expression(node.stmt, register=register)
            self.generate_expression(node.next, register=register)
            self.code += f"b #_for{id}\n_for{id}end:\n"
            self.vars.pop()
        elif isinstance(node, c_ast.BinaryOp):
            if self.comment:
                self.code += "; binary op\n; right expr\n"

            self.generate_expression(node.right, register=register)
            self.code += f"push ${register}\n"

            if self.comment:
                self.code += "; left expr\n"
            type = self.generate_expression(node.left, register=register)

            self.code += f"pop ${register+1}\n"
            self.code += {
                "+": f"add ${register+1} ${register}\n",
                "-": f"sub ${register+1} ${register}\n",
                "*": f"mul ${register+1} ${register}\nmov $13 ${register}\n",
                "/": f"div ${register+1} ${register}\nmov $14 ${register}\n",
                "%": f"div ${register+1} ${register}\nmov $13 ${register}\n",
                "<": f"clt ${register} ${register+1}\n",
                ">": f"cgt ${register} ${register+1}\n",
            }[node.op]
            return type
        elif isinstance(node, c_ast.UnaryOp):
            if self.comment:
                self.code += "; unary op\n"

            if node.op == "p++":
                if not self.is_lvalue(node.expr):
                    raise CompileError(f"invalid lvalue", node)

                type = self.generate_expression(node.expr, register=register)
                self.code += f"add 1 ${register}\nst{self.size_directive(type)[0]} ${register} ${register+1}\n"
                return type
            elif node.op == "*":
                type = self.generate_expression(node.expr, register=register)
                if not isinstance(type, (c_ast.ArrayDecl, c_ast.PtrDecl)):
                    raise CompileError("can't dereference non-pointer value", node)
                self.code += f"mov ${register} ${register+1}\nld{self.size_directive(type.type)[0]} ${register+1} ${register}\n"
                return type.type
            else:
                raise CompileError(f"invalid unary operator `{node.op}`", node)
        elif isinstance(node, c_ast.Cast):
            if self.comment:
                self.code += "; cast\n"

            self.generate_expression(node.expr, register=register)
            type = node.to_type.type
            return type
        elif isinstance(node, c_ast.Constant):
            if self.comment:
                self.code += "; constant\n"

            if node.type in ("char", "short", "int"):
                if node.value == "0":
                    # Moving $0 uses less space than using an immediate value
                    self.code += f"mov $0 ${register}\n"
                else:
                    self.code += f"mov {node.value} ${register}\n"
                return node.type
            else:
                raise CompileError(f"invalid constant type `{node.type}`", node)
        elif isinstance(node, c_ast.ID):
            if self.comment:
                self.code += f"; load variable {node.name}\n"

            var = self.get_var(node.name)
            if var != None:
                (offset, type) = var
                if offset == -1: # Variable is global
                    self.code += f"mov #{node.name} ${register+1}\nld{self.size_directive(type)[0]} ${register+1} ${register}\n"
                else:
                    self.code += f"mov $12 ${register+1}\nsub {-offset} ${register+1}\nld{self.size_directive(type)[0]} ${register+1} ${register}\n"
            else:
                raise CompileError(f"undefined variable `{node.name}`", node)

            return type
        elif isinstance(node, c_ast.EmptyStatement):
            pass
        else:
            raise CompileError(f"invalid expression or statement `{node.__class__.__name__}`", node)

@click.command()
@click.argument("files", type=click.Path(exists=True), required=True, nargs=-1)
@click.option("--comment", is_flag=True, default=False)
@click.option("--show-ast", is_flag=True, default=False)
def run(files, comment, show_ast):
    compiler = Compiler(comment=comment)
    for file in files:
        ast = pycparser.parse_file(file, use_cpp=True)
        if show_ast:
            ast.show(showcoord=True)
        try:
            compiler.compile(ast)
        except CompileError as e:
            click.echo(f"ERROR: {e.message} ({e.node.coord})", err=True)
            with open(e.node.coord.file, "r") as f:
                click.echo(f.readlines()[e.node.coord.line - 1][:-1], err=True)
                click.echo(" " * (e.node.coord.column - 1) + "^", err=True)
            exit(1)
        with open(file + ".out", "w") as f:
            f.write(compiler.code)

if __name__ == "__main__":
    run(None, None, None)