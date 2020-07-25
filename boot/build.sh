set -e
../tools/kl.py main.kl graphics.kl device.kl keyboard.kl utils.kl --comment
../tools/assembler.py @RELOC:0x200 init.asm main.kl.out graphics.kl.out device.kl.out keyboard.kl.out utils.kl.out -o boot.bin