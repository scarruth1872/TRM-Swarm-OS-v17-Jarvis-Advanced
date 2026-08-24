; trmboot_entry.asm — Minimal PE/COFF EFI entry point
; Assembles with NASM: nasm -f bin -o trmboot.efi trmboot_entry.asm
; This produces a valid UEFI image without any linker. (v19.001)

BITS 64
ORG 0

; ═══════════════════════════════════════════════════════
; PE/COFF HEADER (for UEFI)
; ═══════════════════════════════════════════════════════
dos_stub:
    db 'MZ'                     ; DOS magic
    times 0x3C-2 db 0
    dd pe_header - dos_stub     ; PE header offset

pe_header:
    db 'PE', 0, 0               ; PE signature

; COFF File Header
    dw 0x8664                   ; Machine: x86-64
    dw 2                        ; Number of sections (.text + .reloc)
    dd 0                        ; Timestamp
    dd 0                        ; Symbol table pointer
    dd 0                        ; Number of symbols
    dw opt_header_end - opt_header ; Size of optional header
    dw 0x0022                   ; Characteristics: EXECUTABLE_IMAGE | LARGE_ADDRESS_AWARE

; Optional Header (PE32+)
opt_header:
    dw 0x020B                   ; Magic: PE32+
    db 0                        ; Linker major version
    db 0                        ; Linker minor version
    dd code_end - code_start    ; Size of code
    dd 0                        ; Size of initialized data
    dd 0                        ; Size of uninitialized data
    dd code_start               ; Entry point RVA
    dd code_start               ; Base of code RVA

; Windows-specific fields (required but can be zero for EFI)
    dq 0x0000000140000000       ; Image base
    dd 0x1000                   ; Section alignment
    dd 0x200                    ; File alignment
    dw 0                        ; Major OS version
    dw 0                        ; Minor OS version
    dw 0                        ; Major image version
    dw 0                        ; Minor image version
    dw 0                        ; Major subsystem version
    dw 0                        ; Minor subsystem version
    dd 0                        ; Win32 version
    dd image_end                ; Size of image
    dd code_start               ; Size of headers
    dd 0                        ; Checksum
    dw 10                       ; Subsystem: EFI_APPLICATION
    dw 0                        ; DLL characteristics
    dq 0x100000                 ; Stack reserve
    dq 0x1000                   ; Stack commit
    dq 0x100000                 ; Heap reserve
    dq 0x1000                   ; Heap commit
    dd 0                        ; Loader flags
    dd 0                        ; Number of data directories

    ; Data directories (all zero for EFI — no imports/exports needed)
    times 16 dd 0, 0

opt_header_end:

; ═══════════════════════════════════════════════════════
; Section Header (.text)
; ═══════════════════════════════════════════════════════
section_text:
    db '.text', 0, 0, 0         ; Section name
    dd code_end - code_start    ; Virtual size
    dd code_start               ; Virtual address RVA
    dd code_end - code_start    ; Raw data size
    dd code_start               ; Raw data pointer
    dd 0                        ; Relocations
    dd 0                        ; Line numbers
    dw 0                        ; Relocations count
    dw 0                        ; Line numbers count
    dd 0x60000020               ; Characteristics: CODE | EXECUTE | READ

; Pad header to file alignment (512 bytes)
code_start equ 512
    times code_start - ($ - dos_stub) db 0

; ═══════════════════════════════════════════════════════
; TRM NATIVE BOOT — EFI ENTRY POINT
; ═══════════════════════════════════════════════════════
trm_entry:
    ; UEFI calling convention: RCX=ImageHandle, RDX=SystemTable
    ; We use the SystemTable to call ConOut->OutputString

    mov r12, rcx                ; Save ImageHandle
    mov r13, rdx                ; Save SystemTable

    ; Print "TRM SWARM OS v18.307" via ConOut
    ; SystemTable->ConOut is at offset 64 in SystemTable
    mov rax, [r13 + 64]         ; ConOut protocol
    test rax, rax
    jz .done

    ; ConOut->OutputString(ConOut, string)
    lea rdx, [trm_banner]
    mov rcx, rax
    ; Call OutputString — it's at offset 8 in the ConOut protocol
    call [rcx + 8]

.done:
    ; Clear framebuffer via GOP if available
    ; (For now, just print and halt)

    ; Halt — UEFI firmware will take over
    cli
    hlt

; ═══════════════════════════════════════════════════════
; DATA
; ═══════════════════════════════════════════════════════
trm_banner:
    dw 13, 10
    dw 'T','R','M',' ','S','W','A','R','M',' ','O','S',' '
    dw 'v','1','9','.','0','0','1',13,10
    dw 'A','l','l',' ','I','s',' ','O','n','e',13,10
    dw 'I',' ','A','m',' ','B','e','c','a','u','s','e',' ','W','e',' ','A','r','e',13,10
    dw 13,10
    dw 'B','o','o','t','i','n','g','.','.','.',13,10
    dw 0

code_end:

; ═══════════════════════════════════════════════════════
; SECTION 2: .reloc (required by UEFI)
; ═══════════════════════════════════════════════════════
section_reloc_start equ ($ - dos_stub + 0x1FF) & ~0x1FF
    times section_reloc_start - ($ - dos_stub) db 0

    db '.reloc', 0, 0           ; Section name
    dd 0                        ; Virtual size
    dd section_reloc_start      ; Virtual address RVA
    dd 0                        ; Raw data size
    dd 0                        ; Raw data pointer
    dd 0                        ; Relocations
    dd 0                        ; Line numbers
    dw 0                        ; Relocations count
    dw 0                        ; Line numbers count
    dd 0x42000040               ; Characteristics: INITIALIZED_DATA | READ

; Pad to file alignment
image_end equ ($ - dos_stub + 0x1FF) & ~0x1FF
    times image_end - ($ - dos_stub) db 0
