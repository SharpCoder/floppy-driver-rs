.section .text.main

.global wait_cycle
.global pulse10

@ 3 overhead
wait_cycle:
    push {{r0, r7, lr}}
    mov	r7, sp
    top:
    @ Begin the loop
    SUBS r0,r0,#1
    bne top
    pop	{{r0, r7, pc}}

@ This method will take 156 additionalcycles
pulse:
    push {{r0, r6, r7, lr}}
    mov	r7, sp
    mov r6,r0 @ 1
    bl data_low @ 5
    mov r0,#129 @ 1
    bl wait_cycle @ 132
    bl data_high @ 5
    mov r0,r6 @ 1
    bl wait_cycle @ 0
    pop	{{r0, r6, r7, pc}}



@ 264 cycles in total
@ 5
pulse_10:
    push {{r0, r7, lr}}
    mov	r7, sp
    mov r0, #108
    bl pulse
    pop	{{r0, r7, pc}}

@ 396 cycles in total
pulse_100:
    push {{r0, r7, lr}}
    mov	r7, sp
    mov r0, #240
    bl pulse
    pop	{{r0, r7, pc}}

@ 528 cycles in total
pulse_1000:
    push {{r0, r7, lr}}
    mov	r7, sp
    mov r0, #372
    bl pulse
    pop	{{r0, r7, pc}}


