.section .text.main

.global wait_cycle
.global pulse_10
.global pulse_100
.global pulse_1000

@ 3 overhead
wait_cycle:
    push {{r0, r1, lr}}
    top:
    @ Begin the loop
    nop
    SUBS r0,#1
    bne top
    pop	{{r0, r1, pc}}

debug_wait_cycle:
    push {{lr}}
    mov r0,#100
    bl wait_cycle
    pop {{pc}}


pulse:
    push {{r1, r6, lr}} @ 4
    mov r6,r0 @ 1
    bl data_low @ 5
    mov r0,#100 @ 1
    bl wait_cycle @ 100
    bl data_high @ 5
    sub r6, #122
    mov r0,r6 @ 1
    bl wait_cycle @ 0
    pop	{{r1, r6, pc}}



pulse_10:
    push {{r0, lr}}
    mov r0, #396
    sub r0, #4
    bl pulse
    pop	{{r0, pc}}


pulse_100:
    push {{r0, lr}}
    mov r0,#594
    sub r0, #4
    bl pulse
    pop	{{r0, pc}}

pulse_1000:
    push {{r0, lr}}
    mov r0, #792
    sub r0, #4
    bl pulse
    pop	{{r0, pc}}


