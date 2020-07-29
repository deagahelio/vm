# vm

This repo contains a set of development tools for a RISC, 32-bit, yet-to-be-named ISA.

## `tools/assembler.py`

An assembler that also works as a linker. The syntax for the assembly language is defined in `tools/grammar.lark`. Supports basic features such as labels, constants, and data allocation directives.

## `tools/kl.py`

A compiler for a low-level, lisp-like syntax language named KL, designed to be easy to parse and compile. An example of how the language works can be found in `tools/example.kl`. Features nested expressions, named variables, functions, type checking, arrays, loops and more.

[See the wiki page for a more in-depth overview of the language.](https://github.com/deagahelio/vm/wiki/KL)

## `tools/compiler.py`

A work-in-progress C compiler. Only basic features are implemented. Not in active development.

## `vm/`

A basic virtual machine. Includes a simple framebuffer implementation.