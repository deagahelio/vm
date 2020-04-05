.define count 10

mov #stack $15 ; Set stack pointer
sub 4 $15

mov 1 $1 ; First number
mov 1 $2 ; Second number
mov 2 $4 ; Loop counter
push $1 ; Push computed numbers to stack
push $2

loop:
    mov $2 $3 ; Use r3 to store next number temporarily
    add $1 $3
    push $3
    mov $2 $1 ; Move numbers back to r1 and r2
    mov $3 $2
    add 1 $4 ; Update counter, check if finished
    ceq $4 count
    bf #loop

hang:
    j #hang ; Infinte loop

.dword 0 10 ; Allocate some space for the stack
stack: