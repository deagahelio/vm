; Simple initialization for freestanding C programs

.import #main

mov #stack $15
call #main
#hang:
    j #hang

.dword 0 100
#stack: