import pycparser
from pycparser import c_ast
import click

class CompileError(Exception):
    def __init__(self, message, node):
        super().__init__(message)
        self.message = message
        self.node = node

class Compiler:
    def __init__(self, comment=False):
        self.code = ""
        self.funcs = []
        self.vars = [{}]
        self.sp_offset = 0

        self.comment = comment

        self._unique_id = 0

    @staticmethod
    def is_lvalue(node):
        return isinstance(node, (c_ast.ID, c_ast.ArrayRef))
    
    @staticmethod
    def type_size(type):
        if "*" in type:
            return 4
        else:
            return {
                "int": 4,
            }[type]

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
            if isinstance(node, c_ast.FuncDef):
                self.generate_function(node)
            else:
                raise CompileError(f"unknown node {node.__class__.__name__}", node)
    
    def generate_function(self, node):
        self.funcs.append(node.decl)
        self.sp_offset = 0
        self.code += f".export #func_{node.decl.name}\nfunc_{node.decl.name}:\npush $12\nmov $15 $12\n"
        self.generate_expression(node.body)
        if node.decl.name == "main":
            if self.comment:
                self.code += "; default main return\n"
            self.code += "mov $0 $1\nmov $12 $15\npop $12\nret\n"
    
    def generate_expression(self, node, register=1):
        if isinstance(node, c_ast.Compound):
            self.vars.append({})
            if node.block_items != None:
                for item in node.block_items:
                    self.generate_expression(item, register=register)
            self.vars.pop()
        elif isinstance(node, c_ast.Return):
            if self.comment:
                self.code += "; return\n"
            self.generate_expression(node.expr)
            self.code += "mov $12 $15\npop $12\nret\n"
        elif isinstance(node, c_ast.Decl):
            if isinstance(node.type, c_ast.TypeDecl):
                if self.comment:
                    self.code += f"; variable {node.name}\n"
                if node.init == None:
                    self.code += f"mov $0 ${register}\n"
                else:
                    self.generate_expression(node.init, register=register)
                self.code += f"push ${register}\n"
                self.sp_offset -= 4
                self.vars[-1][node.name] = (self.sp_offset, node.type.type.names[0])
            elif isinstance(node.type, c_ast.ArrayDecl):
                if self.comment:
                    self.code += f"; variable (array) {node.name}\n"
                size = int(node.type.dim.value)
                self.code += "push $0\n" * size
                self.sp_offset -= 4 * size
                self.vars[-1][node.name] = (self.sp_offset, node.type.type.type.names[0] + "*")
        elif isinstance(node, c_ast.Assignment):
            if self.comment:
                self.code += "; assignment\n"
            if node.op == "=":
                if not Compiler.is_lvalue(node.lvalue):
                    raise CompileError(f"expr not lvalue {node.lvalue}", node)
                self.generate_expression(node.lvalue, register=register)
                type = self.generate_expression(node.rvalue, register=register+2)
                self.code += f"std ${register+2} ${register+1}\nmov ${register+2} ${register}\n"
                return type
            else:
                raise CompileError(f"unknown assignment operator {node.op}", node)
        elif isinstance(node, c_ast.ArrayRef):
            if self.comment:
                self.code += "; array ref\n"
            if not Compiler.is_lvalue(node.name):
                raise CompileError(f"expr not lvalue {node.name}", node)
            type = self.generate_expression(node.name, register=register)
            self.generate_expression(node.subscript, register=register+2)
            self.code += f"add ${register+2} ${register+1}\nldd ${register+1} ${register}\n"
            if type[-1] != "*":
                raise CompileError(f"can't use array ref on non-pointer value", node)
            return type[:-1]
        elif isinstance(node, c_ast.For):
            if self.comment:
                self.code += "; for loop\n"
            self.vars.append({})
            id = self.unique_id
            for decl in node.init:
                self.generate_expression(decl, register=register)
            self.code += f"loop_{id}:\n"
            self.generate_expression(node.cond, register=register)
            self.code += f"bf #loop_{id}_end\n"
            self.generate_expression(node.stmt, register=register)
            self.generate_expression(node.next, register=register)
            self.code += f"b #loop_{id}\nloop_{id}_end:\n"
            self.vars.pop()
        elif isinstance(node, c_ast.BinaryOp):
            if self.comment:
                self.code += "; binary op\n; left expr\n"
            type = self.generate_expression(node.left, register=register)
            if self.comment:
                self.code += "; right expr\n"
            self.generate_expression(node.right, register=register+1)
            self.code += {
                "+": f"add ${register+1} ${register}\n",
                "-": f"sub ${register+1} ${register}\n",
                "<": f"clt ${register} ${register+1}\n",
                ">": f"cgt ${register} ${register+1}\n",
            }[node.op]
            return type
        elif isinstance(node, c_ast.UnaryOp):
            if self.comment:
                self.code += "; unary op\n"
            if node.op == "p++":
                if not Compiler.is_lvalue(node.expr):
                    raise CompileError(f"expr not lvalue {node.expr}", node)
                type = self.generate_expression(node.expr, register=register)
                self.code += f"add 1 ${register}\nstd ${register} ${register+1}\n"
                return type
            else:
                raise CompileError(f"unknown unary operator {node.op}", node)
        elif isinstance(node, c_ast.Constant):
            if self.comment:
                self.code += "; constant\n"
            if node.type == "int":
                if node.value == "0":
                    self.code += f"mov $0 ${register}\n"
                else:
                    self.code += f"mov {node.value} ${register}\n"
                return "int"
            else:
                raise CompileError(f"unknown constant type {node.type}", node)
        elif isinstance(node, c_ast.ID):
            if self.comment:
                self.code += f"; load variable {node.name}\n"
            (offset, type) = self.get_var(node.name)
            if offset != None:
                self.code += f"mov $12 ${register+1}\nsub {-offset} ${register+1}\nldd ${register+1} ${register}\n"
            else:
                raise CompileError(f"unknown variable {node.name}", node)
            return type
        elif isinstance(node, c_ast.EmptyStatement):
            pass
        else:
            raise CompileError(f"unknown node {node.__class__.__name__}", node)

@click.command()
@click.argument("files", type=click.Path(exists=True), required=True, nargs=-1)
@click.option("--comment", is_flag=True, default=False)
@click.option("--show-ast", is_flag=True, default=False)
def run(files, comment, show_ast):
    compiler = Compiler(comment=comment)
    for file in files:
        ast = pycparser.parse_file(file)
        if show_ast:
            ast.show(showcoord=True)
        try:
            compiler.compile(ast)
        except CompileError as e:
            click.echo(f"ERROR: {e.message} ({e.node.coord})", err=True)
            exit(1)
        with open(file + ".out", "w") as f:
            f.write(compiler.code)

if __name__ == "__main__":
    run(None, None, None)