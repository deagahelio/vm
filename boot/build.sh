set -e
../tools/kl.py main.kl graphics.kl device.kl utils.kl --comment
../tools/assembler.py init.asm main.kl.out graphics.kl.out device.kl.out utils.kl.out -o boot.bin