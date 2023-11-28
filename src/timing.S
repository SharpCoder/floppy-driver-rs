.section .text.main
.extern read_data
.extern data_high
.extern data_low
.global wait_cycle
.global pulse
.global read_sym

T2_5: .word 990
T3_5: .word 1386

@ 3 overhead
wait_cycle:
    push {{lr}}
    top:
    @ Begin the loop
    subs r0,#1
    bcs top
    pop	{{pc}}


pulse:
    push {{r0, r1, r6, lr}} @ 4
    mov r6,r0 @ 1
    bl data_low @ 5
    mov r0,#132 @ 1
    sub r6, #132 @ 1
    bl wait_cycle 
    bl data_high @ 5
    sub r6, #18 @ Everything else
    mov r0,r6 @ 1
    bl wait_cycle 
    pop	{{r0, r1, r6, pc}}


@ This method will read a single flux transition and time it to one 
@ of the 3 known pulse types.
read_sym:
    push {{r1, lr}}
    
    @ Load r1 with 0. This will be our pulse counter
    mov r1,#0

    @ Wait for the pin to be held low
    while_low:
        add r1,#12
        bl read_data
        cmp r0,#0
        beq while_low
    
    @ Wait for the pin to be held high
    while_high:
        add r1,#12
        bl read_data
        cmp r0,#0
        bne while_high
    
    @ Compare the pulses with known limits
    mov r0,r1
    mov r1, #330 @ T2_5
    cmp r0, r1
    bls ret0
    mov r1, #462 @ T3_5
    cmp r0, r1
    bls ret1
    b ret2
    
    ret0:
    mov r0,#0
    pop {{r1, pc}}

    ret1:
    mov r0,#1
    pop {{r1, pc}}

    ret2:
    mov r0,#2
    pop {{r1, pc}}

@ This method will synchronize the clock with
@ 12 bytes of 0x0 
@ 3 bytes of 0xA1 
mfm_sync:
    bx lr