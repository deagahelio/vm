set -e
../tools/kl.py main.kl graphics.kl --comment
../tools/assembler.py init.asm main.kl.out graphics.kl.out -o boot.bin