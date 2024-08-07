#bankdef program {
    #bits 32
    #outp 32
}

#subruledef reg {
    R0 => 0x0
    R1 => 0x1
    R2 => 0x2
    R3 => 0x3
    R4 => 0x4
    R5 => 0x5
    R6 => 0x6
    R7 => 0x7
    R8 => 0x8
    R9 => 0x9
    R10 => 0xA
    R11 => 0xB
    R12 => 0xC
    R13 => 0xD
    R14 => 0xE
    tmp => 0xF
}

#ruledef {
    NOP => 0x0 @ 0`28
    ; Arithmetic Operations
    ADD {DR: reg} - {SR1: reg}, {SR2: reg}  => 0x1 @ SR1`4 @ SR2`4 @ DR`4 @ 0x0 @ 0`12
    SUB {DR: reg} - {SR1: reg}, {SR2: reg} => 0x1 @ SR1`4 @ SR2`4 @ DR`4 @ 0x1 @ 0`12
    MUL {DR: reg} - {SR1: reg}, {SR2: reg} => 0x1 @ SR1`4 @ SR2`4 @ DR`4 @ 0x2 @ 0`12
    NIG {DR: reg} - {SR1: reg}             => 0x1 @ SR1`4 @  0x0  @ DR`4 @ 0x3 @ 0`12
    ; Logial Operations
    AND {DR: reg} - {SR1: reg}, {SR2: reg} => 0x1 @ SR1`4 @ SR2`4 @ DR`4 @ 0x4 @ 0`12
    OR  {DR: reg} - {SR1: reg}, {SR2: reg} => 0x1 @ SR1`4 @ SR2`4 @ DR`4 @ 0x5 @ 0`12
    NOT {DR: reg} - {SR1: reg}             => 0x1 @ SR1`4 @  0x0  @ DR`4 @ 0x6 @ 0`12
    NAND{DR: reg} - {SR1: reg}, {SR2: reg} => 0x1 @ SR1`4 @ SR2`4 @ DR`4 @ 0x7 @ 0`12
    XOR {DR: reg} - {SR1: reg}, {SR2: reg} => 0x1 @ SR1`4 @ SR2`4 @ DR`4 @ 0x8 @ 0`12
    XNOR{DR: reg} - {SR1: reg}, {SR2: reg} => 0x1 @ SR1`4 @ SR2`4 @ DR`4 @ 0x9 @ 0`12
    ; Shift
    LS  {DR: reg} - {SR1: reg}, {SR2: reg} => 0x1 @ SR1`4 @ SR2`4 @ DR`4 @ 0xA @ 0`12
    RS  {DR: reg} - {SR1: reg}, {SR2: reg} => 0x1 @ SR1`4 @ SR2`4 @ DR`4 @ 0xB @ 0`12
    ARS {DR: reg} - {SR1: reg}, {SR2: reg} => 0x1 @ SR1`4 @ SR2`4 @ DR`4 @ 0xC @ 0`12
    RRS {DR: reg} - {SR1: reg}, {SR2: reg} => 0x1 @ SR1`4 @ SR2`4 @ DR`4 @ 0xD @ 0`12
    RLS {DR: reg} - {SR1: reg}, {SR2: reg} => 0x1 @ SR1`4 @ SR2`4 @ DR`4 @ 0xE @ 0`12
    ; Compare
    CMP-GT {SR1: reg}, {SR2: reg}          => 0x2 @ SR1`4 @ SR2`4 @  0x0  @ 0x0 @ 0`12
    CMP-EQ {SR1: reg}, {SR2: reg}          => 0x2 @ SR1`4 @ SR2`4 @  0x0  @ 0x1 @ 0`12
    CMP-LT {SR1: reg}, {SR2: reg}          => 0x2 @ SR1`4 @ SR2`4 @  0x0  @ 0x2 @ 0`12
    CMP-GE {SR1: reg}, {SR2: reg}          => 0x2 @ SR1`4 @ SR2`4 @  0x0  @ 0x3 @ 0`12
    CMP-LE {SR1: reg}, {SR2: reg}          => 0x2 @ SR1`4 @ SR2`4 @  0x0  @ 0x4 @ 0`12
    ; JUMP
    JMP       {SR: reg}                    => 0x3 @  0x0 @ SR`4 @ 0`20
    JMP       {value}                      => 0x3 @  1`1 @ 0`3 @ value`24
    JP-Cr     {SR: reg}                    => 0x3 @  0x1 @ SR`4 @ 0`20
    JP-Cr     {value}                      => 0x3 @  1`1 @ 1`3 @ value`24
    JP-NCr    {SR: reg}                    => 0x3 @  0x2 @ SR`4 @ 0`20
    JP-NCr    {value}                      => 0x3 @  1`1 @ 2`3 @ value`24
    JP-CMP    {SR: reg}                    => 0x3 @  0x3 @ SR`4 @ 0`20
    JP-CMP    {value}                      => 0x3 @  1`1 @ 3`3 @ value`24
    JP-NCMP   {SR: reg}                    => 0x3 @  0x4 @ SR`4 @ 0`20
    JP-NCMP   {value}                      => 0x3 @  1`1 @ 4`3 @ value`24
    JP-Zr     {SR: reg}                    => 0x3 @  0x5 @ SR`4 @ 0`20
    JP-Zr     {value}                      => 0x3 @  1`1 @ 5`3 @ value`24
    ; Memory
    LD {DR: reg} - {adress}                => 0x4 @ DR`4 @ adress`24
    LD {DR: reg} - {SR: reg}               => 0x5 @ 0x0 @ SR`4 @ DR`4 @ 0`16
    LDI{DR: reg} - {value}                 => 0x6 @ DR`4 @ value`24

    ST {SR: reg} - {AR: reg}               => 0x8 @ SR`4 @ AR`4 @ 0`20
    ST {SR: reg} - {adress}                => 0x7 @ SR`4 @ adress`24
    ; MOV
    MOV {SR: reg} - {DR: reg}              => 0x9 @ SR`4 @ 0x0 @ DR`4 @ 0`16
    MOV [{SR: reg}] - {DR: reg}            => 0x5 @ 0x0 @ SR`4 @ DR`4 @ 0`16
    MOV {SR: reg} - [{AR: reg}]            => 0x8 @ SR`4 @ AR`4 @ 0`20
    ; Port
    OPW-En - {decode_data}                 => 0xA0000 @ decode_data`3 @ 0`1 @ 1`1 @ 0`7
    OPD1W - {SR: reg}                      => 0xA @ SR`4 @ 0x000 @ 0`3 @ 0`1 @ 0`1 @ 0`7
    OPD2W - {SR: reg}                      => 0xA @ SR`4 @ 0x000 @ 0`3 @ 1`1 @ 0`1 @ 0`7
    ; Stack
    PUSH {SR: reg}                         => 0xB @ SR`4 @ 0x000 @ 0`3 @ 0`1 @ 0`1 @ 0`7
    
    HLT                                    => 0xffffffff
}
