.section .text

.extern read_data
.extern data_high
.extern data_low
.extern fdd_read_index

.global _asm_pulse
.global _asm_read_sym
.global _asm_sync

wait_cycle:
    push {{lr}}
    top:
    subs r0,#1
    bcs top
    pop	{{pc}}


_asm_pulse:
    push {{r0, r6, lr}}
    mov r6,r0
    bl data_low 
    sub r6, #148
    mov r0, #148
    bl wait_cycle 
    bl data_high
    mov r0,r6
    bl wait_cycle 
    pop	{{r0, r6, pc}}


@ This method will read a single flux transition and time it to one 
@ of the 3 known pulse types.
_asm_read_sym:
    push {{r1, lr}}
    
    @ Load r1 with 0. This will be our pulse counter
    mov r1,#0

    @ Wait for the pin to be held low
    while_low:
        add r1,#5
        bl read_data
        cmp r0,#0
        beq while_low
    
    @ Wait for the pin to be held high
    while_high:
        add r1,#5
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
_asm_sync:
    push {{r1,r2,r3,lr}}
    mov r1, #0

    full_restart:
    @ Check if we're past the index
    bl fdd_read_index
    cmp r0,#0
    @ beq err
    

    restart:
    @ If we encounter something that isn't a short pulse
    @ first check if we've collected enough to indicate
    @ that we're ready for signal processing.
    cmp r1,#80
    bge process_signal
    
    @ Otherwise, do the actual restart
    mov r1,#0

    s80_loop: @ Short 80 pulses
    @ Read a symbol, if its a short, increment
    @ otherwise, reset. Compare with #80 and
    @ if matching, then break to next segment

    bl _asm_read_sym
    cmp r0,#0
    bne restart @ restart if its not 0
    add r1,#1
    cmp r1,#80
    b s80_loop @ loop if we're less than 80 yet

    @ If we get here, we've found 80 pulses
    @ M L M L M S L M L M S L M L M
    process_signal:

    @ Reset the short pulses
    mov r1,#0

    @ When we get here, we've still got one pulse in the buffer
    @ so we can directly evaluate it.
    cmp r0,#1 @ M
    bne full_restart

    bl _asm_read_sym
    cmp r0,#2 @ L
    bne full_restart

    bl _asm_read_sym
    cmp r0,#1 @ M
    bne full_restart

    bl _asm_read_sym
    cmp r0,#2 @ L
    bne full_restart

    bl _asm_read_sym
    cmp r0,#1 @ M
    bne full_restart

    bl _asm_read_sym
    cmp r0,#0 @ S
    bne full_restart

    bl _asm_read_sym
    cmp r0,#2 @ L
    bne full_restart

    bl _asm_read_sym
    cmp r0,#1 @ M
    bne full_restart

    bl _asm_read_sym
    cmp r0,#2 @ L
    bne full_restart

    bl _asm_read_sym
    cmp r0,#1 @ M
    bne full_restart

    bl _asm_read_sym
    cmp r0,#0 @ S
    bne full_restart

    bl _asm_read_sym
    cmp r0,#2 @ L
    bne full_restart

    bl _asm_read_sym
    cmp r0,#1 @ M
    bne full_restart

    bl _asm_read_sym
    cmp r0,#2 @ L
    bne full_restart

    bl _asm_read_sym
    cmp r0,#1 @ M
    bne full_restart

    mov r0,#1
    pop {{r1,r2,r3,pc}}

    @ This condition signifies that we've
    @ hit an index loop and must terminate
    @ during this cycle.
    err:
    mov r0,#0
    pop {{r1,r2,r3,pc}}
