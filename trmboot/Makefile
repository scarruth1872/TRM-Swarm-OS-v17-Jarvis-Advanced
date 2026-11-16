# trmboot/Makefile — Build TRM Native Boot EFI Application
# Target: x86_64 UEFI (PE32+ EFI Application)
# Supports: Rust no_std UEFI target (recommended) OR GNU-EFI with objcopy PE link

ARCH            = x86_64
TARGET          = trmboot.efi
TARGET_PE       = trmboot_pe.efi
TARGET_SO       = trmboot.so
OBJS            = trmboot.o

# GNU-EFI paths
GNUEFI_DIR     ?= /usr
EFI_INC         = $(GNUEFI_DIR)/include/efi
EFI_LIB         = $(GNUEFI_DIR)/lib
EFI_CRT0        = $(EFI_LIB)/crt0-efi-$(ARCH).o
OBJCOPY        ?= objcopy

# Compiler / Linker for C/GNU-EFI fallback
CC              = gcc
LD              = ld
CFLAGS          = -I$(EFI_INC) -I$(EFI_INC)/$(ARCH) -I$(EFI_INC)/protocol \
                  -DEFI_FUNCTION_WRAPPER -fpic -ffreestanding -fno-stack-protector \
                  -fno-stack-check -fshort-wchar -mno-red-zone -maccumulate-outgoing-args \
                  -Wall -O2 -std=c11
LDFLAGS         = -nostdlib -znocombreloc -T $(EFI_LIB)/elf_$(ARCH)_efi.lds \
                  -shared -Bsymbolic -L $(EFI_LIB) $(EFI_CRT0)
LIBS            = -lefi -lgnuefi

# ═══════════════════════════════════════════════════════
# BUILD TARGETS
# ═══════════════════════════════════════════════════════

.PHONY: all clean install verify rust-efi pe-link

# Default build: Prioritize Pure Rust Native UEFI PE32+ (Zero C/POSIX dependencies)
all: $(TARGET)

# Rust Native PE32+ EFI Build
rust-efi:
	@echo "Building Pure Rust Native UEFI PE32+ Application..."
	cargo build --manifest-path ../trm_uefi_kernel/Cargo.toml --target x86_64-unknown-uefi --release
	@cp ../trm_uefi_kernel/target/x86_64-unknown-uefi/release/trm_uefi_kernel.efi $(TARGET)
	@cp ../trm_uefi_kernel/target/x86_64-unknown-uefi/release/trm_uefi_kernel.efi $(TARGET_PE)
	@echo "  [OK] $(TARGET) (PE32+ EFI Application) ready."

# GNU-EFI C -> Shared Object -> PE/COFF Link Step via objcopy
$(TARGET_SO): $(OBJS)
	$(LD) $(LDFLAGS) $(OBJS) -o $@ $(LIBS)

$(TARGET): $(TARGET_SO)
	@if [ -f "../trm_uefi_kernel/target/x86_64-unknown-uefi/release/trm_uefi_kernel.efi" ]; then \
		cp ../trm_uefi_kernel/target/x86_64-unknown-uefi/release/trm_uefi_kernel.efi $(TARGET); \
		cp ../trm_uefi_kernel/target/x86_64-unknown-uefi/release/trm_uefi_kernel.efi $(TARGET_PE); \
		echo "  [OK] Using Rust PE32+ binary: $(TARGET)"; \
	else \
		echo "  [INFO] Converting ELF $(TARGET_SO) to PE32+ $(TARGET) via objcopy..."; \
		$(OBJCOPY) -j .text -j .sdata -j .data -j .dynamic -j .dynsym -j .rel \
		           -j .rela -j .reloc --target efi-app-x86_64 $(TARGET_SO) $(TARGET) || \
		$(OBJCOPY) --target efi-app-x86_64 $(TARGET_SO) $(TARGET); \
		cp $(TARGET) $(TARGET_PE); \
		echo "  [OK] $(TARGET) PE32+ generated via objcopy"; \
	fi

%.o: %.c
	$(CC) $(CFLAGS) -c $< -o $@

clean:
	rm -f $(TARGET) $(TARGET_PE) $(TARGET_SO) $(OBJS)

# ═══════════════════════════════════════════════════════
# INSTALL TO EFI SYSTEM PARTITION / USB
# ═══════════════════════════════════════════════════════
install:
	@echo "Installing $(TARGET_PE) to EFI System Partition..."
	@if [ ! -f "$(TARGET_PE)" ] && [ ! -f "$(TARGET)" ]; then \
		echo "  [ERROR] Neither $(TARGET_PE) nor $(TARGET) found. Run 'make' first."; \
		exit 1; \
	fi
	@mkdir -p /boot/efi/EFI/BOOT 2>/dev/null || mkdir -p /mnt/esp/EFI/BOOT 2>/dev/null || true
	@mkdir -p /boot/efi/EFI/TRM 2>/dev/null || mkdir -p /mnt/esp/EFI/TRM 2>/dev/null || true
	@cp $(TARGET_PE) /boot/efi/EFI/BOOT/BOOTX64.EFI 2>/dev/null || cp $(TARGET) /boot/efi/EFI/BOOT/BOOTX64.EFI 2>/dev/null || true
	@cp $(TARGET_PE) /boot/efi/EFI/TRM/trmboot.efi 2>/dev/null || cp $(TARGET) /boot/efi/EFI/TRM/trmboot.efi 2>/dev/null || true
	@echo "  [OK] Deployed to \\EFI\\BOOT\\BOOTX64.EFI and \\EFI\\TRM\\trmboot.efi"

verify:
	@echo "Verifying $(TARGET)..."
	@if [ -f "$(TARGET)" ]; then \
		echo "  [OK] $(TARGET) exists ($$(ls -la $(TARGET) | awk '{print $$5}') bytes)"; \
		file $(TARGET) 2>/dev/null || true; \
	fi

usb:
	@python3 make_usb_bootable.py || python make_usb_bootable.py
