    org $0000

    lda #$01
    adc #$01
    sta $0000 ; after the program halts, check that this byte equals 2
halt
    jmp halt
