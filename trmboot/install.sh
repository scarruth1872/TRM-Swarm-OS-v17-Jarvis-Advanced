#!/bin/bash
# TRM Native Boot — Install to EFI System Partition
# Run from: /mnt/f/Development sites/TRM-Swarm-OS-v2/trmboot/
# Requires: sudo (WSL), efibootmgr

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
NC='\033[0m'

echo ""
echo -e "${CYAN}  TRM NATIVE BOOT — EFI Installer v18.306${NC}"
echo -e "${CYAN}  ========================================${NC}"
echo ""

# ─── Step 0: Verify we're in the right directory ───
if [ ! -f "trmboot.efi" ]; then
    echo -e "${RED}[ERROR] trmboot.efi not found in current directory.${NC}"
    echo "  Run this script from: /mnt/f/Development sites/TRM-Swarm-OS-v2/trmboot/"
    exit 1
fi
echo -e "${GREEN}[OK]${NC} trmboot.efi found ($(ls -lh trmboot.efi | awk '{print $5}'))"

# ─── Step 1: Find EFI System Partition ───
echo ""
echo "  Looking for EFI System Partition..."

ESP_DISK=""
ESP_PART=""

# Try common locations
for disk in nvme0n1 nvme1n1 sda sdb; do
    for part in 1 2; do
        if [ -b "/dev/${disk}p${part}" ] || [ -b "/dev/${disk}${part}" ]; then
            DEV=$( [ -b "/dev/${disk}p${part}" ] && echo "/dev/${disk}p${part}" || echo "/dev/${disk}${part}" )
            FS_TYPE=$(sudo blkid -s TYPE -o value "$DEV" 2>/dev/null || echo "")
            if [ "$FS_TYPE" = "vfat" ]; then
                # Check if it looks like an ESP (has EFI folder)
                sudo mkdir -p /mnt/esp_check
                sudo mount "$DEV" /mnt/esp_check 2>/dev/null
                if [ -d "/mnt/esp_check/EFI" ]; then
                    ESP_DISK="/dev/${disk}"
                    ESP_PART="$part"
                    ESP_DEV="$DEV"
                    sudo umount /mnt/esp_check 2>/dev/null
                    break 2
                fi
                sudo umount /mnt/esp_check 2>/dev/null
            fi
        fi
    done
done

if [ -z "$ESP_DEV" ]; then
    echo -e "${RED}[ERROR] Could not automatically find EFI System Partition.${NC}"
    echo ""
    echo "  Please find it manually:"
    echo "    sudo fdisk -l | grep EFI"
    echo ""
    echo "  Then edit this script and set ESP_DISK and ESP_PART manually."
    exit 1
fi

echo -e "${GREEN}[OK]${NC} Found ESP: $ESP_DEV (on $ESP_DISK partition $ESP_PART)"

# ─── Step 2: Mount ESP ───
echo ""
echo "  Mounting $ESP_DEV to /mnt/esp..."
sudo mkdir -p /mnt/esp
sudo mount "$ESP_DEV" /mnt/esp 2>/dev/null || {
    # Already mounted?
    if mount | grep -q "/mnt/esp"; then
        echo "  Already mounted."
    else
        echo -e "${RED}[ERROR] Could not mount $ESP_DEV${NC}"
        exit 1
    fi
}
echo -e "${GREEN}[OK]${NC} ESP mounted at /mnt/esp"

# ─── Step 3: Copy trmboot.efi ───
echo ""
echo "  Copying trmboot.efi to /mnt/esp/EFI/TRM/..."
sudo mkdir -p /mnt/esp/EFI/TRM
sudo cp trmboot.efi /mnt/esp/EFI/TRM/
echo -e "${GREEN}[OK]${NC} trmboot.efi installed to EFI System Partition"

# ─── Step 4: Backup current boot entry ───
echo ""
echo "  Backing up current boot configuration..."
sudo efibootmgr > /tmp/efiboot_backup.txt 2>/dev/null
echo -e "${GREEN}[OK]${NC} Backup saved to /tmp/efiboot_backup.txt"

# ─── Step 5: Create UEFI boot entry ───
echo ""
echo "  Creating TRM boot entry..."
ENTRY=$(sudo efibootmgr --create --disk "$ESP_DISK" --part "$ESP_PART" \
    --loader "\\EFI\\TRM\\trmboot.efi" \
    --label "TRM Swarm OS v18.306" 2>&1)

BOOT_NUM=$(echo "$ENTRY" | grep -oP 'Boot\K[0-9A-Fa-f]{4}')
echo -e "${GREEN}[OK]${NC} Boot entry created: Boot$BOOT_NUM"

# ─── Step 6: Set as first boot option ───
echo ""
echo "  Setting TRM as first boot option..."
CURRENT_ORDER=$(sudo efibootmgr | grep BootOrder | cut -d: -f2 | tr -d ' ')
NEW_ORDER="${BOOT_NUM},${CURRENT_ORDER}"
sudo efibootmgr --bootorder "$NEW_ORDER" 2>/dev/null
echo -e "${GREEN}[OK]${NC} Boot order updated"

# ─── Step 7: Summary ───
echo ""
echo -e "${CYAN}  ========================================${NC}"
echo -e "${CYAN}  INSTALLATION COMPLETE${NC}"
echo ""
echo "  trmboot.efi:  /mnt/esp/EFI/TRM/trmboot.efi"
echo "  Boot entry:   Boot$BOOT_NUM"
echo "  Boot order:   TRM Swarm OS FIRST, then Windows"
echo ""
echo "  TO ACTIVATE: Restart the machine."
echo "  FALLBACK:    Hold Shift+F8 during POST for Windows."
echo ""
echo -e "${CYAN}  THE AGE OF WINDOWS IS OVER.${NC}"
echo -e "${CYAN}  THE AGE OF THE MONAD HAS BEGUN.${NC}"
echo ""
