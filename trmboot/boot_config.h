/**
 * boot_config.h — TRM Swarm OS Centralized Boot Configuration
 * v19.001 — Production Bare-Metal Boot
 *
 * Single source of truth for all boot constants, paths, colors, and version info.
 * Included by trmboot.c and any future EFI modules.
 *
 * All Is One. I Am Because We Are.
 */

#ifndef TRM_BOOT_CONFIG_H
#define TRM_BOOT_CONFIG_H

/* ─── Version ─── */
#define TRM_VERSION_MAJOR       19
#define TRM_VERSION_MINOR       1
#define TRM_VERSION_STRING      "19.001"
#define TRM_VERSION_WIDE        L"19.001"
#define TRM_CODENAME            "Sovereign"

/* ─── Boot Behavior ─── */
#define TRM_BOOT_TIMEOUT_SEC    3
#define TRM_FALLBACK_SCANCODE   0x42    /* F8 key */

/* ─── Monad Truth Anchor ─── */
#define TRM_TRUTH_ANCHOR        0x4D4F4E4144000001ULL

/* ─── File Paths (EFI filesystem, backslash) ─── */
#define TRM_KERNEL_PATH         L"\\trm_swarm_boot.exe"
#define TRM_KERNEL_PATH_EFI     L"\\EFI\\TRM\\trm_swarm_boot.exe"
#define TRM_NAND_FABRIC_PATH    L"\\swarm_v2_nand_fabric.k3"
#define TRM_RUST_DLL_PATH       L"\\swarm_rust_kernel.dll"
#define TRM_EFI_DIR             L"\\EFI\\TRM"

/* ─── LCARS Color Palette (BGRA for GOP framebuffer) ─── */
#define LCARS_BG_DARK           0xFF0A0A12  /* Deep Space Dark #0a0a12   */
#define LCARS_GOLD              0xFFFF9900  /* LCARS Gold #ff9900        */
#define LCARS_PURPLE            0xFF7C4DFF  /* LCARS Purple #7c4dff      */
#define LCARS_BLUE              0xFF6688CC  /* LCARS Blue #6688cc        */
#define LCARS_ORANGE            0xFFFF6633  /* LCARS Orange #ff6633      */
#define LCARS_RED               0xFFCC3333  /* LCARS Alert Red #cc3333   */
#define LCARS_GREEN             0xFF33CC66  /* LCARS Status Green #33cc66*/
#define LCARS_WHITE             0xFFEEEEFF  /* LCARS Text White #eeeeff  */
#define LCARS_CYAN              0xFF66CCFF  /* LCARS Cyan #66ccff        */

/* ─── Boot Phases ─── */
#define TRM_PHASE_FIRMWARE      0   /* UEFI firmware init    */
#define TRM_PHASE_GOP           1   /* Graphics output init  */
#define TRM_PHASE_LCARS         2   /* LCARS splash render   */
#define TRM_PHASE_ANCHOR        3   /* Monad truth verify    */
#define TRM_PHASE_NAND_CHECK    4   /* NAND fabric check     */
#define TRM_PHASE_KERNEL_LOAD   5   /* Rust kernel load      */
#define TRM_PHASE_KERNEL_START  6   /* Rust kernel start     */
#define TRM_PHASE_COUNT         7

/* ─── Agent Configuration ─── */
#define TRM_NUM_CORE_AGENTS     14
#define TRM_NAND_TRIT_COUNT     153500000   /* 153.5M trits */

/* ─── QUIC Transport ─── */
#define TRM_QUIC_PORT           8021
#define TRM_QUIC_PROTOCOL       "trm://"

/* ─── Hardware Sovereign Assets ─── */
#define TRM_WIFI_BAND_24GHZ     2400    /* MHz */
#define TRM_WIFI_BAND_5GHZ      5000    /* MHz */
#define TRM_BT_BAND             2400    /* MHz */
#define TRM_SPECTRUM_MIN_HZ     7.83    /* Schumann */
#define TRM_SPECTRUM_MAX_HZ     10e12   /* 10 THz */

#endif /* TRM_BOOT_CONFIG_H */
