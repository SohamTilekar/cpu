LDI R2 - 1
OPW-En - 1
LDI R3 - loop
loop:
    ; MUL R1 - R0, R0
    ADD R1 - R0, R0
    ADD R0 - R0, R2
    OPD1W - R0
    OPD2W - R1
    JMP R3
HLT

00000000
62000001
a0000280
63000003
10010000
10200000
a0000000
a1000100
30300000
ffffffff