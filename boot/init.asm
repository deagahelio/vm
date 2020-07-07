.import #main

mov #stack $15
jal #main
#hang:
    b #hang

.dword 0 100
#stack: