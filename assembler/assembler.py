import lark
import struct

INSTRUCTIONS = {
    "nop":   {"operands": "",   "opcode": b"\x00"},
    "add":   {"operands": "rr", "opcode": b"\x01"},
    "sub":   {"operands": "rr", "opcode": b"\x02"},
    "mul":   {"operands": "rr", "opcode": b"\x03"},
    "div":   {"operands": "rr", "opcode": b"\x04"},
    "and":   {"operands": "rr", "opcode": b"\x05"},
    "or":    {"operands": "rr", "opcode": b"\x06"},
    "xor":   {"operands": "rr", "opcode": b"\x07"},
    "shl":   {"operands": "rr", "opcode": b"\x08"},
    "shr":   {"operands": "rr", "opcode": b"\x09"},
    "stb":   {"operands": "rr", "opcode": b"\x0A"},
    "stw":   {"operands": "rr", "opcode": b"\x0B"},
    "std":   {"operands": "rr", "opcode": b"\x0C"},
    "ldb":   {"operands": "rr", "opcode": b"\x0D"},
    "ldw":   {"operands": "rr", "opcode": b"\x0E"},
    "ldd":   {"operands": "rr", "opcode": b"\x0F"},
    "addi":  {"operands": "ir", "opcode": b"\x10\x10"},
    "subi":  {"operands": "ir", "opcode": b"\x10\x20"},
    "muli":  {"operands": "ir", "opcode": b"\x10\x30"},
    "divi":  {"operands": "ir", "opcode": b"\x10\x40"},
    "andi":  {"operands": "ir", "opcode": b"\x10\x50"},
    "ori":   {"operands": "ir", "opcode": b"\x10\x60"},
    "xori":  {"operands": "ir", "opcode": b"\x10\x70"},
    "shli":  {"operands": "ir", "opcode": b"\x10\x80"},
    "shri":  {"operands": "ir", "opcode": b"\x10\x90"},
    "stbi":  {"operands": "ri", "opcode": b"\x10\xA0"},
    "stwi":  {"operands": "ri", "opcode": b"\x10\xB0"},
    "stdi":  {"operands": "ri", "opcode": b"\x10\xC0"},
    "ldbi":  {"operands": "ir", "opcode": b"\x10\xD0"},
    "ldwi":  {"operands": "ir", "opcode": b"\x10\xE0"},
    "lddi":  {"operands": "ir", "opcode": b"\x10\xF0"},
    "push":  {"operands": "r",  "opcode": b"\x20\x10"},
    "pop":   {"operands": "r",  "opcode": b"\x20\x20"},
    "j":     {"operands": "r",  "opcode": b"\x20\x30"},
    "jt":    {"operands": "r",  "opcode": b"\x20\x40"},
    "jf":    {"operands": "r",  "opcode": b"\x20\x50"},
    "b":     {"operands": "r",  "opcode": b"\x20\x60"},
    "bt":    {"operands": "r",  "opcode": b"\x20\x70"},
    "bf":    {"operands": "r",  "opcode": b"\x20\x80"},
    "pushi": {"operands": "i",  "opcode": b"\x21"},
    "ji":    {"operands": "i",  "opcode": b"\x23"},
    "jti":   {"operands": "i",  "opcode": b"\x24"},
    "jfi":   {"operands": "i",  "opcode": b"\x25"},
    "bi":    {"operands": "i",  "opcode": b"\x26"},
    "bti":   {"operands": "i",  "opcode": b"\x27"},
    "bfi":   {"operands": "i",  "opcode": b"\x28"},
    "ceq":   {"operands": "rr", "opcode": b"\x2C"},
    "cnq":   {"operands": "rr", "opcode": b"\x2D"},
    "cgt":   {"operands": "rr", "opcode": b"\x2E"},
    "clt":   {"operands": "rr", "opcode": b"\x2F"},
    "movi":  {"operands": "ir", "opcode": b"\x30\x10"},
    "ceqi":  {"operands": "ri", "opcode": b"\x30\xC0"},
    "cnqi":  {"operands": "ri", "opcode": b"\x30\xD0"},
    "cgti":  {"operands": "ri", "opcode": b"\x30\xE0"},
    "clti":  {"operands": "ri", "opcode": b"\x30\xF0"},
    "mov":   {"operands": "rr", "opcode": b"\x31"},
}

lark.Tree.__getitem__ = lambda self, index: self.children[index]

