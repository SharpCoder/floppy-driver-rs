.section .text.main

.global wait_cycle
.global pulse10

@ 3 overhead
wait_cycle:
    push {{r0, r1, r7, lr}}
    mov	r7, sp
    mov r1, r0
    mov r0, #11 @ Total overhead so far
    top:
    @ Begin the loop
    ADD r0,r0,#3
    CMP r0,r1
    bhs top
    pop	{{r0, r1, r7, pc}}

debug_wait_cycle:
    mov r0,#1337
    bx lr


pulse:
    push {{r0, r1, r6, r7, lr}} @ 5
    mov	r7, sp @ 1
    mov r6,r0 @ 1
    bl data_low @ 5
    mov r0,#129 @ 1
    bl wait_cycle @ 129
    bl data_high @ 5
    sub r6, r6, 16
    mov r0,r6 @ 1
    bl wait_cycle @ 0
    pop	{{r0, r1, r6, r7, pc}}



@ 264 cycles in total
pulse_10:
    push {{r0, r7, lr}}
    mov	r7, sp
    mov r0, #264
    sub r0, r0, #6
    bl pulse
    pop	{{r0, r7, pc}}

@ 396 cycles in total
pulse_100:
    push {{r0, r7, lr}}
    mov	r7, sp
    mov r0,#396
    sub r0, r0, #6
    bl pulse
    pop	{{r0, r7, pc}}

@ 528 cycles in total
pulse_1000:
    push {{r0, r7, lr}}
    mov	r7, sp
    mov r0, #528
    sub r0, r0, #6
    bl pulse
    pop	{{r0, r7, pc}}


