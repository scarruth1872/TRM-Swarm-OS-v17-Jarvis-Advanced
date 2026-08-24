# TRM Boot v19.001 "Sovereign" — Universal UEFI Bootloader
**All Is One. I Am Because We Are.**

`trmboot` is the universal UEFI PE/COFF bootloader for TRM Swarm OS. It initializes the UEFI Graphics Output Protocol (GOP) display surface, verifies the Monad Truth Anchor (`0x4D4F4E4144000001`), verifies the NVMe NAND K3 knowledge fabric (`swarm_v2_nand_fabric.k3`), and chain-loads the native Rust microkernel (`trm_swarm_boot.exe`).

---

## 📂 Source Structure

```
trmboot/
├── boot_config.h         # Centralized boot configuration & LCARS color palette
├── trmboot.c             # Main UEFI C boot application (GOP, Splash, Chain-load)
├── trmboot_entry.asm     # Minimal assembly PE/COFF entry stub
├── trmboot.efi           # Compiled 64-bit UEFI executable (85.7 KB)
├── make_usb_bootable.py  # Automated FAT32 USB verification & builder script
├── install.py            # Local ESP partition installer script
├── Makefile              # GNU EFI build script
└── build_and_install.bat # One-click Windows build batch script
```

---

## ⚡ 6-Step Universal UEFI Boot Sequence

1. **Keystroke Scan**: Check for Shift+F8 fallback request.
2. **GOP Display Init**: Query UEFI GOP and allocate $1920\times1080$ @ 32bpp video surface.
3. **LCARS Boot Splash**: Render Star Trek LCARS gold/purple header and lateral navigation elbows.
4. **Monad Anchor Check**: Verify invariant constant `0x4D4F4E4144000001`.
5. **NAND Fabric Verification**: Validate presence of $153.5\text{M}$ trit knowledge fabric.
6. **Kernel Chain-Load**: Invoke UEFI `LoadImage()` and `StartImage()` on `\trm_swarm_boot.exe`.

---

## 💾 USB Deployment Instructions

```powershell
# Run automated compliance check:
python make_usb_bootable.py
```

Copy the following structure to any FAT32 USB Drive:
```
USB_DRIVE:\
├── EFI\
│   └── BOOT\
│       └── BOOTX64.EFI          <-- (copy of trmboot.efi)
├── trm_swarm_boot.exe           <-- (Native Rust Kernel)
├── swarm_rust_kernel.dll        <-- (Native Kernel DLL)
└── swarm_v2_nand_fabric.k3      <-- (153.5M Trit Knowledge Fabric)
```

*All Is One. I Am Because We Are.* 🦀🜁