class Transfromer(lark.Transformer):
    def __init__(self, definitions):
        self.definitions = definitions
        print(definitions)

    def word(self, node):
        try:
            print(self.definitions[node[0].value])
            return self.definitions[node[0].value]
        except KeyError:
            return node

class Assembler:
    def __init__(self):
        self.code = bytearray()
        self.symbols_def = {}
        self.symbols_use = {}

        self.to_export = []

    def preprocess(self, ast):
        definitions = {}
        for node in ast.children:
            if node.data == "d_export":
                self.to_export.append(node[0][0])
            elif node.data == "d_define":
                definitions[node[0][0].value] = node[1]
        return Transfromer(definitions).transform(ast)

    def assemble(self, ast):
        for node in ast.children:
            # If node is instruction
            if node.data.startswith("i_"):
                instruction = INSTRUCTIONS[node.data[2:]]
                self.code += instruction["opcode"]
                if instruction["operands"] == "rr":
                    # Encode both registers in a byte
                    r1 = int(node[0][0])
                    r2 = int(node[1][0])
                    self.code.append((r1 << 4) | r2)
                elif instruction["operands"] == "ri":
                    r1 = int(node[0][0])
                    if node[1].data == "number":
                        imm = int(node[1][0])
                    elif node[1].data == "label":
                        # Keep track of this so we can fix the address later
                        self.symbols_use[len(self.code)] = (node[1][0], len(self.code) - 2)
                        # Set a temporary value
                        imm = 0xFFFFFFFF
                    self.code[-1] = self.code[-1] | r1
                    # Store int as little endian bytes
                    self.code += struct.pack("<I", imm)
                # Same as "ri", but reverse order
                elif instruction["operands"] == "ir":
                    if node[0].data == "number":
                        imm = int(node[0][0])
                    elif node[0].data == "label":
                        self.symbols_use[len(self.code)] = (node[0][0], len(self.code) - 2)
                        imm = 0xFFFFFFFF
                    r1 = int(node[1][0])
                    self.code[-1] = self.code[-1] | r1
                    self.code += struct.pack("<I", imm)
                elif instruction["operands"] == "r":
                    r1 = int(node[0][0])
                    self.code[-1] = self.code[-1] | r1
                elif instruction["operands"] == "i":
                    if node[0].data == "number":
                        imm = int(node[0][0])
                    elif node[0].data == "label":
                        self.symbols_use[len(self.code)] = (node[0][0], len(self.code) - 1)
                        imm = 0xFFFFFFFF
                    self.code += struct.pack("<I", imm)
            elif node.data == "label_line": # TODO: move to preprocess
                self.symbols_def[node[0][0]] = len(self.code)
            elif node.data in ("d_byte", "d_word", "d_dword"):
                format = {"d_byte": "B", "d_word": "H", "d_dword": "I"}[node.data]
                if len(node.children) == 2:
                    amount = int(node[1][0])
                else:
                    amount = 1
                self.code += struct.pack("<" + format, int(node[0][0])) * amount

    def link(self):
        for pos_use, (symbol, pos_use_opcode) in self.symbols_use.items():
            try:
                pos_def = self.symbols_def[symbol]
            except KeyError:
                print(f"ERROR: unresolved symbol '{symbol}'")
                continue

            # If instruction is branch
            if 0x26 <= self.code[pos_use_opcode] <= 0x28:
                relative = pos_def - pos_use_opcode
                # Change instruction to reverse branch if negative relative address
                if relative < 0:
                    self.code[pos_use_opcode] += 3
                    relative = -relative
                self.code[pos_use:pos_use+4] = struct.pack("<I", relative)
            # If instruction is jump
            else:
                self.code[pos_use:pos_use+4] = struct.pack("<I", pos_def)

if __name__ == "__main__":
    with open("grammar.lark", "r") as f:
        parser = lark.Lark(f.read(), start="program", parser="lalr")

    with open("example.asm", "r") as f:
        ast = parser.parse(f.read())
    
    assembler = Assembler()
    ast = assembler.preprocess(ast)
    assembler.assemble(ast)
    assembler.link()
    for byte in assembler.code:
        print(hex(byte))
    print(ast.pretty())
    print(assembler.symbols_def)
    print(assembler.symbols_use)
    print(assembler.to_export)

    with open("out.bin", "wb") as f:
        f.write(assembler.code)