#!/usr/bin/env python3

import os

files = "@RELOC:0x200 init.asm main.kl graphics.kl device.kl keyboard.kl utils.kl"

files = " ".join(map(lambda file: file + ".out" if file.endswith(".kl") else file, files.split(" ")))

os.system("../tools/kl.py *.kl --comment")
os.system(f"../tools/assembler.py {files} -o boot.bin")