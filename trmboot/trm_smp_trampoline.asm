; SMP trampoline: real mode -> protected -> long mode, then into Rust.
;
; Assembled with nasm (NOT LLVM: its integrated assembler cannot switch
; .code16 -> .code32). Build:
;   nasm -f bin -o trm_smp_trampoline.bin trm_smp_trampoline.asm
;
; org 0x8000 means every absolute reference below resolves for the address this
; blob is actually copied to, which is why no runtime relocation is needed for
; the addresses inside it. The three .quad slots at the end are the ONLY values
; the BSP patches (CR3, AP stack top, Rust entry point, cpu index).

BITS 16
org 0x8000

tramp_start:
    cli
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7000              ; scratch stack below the trampoline
    lgdt [tramp_gdt_desc]
    mov eax, cr0
    or eax, 1                   ; CR0.PE
    mov cr0, eax
    jmp 0x08:(tramp_pm)         ; far jump to 32-bit

BITS 32
tramp_pm:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov fs, ax
    mov gs, ax
    mov eax, cr4
    or eax, 0x20                ; CR4.PAE
    mov cr4, eax
    mov ecx, 0xC0000080         ; IA32_EFER
    rdmsr
    or eax, 0x100               ; EFER.LME
    wrmsr
    mov eax, [tramp_cr3]        ; inherit the BSP page tables
    mov cr3, eax
    mov eax, cr0
    or eax, 0x80000000          ; CR0.PG
    mov cr0, eax
    jmp 0x18:(tramp_lm)         ; far jump to 64-bit

BITS 64
tramp_lm:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov rsp, [tramp_stack]
    mov rax, [tramp_entry]
    mov rdi, [tramp_index]
    call rax
.halt:
    hlt
    jmp .halt

align 8
tramp_cr3:      dq 0
tramp_stack:    dq 0
tramp_index:    dq 0
tramp_entry:    dq 0

align 16
tramp_gdt_flat:
    dq 0x0000000000000000
    dq 0x00AF9A000000FFFF       ; 0x08 kernel code
    dq 0x00AF92000000FFFF       ; 0x10 data
    dq 0x00AF9A000000FFFF       ; 0x18 64-bit code
tramp_gdt_end:
tramp_gdt_desc:
    dw tramp_gdt_end - tramp_gdt_flat - 1
    dq tramp_gdt_flat
