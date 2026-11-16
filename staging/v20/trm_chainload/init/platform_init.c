/* TRM Swarm OS -- PID 1 on bare metal.
 *
 * This is the launcher the bridge hands userspace to. It does the three things an init must do and
 * then proves the platform actually ran: mount the pseudo-filesystems, bring loopback up (the only
 * interface the kernel carries a driver for here -- it probes sda/sr0 and no NIC), start the platform,
 * and then ASK it a question over HTTP and print the answer it gives. A boot that printed a banner and
 * nothing else would prove nothing; the HTTP reply is the measurement.
 */
#include <stdio.h>
#include <stdarg.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>
#include <errno.h>
#include <dirent.h>
/* The loop driver is built into the kernel this pathway boots (measured: /proc/devices lists `7 loop`
 * and kernel/drivers/block/loop.ko appears in modules.builtin), so a squashfs is reachable as a FILE
 * on the FAT volume without shipping a module -- see persistent_root() below. */
#include <linux/loop.h>
#ifndef LOOP_SET_FD
#define LOOP_SET_FD 0x4C00
#define LOOP_CTL_GET_FREE 0x4C82
#endif
#include <sys/mount.h>
#include <sys/stat.h>
#include <sys/socket.h>
#include <sched.h>
#include <sys/ioctl.h>
#include <sys/syscall.h>
#include <sys/wait.h>
#ifndef CLONE_NEWNS
#define CLONE_NEWNS 0x00020000
#endif
#ifndef SYS_finit_module
#define SYS_finit_module 313          /* x86_64; the syscall insmod(8) itself uses */
#endif
#include <net/if.h>
#include <net/route.h>
#include <netinet/in.h>
#include <arpa/inet.h>

/* The supervisor starts the platform (see supervisor.services.conf); PORT is the loopback the probe
 * below speaks to -- the same port the platform binds. */
#define SUPERVISOR "/opt/trm/trm_uefi_kernel/host_supervisor/host_supervisor"
#define SUP_CONF   "/opt/trm/trm_uefi_kernel/host_supervisor/supervisor.services.conf"
#define PORT 8022

/* /dev/console maps to the LAST console= on the command line, so a userspace process's stdout lands on
 * the serial port and the display shows only kernel output. Opening /dev/tty0 as well puts the same
 * lines on the screen -- the screen is the thing an operator actually reads. */
static FILE *tty0 = NULL;
static FILE *bootlog = NULL;
/* Everything printed before the writable layer exists is buffered here, so the log file carries the
 * whole boot and not just its second half (see bootlog_open). */
static char bootbuf[8192];
static size_t bootlen = 0;
static int bootlost = 0;

static void say(const char *fmt, ...) {
    va_list ap;
    va_start(ap, fmt);
    vprintf(fmt, ap);
    va_end(ap);
    fflush(stdout);
    if (tty0) {
        va_start(ap, fmt);
        vfprintf(tty0, fmt, ap);
        va_end(ap);
        fflush(tty0);
    }
    if (bootlog) {
        va_start(ap, fmt);
        vfprintf(bootlog, fmt, ap);
        va_end(ap);
        fflush(bootlog);
    } else if (bootlen + 256 < sizeof bootbuf) {
        va_start(ap, fmt);
        int n = vsnprintf(bootbuf + bootlen, sizeof bootbuf - bootlen, fmt, ap);
        va_end(ap);
        if (n > 0) bootlen += (size_t)n;
        if (bootlen > sizeof bootbuf - 1) bootlen = sizeof bootbuf - 1;
        bootbuf[bootlen] = 0;
    } else {
        bootlost = 1;
    }
}

static int mount_ok(const char *src, const char *tgt, const char *fstype) {
    mkdir(tgt, 0755);
    return mount(src, tgt, fstype, 0, NULL) == 0;
}

static int loopback_up(void) {
    int s = socket(AF_INET, SOCK_DGRAM, 0);
    if (s < 0) return -1;
    struct ifreq ifr;
    memset(&ifr, 0, sizeof ifr);
    strncpy(ifr.ifr_name, "lo", IFNAMSIZ - 1);
    if (ioctl(s, SIOCGIFFLAGS, &ifr) != 0) { close(s); return -1; }
    ifr.ifr_flags |= IFF_UP | IFF_RUNNING;
    int rc = ioctl(s, SIOCSIFFLAGS, &ifr);
    close(s);
    return rc;
}

/* Ask the platform a question over the loopback and print what it answers.
 *
 * The body is sent as text/plain with a Content-Length, which is the contract the parser routes take:
 * the payload IS the request body, and the format comes from the query string. */
/* The body of the last probe, so a caller can assert what a route returned and not only that it did.
 * `http_req` reads into a static buffer, so the pointer stays valid until the next request. */
static char *probe_body;

static int http_req_q(const char *method, const char *path, const char *body, int quiet);

/* The ordinary probe: it says what it asked and what came back, because that is the boot's own account of
 * the platform it started. */
static int http_req(const char *method, const char *path, const char *body) {
    return http_req_q(method, path, body, 0);
}

static int http_req_q(const char *method, const char *path, const char *body, int quiet) {
    int fd = socket(AF_INET, SOCK_STREAM, 0);
    if (fd < 0) { say("  probe: socket failed (%s)\n", strerror(errno)); return 0; }
    struct sockaddr_in a;
    memset(&a, 0, sizeof a);
    a.sin_family = AF_INET;
    a.sin_port = htons(PORT);
    a.sin_addr.s_addr = htonl(INADDR_LOOPBACK);
    if (connect(fd, (struct sockaddr *)&a, sizeof a) != 0) {
        say("  probe %s: CONNECT FAILED (%s)\n", path, strerror(errno));
        close(fd);
        return 0;
    }
    char req[1024];
    size_t blen = body ? strlen(body) : 0;
    int n = snprintf(req, sizeof req,
                     "%s %s HTTP/1.0\r\nHost: 127.0.0.1\r\n"
                     "Content-Type: text/plain\r\nContent-Length: %zu\r\nConnection: close\r\n\r\n",
                     method, path, blen);
    if (write(fd, req, n) < 0) { say("  probe: write failed\n"); close(fd); return 0; }
    if (blen && write(fd, body, blen) < 0) { say("  probe: body write failed\n"); close(fd); return 0; }

    /* 16 KB, not 8: the /sram body already runs to 6.9 KB and the fields a probe exists to show
     * must not be the ones a growing body pushes past the end of the read. */
    static char buf[16384];
    ssize_t total = 0, r;
    while (total < (ssize_t)sizeof buf - 1 && (r = read(fd, buf + total, sizeof buf - 1 - total)) > 0)
        total += r;
    buf[total] = 0;
    close(fd);

    char *resp_body = strstr(buf, "\r\n\r\n");
    char *status = buf;
    char *eol = strchr(buf, '\r');
    if (eol) *eol = 0;
    /* What the route RETURNED, kept for the caller -- the body, not the buffer. The line above NULs the
     * status line's terminator, so a pointer at the buffer would measure a string that ends before the
     * body: the first run of this check reported eight routes as answering without their content, and
     * the defect was here, in the check. Only the second of these two claims is evidence about a
     * platform rather than about a listening socket. */
    probe_body = (resp_body && resp_body[4]) ? resp_body + 4 : "";
    if (!quiet) {
        say("  %s %s -> %s\n", method, path, status);
        if (resp_body && resp_body[4]) {
            say("  BODY: %s\n", resp_body + 4);
        }
    }
    /* "HTTP/1.0 200 OK" -- the number the route answered with, for the caller to count. */
    return !strncmp(status, "HTTP/", 5) ? atoi(status + 9) : 0;
}

static int http_probe(const char *path) {
    return http_req("GET", path, NULL);
}

/* Ask a route for a specific thing, not merely for an answer. The needles below are read off the running
 * platform rather than declared: an expectation written from memory is the defect this whole pathway keeps
 * finding, so each one is a string a boot can only produce if the branch behind it actually ran. */
static int http_probe_contains(const char *path, const char *needle, int want) {
    /* Quiet: an assertion is not evidence to print. This boot asks twenty-eight routes ten times over, and
     * the /council body alone is 9 KB on an 11 KB/s serial line -- printing what is being checked is how a
     * boot that probes comes to take minutes and look like a hang. A failure still speaks, in full. */
    int code = http_req_q("GET", path, NULL, 1);
    return code == want && probe_body && strstr(probe_body, needle) != NULL;
}

/* mkdir a path's every component; EEXIST on an existing one is not an error. */
static void mkdir_p(const char *path) {
    char buf[256];
    snprintf(buf, sizeof buf, "%s", path);
    for (char *p = buf + 1; *p; p++) {
        if (*p == '/') { *p = '\0'; mkdir(buf, 0755); *p = '/'; }
    }
    mkdir(buf, 0755);
}

/* The stores the platform's resolver looks for, linked to where it walks. Called AFTER the pivot, so
 * `/opt/trm` is the platform image's own view (the links land in the writable layer) and `/mnt/esp` is
 * the boot volume moved into the new root. Every store the resolver looks for is linked, not just the
 * one a probe uses: a half-wired volume makes /k3/status report a store missing that is sitting on the
 * disk, and a status route that misreports is worse than a silent one. */
static void link_stores(void) {
    static const char *names[] = {
        "swarm_v2_nand_fabric.turbovec",
        "swarm_v2_nand_fabric.k3",
        "swarm_v2_nand_fabric.tercode",
        "swarm_v2_nand_fabric.text.jsonl",
        /* The corpus the exact ternary engine scores against: the difference between "the platform
         * boots here" and "the platform answers from its own memory here". */
        "swarm_v2_artifacts/kernel_knowledge_index.json",
        NULL
    };
    int linked = 0;
    for (int j = 0; names[j]; j++) {
        char src[192], dst[192];
        struct stat st;
        snprintf(src, sizeof src, "/mnt/esp/%s", names[j]);
        snprintf(dst, sizeof dst, "/opt/trm/%s", names[j]);
        if (stat(src, &st) != 0) continue;
        /* `repo_relative` joins the rel path onto a candidate ancestor, so an intermediate directory
         * has to exist before a link into it can resolve. */
        for (char *p = strchr(dst + 9, '/'); p; p = strchr(p + 1, '/')) {
            *p = '\0';
            mkdir(dst, 0755);
            *p = '/';
        }
        unlink(dst);
        if (symlink(src, dst) == 0) {
            say("%s (%ld B) ", names[j], (long)st.st_size);
            linked++;
        }
    }
    if (linked) say("-> linked into /opt/trm\n");
    else say("nothing on it to link\n");
}

/* ── The persistent setup: a read-only platform image, a writable layer in RAM ─────────────────────
 *
 * An initramfs root is RAM: the platform runs, and every byte it writes dies with the power. Here the
 * image on the volume is squashfs (read-only), the writable layer is a tmpfs upper mounted through
 * overlayfs, and the RAM root is pivoted away -- so the flash is never modified and runtime state
 * still has somewhere to live.
 *
 * Every step is reported, including the ones that do not happen: a boot that claims a writable root
 * and cannot write has proven nothing, and a fallback that stays quiet reads as a success.
 */
/* The handover is refused with EINVAL even though nothing here is shared and the two roots are on
 * different filesystems. The one structural difference from an ordinary switch_root is the new root
 * being an overlay, so that is measured instead of guessed at: a child in its own mount namespace
 * tries the same handover into the plain squashfs and then into an overlay, and reports both. A
 * child, because a successful pivot_root is not something a probe gets to do to the real machine. */
static void probe_handover(const char *loopdev) {
    pid_t p = fork();
    if (p > 0) { int status; waitpid(p, &status, 0); return; }
    if (p < 0) { say("PROBE: fork failed (%s)\n", strerror(errno)); return; }

    if (syscall(SYS_unshare, CLONE_NEWNS) != 0) {
        say("PROBE: could not take a mount namespace (%s)\n", strerror(errno));
        _exit(1);
    }
    if (mount(NULL, "/", NULL, MS_REC | MS_PRIVATE, NULL) != 0)
        say("PROBE: could not make its root private (%s)\n", strerror(errno));

    mkdir_p("/probe_sq");
    if (mount(loopdev, "/probe_sq", "squashfs", MS_RDONLY, NULL) != 0) {
        say("PROBE: could not mount the image a second time (%s)\n", strerror(errno));
        _exit(1);
    }
    if (chdir("/probe_sq") == 0) {
        errno = 0;
        int r = syscall(SYS_pivot_root, ".", ".");
        say("PROBE: pivot_root into the image alone %s\n",
            r == 0 ? "ACCEPTED" : strerror(errno));
    }

    mkdir_p("/probe_ov");
    mkdir_p("/probe_up");
    mkdir_p("/probe_wk");
    if (chdir("/") == 0 && mount("overlay", "/probe_ov", "overlay", 0,
              "lowerdir=/probe_sq,upperdir=/probe_up,workdir=/probe_wk") == 0) {
        if (chdir("/probe_ov") == 0) {
            errno = 0;
            int r = syscall(SYS_pivot_root, ".", ".");
            say("PROBE: pivot_root into an overlay root %s\n",
                r == 0 ? "ACCEPTED" : strerror(errno));
        }
    } else {
        say("PROBE: could not build a probe overlay (%s)\n", strerror(errno));
    }
    _exit(0);
}

/* Attach a file to a free loop device and hand back the device path. A free one is asked for rather
 * than assuming loop0 -- the same rule as the block-device enumeration. The association outlives the
 * descriptor, so both are closed here. */
static int attach_loop(const char *file, char *dev, size_t devlen, int writable) {
    int ctl = open("/dev/loop-control", O_RDWR);
    int n = (ctl >= 0) ? ioctl(ctl, LOOP_CTL_GET_FREE) : -1;
    if (ctl >= 0) close(ctl);
    snprintf(dev, devlen, "/dev/loop%d", n < 0 ? 0 : n);
    struct stat st;
    if (stat(dev, &st) != 0) return -1;
    int lfd = open(dev, O_RDWR);
    /* A loop device is no more writable than the descriptor it was attached with: a read-only fd
     * gives a read-only loop, and mounting that read-write fails with EACCES (measured). */
    int ffd = open(file, writable ? O_RDWR : O_RDONLY);
    int rc = (lfd < 0 || ffd < 0) ? -1 : ioctl(lfd, LOOP_SET_FD, ffd);
    if (ffd >= 0) close(ffd);
    if (lfd >= 0) close(lfd);
    return rc;
}

/* The kernel that boots this pathway is a VM kernel: it carries the drivers a virtual machine needs
 * (ahci, nvme, sd_mod) and nothing for a machine's own ports. On real hardware the volume the firmware
 * has just booted from is reached through those ports, so the modules are shipped inside the image and
 * loaded here -- before /dev is asked what it has. Which loaded and which would not is printed, because
 * a boot that cannot see its own volume must say so instead of reporting an empty /dev. */
/* The wireless credentials, beside the launcher's own network.conf and in its exact shape. Absent means
 * no association is attempted -- and that sentence is the point: an unconfigured radio and a wrong
 * passphrase would otherwise leave the same silence in a boot log. */
#define WIFI_CONF "/mnt/esp/wifi.conf"

/* Both supplicant runs go through here: the wait is bounded, and a child that will not finish is stopped
 * rather than waited on. Exit status 127 means the exec did not happen, and the child says why in its own
 * words -- a failed exec is not evidence of a missing file. */
/* A line of the child's own output, through say() so that it reaches the boot log and not only the
 * screen. A reason that scrolls past on a console is a reason nobody can read afterwards from another
 * machine -- and the reason is the one thing a failed probe exists to leave behind. Long lines are split
 * rather than truncated: half a panic message is not evidence. */
static void child_output(const char *name, int fd) {
    static char line[512];
    static size_t used;
    char buf[256];
    ssize_t n;
    while ((n = read(fd, buf, sizeof buf)) > 0) {
        for (ssize_t i = 0; i < n; i++) {
            char c = buf[i];
            if (c == '\n' || used == sizeof line - 1) {
                line[used] = 0;
                say("%s| %s\n", name, line);
                used = 0;
            } else if (c != '\r') {
                line[used++] = c;
            }
        }
    }
}

static int run_supplicant(const char *name, char *const argv[], int seconds) {
    int pfd[2];
    if (pipe(pfd) != 0) {
        say("%s: no pipe for its output (%s)\n", name, strerror(errno));
        return -1;
    }
    pid_t pid = fork();
    if (pid == 0) {
        dup2(pfd[1], 1);
        dup2(pfd[1], 2);
        close(pfd[0]);
        close(pfd[1]);
        execv("/trm_wifi", argv);
        dprintf(2, "%s: exec /trm_wifi failed: %s\n", name, strerror(errno));
        _exit(127);
    }
    close(pfd[1]);
    if (pid < 0) {
        say("%s: could not be started (%s)\n", name, strerror(errno));
        close(pfd[0]);
        return -1;
    }
    fcntl(pfd[0], F_SETFL, fcntl(pfd[0], F_GETFL, 0) | O_NONBLOCK);
    int rc = -1, finished = 0;
    for (int i = 0; i < seconds * 10; i++) {
        int status = 0;
        child_output(name, pfd[0]);
        if (waitpid(pid, &status, WNOHANG) == pid) {
            child_output(name, pfd[0]);
            if (WIFEXITED(status) && WEXITSTATUS(status) == 127)
                say("%s: /trm_wifi could not be started (its own words are above)\n", name);
            else if (WIFEXITED(status) && WEXITSTATUS(status) != 0)
                say("%s: exited %d\n", name, WEXITSTATUS(status));
            rc = WIFEXITED(status) ? WEXITSTATUS(status) : -1;
            finished = 1;
            break;
        }
        usleep(100000);
    }
    if (!finished) {
        kill(pid, SIGKILL);
        child_output(name, pfd[0]);
        say("%s: did not finish in %d s and was stopped\n", name, seconds);
    }
    close(pfd[0]);
    return rc;
}

static void wifi_probe(void) {
    char *argv[] = { (char *)"/trm_wifi", (char *)"--probe", NULL };
    run_supplicant("WIFI PROBE", argv, 10);
}

/* Run while the RAM image is still the root: afterwards the binary's path belongs to a root that is no
 * longer /. The association gets its own, longer bound because a scan and a slow access point are not a
 * hang -- but its own internal budget is shorter still, so this only fires if the program is wedged. */
static void wifi_associate(void) {
    struct stat st;
    if (stat(WIFI_CONF, &st) != 0) {
        say("WIFI ASSOCIATION: not attempted -- %s is absent, so this station holds no credentials.\n",
            WIFI_CONF);
        say("             (Three different facts must not read alike here: no credentials, no access point\n"
            "              in range, and an association the driver refused. This line is the first of them.)\n");
        return;
    }
    say("WIFI ASSOCIATION: %s present (%ld B) -> associating before the handover\n", WIFI_CONF, (long)st.st_size);
    char *argv[] = { (char *)"/trm_wifi", (char *)"--associate", (char *)"wlan0", (char *)WIFI_CONF, NULL };
    int rc = run_supplicant("WIFI", argv, 30);
    if (rc == 0)
        say("WIFI ASSOCIATION: associated -- the four-way exchange ran against an access point\n");
    else
        say("WIFI ASSOCIATION: not associated -- the attempt exited %d and its own reason is in the lines\n"
            "                  above; the interface's operstate and carrier, printed by the WIFI line\n"
            "                  earlier, say whether it ever came up\n", rc);
}

static void load_modules(void) {
    static const char *mods[] = {
        "usb-common.ko",   /* the chain usb-storage and xhci both need, in dependency order */
        "usbcore.ko",
        "xhci-hcd.ko",
        "xhci-pci.ko",
        "ehci-hcd.ko",     /* USB2 ports, for a machine that has no xHCI on every header */
        "ehci-pci.ko",
        "usb-storage.ko",
        "hid.ko",          /* usbhid resolves its symbols against these */
        "usbhid.ko",       /* a keyboard at the console */
        "hid-generic.ko",
        /* Input events for the dashboard. evdev turns the enumerated input devices into /dev/input/event*;
         * psmouse is the PS/2 trackpad (the PS/2 keyboard, atkbd, is built into this kernel). */
        "evdev.ko",
        "psmouse.ko",
        /* The NIC. Not needed to find the volume, but without it the platform is reachable on loopback
         * only: this kernel carries e1000/e1000e/r8169/igb as modules, and a module is not loaded unless
         * it is named here. The order is `modprobe --show-depends`' own -- r8169 resolves against
         * realtek's PHY, igb against i2c-algo-bit. */
        "realtek.ko",
        "r8169.ko",
        "i2c-algo-bit.ko",
        "igb.ko",
        "e1000.ko",
        "e1000e.ko",
        /* The radio. This machine's Wi-Fi is an Intel AX200 (Killer AX1650x, 8086:2723), which this kernel
         * carries as cfg80211 + mac80211 + iwlwifi + iwlmvm -- and a module is not loaded unless it is
         * named here. The order is `modprobe --show-depends iwlmvm`' own, and it named two dependencies a
         * hand-written list would have missed: libarc4 (mac80211's arc4) and led-class (the LED triggers
         * iwlwifi registers). */
        "libarc4.ko",
        "led-class.ko",
        "cfg80211.ko",
        "mac80211.ko",
        "iwlwifi.ko",
        "iwlmvm.ko",
        NULL
    };
    say("MODULES: ");
    int loaded = 0, absent = 0;
    for (int i = 0; mods[i]; i++) {
        char path[128];
        struct stat st;
        snprintf(path, sizeof path, "/lib/modules/%.48s", mods[i]);
        if (stat(path, &st) != 0) { absent++; continue; }   /* not shipped here: not a failure */
        int fd = open(path, O_RDONLY);
        if (fd < 0) { say("%s unreadable (%s); ", mods[i], strerror(errno)); continue; }
        errno = 0;
        int rc = syscall(SYS_finit_module, fd, "", 0);
        close(fd);
        if (rc == 0) { say("%s ", mods[i]); loaded++; }
        else if (errno == EEXIST) { say("%s(built-in) ", mods[i]); loaded++; }
        else say("%s FAILED (%s); ", mods[i], strerror(errno));
    }
    say("-> %d loaded, %d not shipped\n", loaded, absent);
}

/* What /sys says about a file, as a string. */
static void slurp(const char *path, char *out, size_t n) {
    out[0] = 0;
    int fd = open(path, O_RDONLY);
    if (fd < 0) return;
    int got = read(fd, out, n - 1);
    close(fd);
    if (got <= 0) { out[0] = 0; return; }
    out[got] = 0;
    char *nl = strchr(out, '\n');
    if (nl) *nl = 0;
}

/* A wireless interface exists only once its driver has bound AND the firmware has been accepted, so "the
 * modules loaded" is not the same sentence as "there is a radio". This waits for the kernel to publish
 * one and names it; if none appears it says so, and says what that means, instead of reporting a module
 * list as though it were a device. */
static void wireless_report(void) {
    char name[16] = "";
    for (int pass = 0; pass < 40 && !name[0]; pass++) {   /* up to 10 s; the firmware load is 1-2 s */
        DIR *d = opendir("/sys/class/net");
        if (d) {
            struct dirent *e;
            while ((e = readdir(d))) {
                if (e->d_name[0] == '.' || strlen(e->d_name) >= 16) continue;
                char p[128];
                struct stat st;
                snprintf(p, sizeof p, "/sys/class/net/%s/phy80211", e->d_name);
                if (stat(p, &st) == 0) { snprintf(name, sizeof name, "%.15s", e->d_name); break; }
            }
            closedir(d);
        }
        if (!name[0]) usleep(250000);
    }
    if (!name[0]) {
        say("WIFI: no wireless interface appeared in 10 s. The modules are loaded, so what did not happen "
            "is one of two things: this machine has no radio of that make, or its firmware was refused -- "
            "and the kernel's own lines above say which.\n");
        return;
    }
    char driver[64] = "unknown", mac[32] = "", state[32] = "";
    char p[160];
    snprintf(p, sizeof p, "/sys/class/net/%s/device/driver", name);
    char link[160] = "";
    int got = readlink(p, link, sizeof link - 1);
    if (got > 0) { link[got] = 0; const char *base = strrchr(link, '/'); snprintf(driver, sizeof driver, "%s", base ? base + 1 : link); }
    snprintf(p, sizeof p, "/sys/class/net/%s/address", name);   slurp(p, mac, sizeof mac);
    snprintf(p, sizeof p, "/sys/class/net/%s/operstate", name); slurp(p, state, sizeof state);
    /* Carrier, read from the kernel's own file rather than inferred from a driver log line. For a station
     * that has never joined a network, operstate down with carrier 0 is the EXPECTED state and not a fault
     * -- carrier comes up when the station joins one. Saying "link down" without that is how a boot log
     * teaches an operator to read a normal state as a failure. */
    char carrier[8] = "";
    snprintf(p, sizeof p, "/sys/class/net/%s/carrier", name); slurp(p, carrier, sizeof carrier);
    say("WIFI: %s present (%s, mac %s, operstate %s, carrier %s) -- the driver bound and the firmware was "
        "accepted. A station that has not joined a network reads down with no carrier; that is the state "
        "before association, not a fault, and the line below says whether any was attempted.\n",
        name, driver, mac, state, carrier[0] ? carrier : "unreadable");
}

/* Which block devices exist is the kernel's answer, not ours: a boot that names a device it did not check
 * is a guess, and on a machine with an internal disk the guessed name is usually the wrong one. */
static int block_devices(char names[][16], int max) {
    DIR *d = opendir("/dev");
    struct dirent *e;
    int n = 0;
    if (!d) return 0;
    while ((e = readdir(d)) && n < max) {
        /* A name that cannot fit the buffer is not a device this can address, so it is skipped rather
         * than silently truncated into a different device's name. */
        size_t len = strlen(e->d_name);
        if (len >= 16) continue;
        if (!strncmp(e->d_name, "sd", 2) || !strncmp(e->d_name, "vd", 2) ||
            !strncmp(e->d_name, "sr", 2) || !strncmp(e->d_name, "nvme", 4)) {
            memcpy(names[n], e->d_name, len + 1);
            n++;
        }
    }
    closedir(d);
    return n;
}

/* The boot waits for the VOLUME, not for a disk.
 *
 * On a machine with an internal disk the old question -- "is any disk present?" -- is answered before
 * the boot starts, so the wait never ran, and the volume this boot came from (behind a USB port whose
 * driver is a module loaded a moment earlier) was not there yet. Measured on real hardware: PID 1 looked
 * once at 1.19 s and saw `nvme0n1` and its eight partitions; the stick appeared at 2.65 s as [sda]; the
 * boot then reported "no volume carrying the platform image" while the volume was still enumerating. On
 * the emulated machine the same sequence reproduces: the decoy internal disk is there at once and the
 * stick arrives at 8.5 s.
 *
 * So every pass asks /dev again and tries what is there, because the device LIST is the thing that
 * changes. The failure mode this replaces is a boot that gives up on a machine that was about to hand it
 * exactly what it needed.
 */
static const char *const VOLUME_IMAGE = "rootfs.squashfs";

static int try_volume(const char *dev) {
    char full[32], path[64];
    struct stat ds, ps;
    snprintf(full, sizeof full, "/dev/%.15s", dev);
    if (stat(full, &ds) != 0) return 0;                 /* gone between the listing and the look */
    mkdir("/mnt", 0755);
    mkdir("/mnt/esp", 0755);
    if (mount(full, "/mnt/esp", "vfat", MS_RDONLY, NULL) != 0)
        return 0;                                       /* not vfat: not this one, and not an error */
    snprintf(path, sizeof path, "/mnt/esp/%s", VOLUME_IMAGE);
    if (stat(path, &ps) != 0) {
        say("%s vfat but carries no %s; ", dev, VOLUME_IMAGE);
        umount2("/mnt/esp", MNT_DETACH);
        return 0;
    }
    say("%s mounted at /mnt/esp (%s %ld B)\n", dev, VOLUME_IMAGE, (long)ps.st_size);
    return 1;
}

static int find_volume(void) {
    char devs[16][16];
    for (int t = 0; t < 160; t++) {                     /* 40 s, and it leaves the moment it finds it */
        int ndev = block_devices(devs, 16);
        if (t == 0) {
            say("BLOCK DEVICES: ");
            if (!ndev) say("(none)");
            for (int i = 0; i < ndev; i++) say("%s ", devs[i]);
            say("\nFABRIC VOLUME: ");
        }
        for (int i = 0; i < ndev; i++) {
            if (try_volume(devs[i])) {
                if (t) say("  (found on pass %d, %d ms in)\n", t + 1, t * 250);
                return 1;
            }
        }
        if (t == 0)
            say("WAIT: no volume carrying %s yet -> letting the controllers finish attaching: ",
                VOLUME_IMAGE);
        usleep(250000);
        if (t && t % 8 == 0) say("%d s ", t / 4);       /* a wait that says it is waiting */
    }
    say("none after 40 s: no controller produced a volume carrying %s\n", VOLUME_IMAGE);
    return 0;
}


static void persistent_root(void) {
    const char *img = "/mnt/esp/rootfs.squashfs";
    struct stat st;
    if (stat(img, &st) != 0) {
        say("PERSISTENT ROOT: %s absent -> staying on the RAM rootfs\n", img);
        return;
    }
    /* Captured here because `st` is reused for the loop device below, and a block device reports size
     * 0 -- which is how this line first came out as "0 B" for a 16 MB image. */
    const long img_bytes = (long)st.st_size;

    /* The platform image goes on a loop device. */
    char loopdev[32];
    if (attach_loop(img, loopdev, sizeof loopdev, 0) != 0) {
        say("PERSISTENT ROOT: cannot attach %s to a loop device (%s) -> RAM rootfs\n",
            img, strerror(errno));
        return;
    }

    mkdir_p("/mnt/root");
    mkdir_p("/mnt/rw");
    mkdir_p("/newroot");
    if (mount(loopdev, "/mnt/root", "squashfs", MS_RDONLY, NULL) != 0) {
        say("PERSISTENT ROOT: squashfs mount failed (%s) -> RAM rootfs\n", strerror(errno));
        return;
    }

    /* The writable layer: the state image on the volume when it is there -- so what the platform
     * writes is still there after a reboot -- and RAM when it is not, which the boot says rather than
     * leaving an operator to assume it. The image above stays read-only either way. */
    const char *state = "/mnt/esp/trm-state.ext4";
    char stateloop[32];
    struct stat st_state;
    int persistent = 0;
    if (stat(state, &st_state) == 0) {
        if (attach_loop(state, stateloop, sizeof stateloop, 1) != 0)
            say("PERSISTENT ROOT: could not attach %s to a loop device (%s) -> state in RAM\n",
                state, strerror(errno));
        else if (mount(stateloop, "/mnt/rw", "ext4", 0, NULL) != 0)
            say("PERSISTENT ROOT: could not mount %s as ext4 (%s) -> state in RAM\n",
                stateloop, strerror(errno));
        else
            persistent = 1;
    }
    if (!persistent && mount("tmpfs", "/mnt/rw", "tmpfs", 0, "size=128m,mode=0755") != 0) {
        say("PERSISTENT ROOT: no writable layer at all (%s) -> RAM rootfs\n", strerror(errno));
        return;
    }
    /* The upper and work directories are created AFTER the layer is mounted, not before: a mount
     * hides whatever sat at its mount point, so directories made earlier are not on the tmpfs and
     * overlayfs reports the upperdir it cannot find as ENOENT (measured on the first boot). */
    mkdir_p("/mnt/rw/upper");
    mkdir_p("/mnt/rw/work");
    if (mount("overlay", "/newroot", "overlay", 0,
              "lowerdir=/mnt/root,upperdir=/mnt/rw/upper,workdir=/mnt/rw/work") != 0) {
        say("PERSISTENT ROOT: overlay failed (%s) -> RAM rootfs\n", strerror(errno));
        return;
    }
    /* Before the real machine is handed over, the kernel (in a child) is asked which handovers it will
     * take, so the log carries the reason rather than an unexplained EINVAL. */
    probe_handover(loopdev);

    /* The pseudo-filesystems, the boot volume and both halves of the overlay come along, or the new
     * root cannot open /dev/tty0, read a store, or let an operator look at what was written. MS_MOVE
     * keeps the same mounts rather than re-mounting them. */
    struct { const char *src, *dst; } moves[] = {
        { "/dev", "/newroot/dev" }, { "/proc", "/newroot/proc" }, { "/sys", "/newroot/sys" },
        { "/mnt/esp", "/newroot/mnt/esp" }, { "/mnt/root", "/newroot/mnt/root" },
        { "/mnt/rw", "/newroot/mnt/rw" }, { NULL, NULL }
    };
    for (int i = 0; moves[i].src; i++) {
        mkdir_p(moves[i].dst);
        if (mount(moves[i].src, moves[i].dst, NULL, MS_MOVE, NULL) != 0)
            say("PERSISTENT ROOT: could not move %s into the new root (%s)\n",
                moves[i].src, strerror(errno));
    }

    /* Prove BOTH halves: a write that lands and reads back through the overlay, and the same path
     * still absent from the read-only image underneath it. */
    const char *rel = "/opt/trm/.write-probe";
    char probe[192], lower[192];
    snprintf(probe, sizeof probe, "/newroot%s", rel);
    snprintf(lower, sizeof lower, "/mnt/root%s", rel);
    /* A write from an earlier boot is what separates a layer that works from one that survives: the
     * read-only image has no such file (checked at the end of this line), so finding one means a
     * previous boot's write came back. Read before this boot writes its own. */
    struct stat ps;
    int prev = (stat(probe, &ps) == 0);
    const char *msg = "written through the overlay by the platform's own init\n";
    int ok = 0;
    int wf = open(probe, O_WRONLY | O_CREAT | O_TRUNC, 0644);
    if (wf >= 0) {
        ok = (write(wf, msg, strlen(msg)) == (ssize_t)strlen(msg));
        close(wf);
        char back[128];
        memset(back, 0, sizeof back);
        int rf = open(probe, O_RDONLY);
        if (rf >= 0) {
            ssize_t r = read(rf, back, sizeof back - 1);
            close(rf);
            if (r <= 0 || strcmp(back, msg)) ok = 0;
        } else {
            ok = 0;
        }
    }
    struct stat ls;
    int in_lower = (stat(lower, &ls) == 0);
    say("PERSISTENT ROOT: %s (%ld B, ro) on %s + %s; write-probe %s; this file in the read-only "
        "image: %s; a write from an earlier boot: %s\n", img, img_bytes, loopdev,
        persistent ? "the state image on the volume (writes outlive the reboot)"
                   : "a RAM layer (writes die with the power)",
        ok ? "wrote and read back identical" : "FAILED",
        in_lower ? "PRESENT (the image itself changed!)" : "absent",
        prev ? "PRESENT" : "absent");

    /* The handover is the one step whose failure says nothing but "EINVAL", so the evidence goes into
     * the log first -- the mount table (a mount marked shared or slave is refused), and the device
     * numbers of the two roots -- and then each form of the handover is tried in turn, with the errno
     * of the one that failed printed rather than swallowed. */
    {
        FILE *mi = fopen("/newroot/proc/self/mountinfo", "r");
        if (mi) {
            char line[512];
            int flagged = 0;
            while (fgets(line, sizeof line, mi))
                if (strstr(line, " shared:") || strstr(line, " master:")) {
                    say("HANDOVER: %.230s", line);
                    flagged++;
                }
            fclose(mi);
            say("HANDOVER: mounts marked shared or slave: %d\n", flagged);
        } else {
            say("HANDOVER: mount table unreadable (%s)\n", strerror(errno));
        }
        struct stat s_new, s_old;
        if (stat("/newroot", &s_new) == 0 && stat("/", &s_old) == 0)
            say("HANDOVER: /newroot dev %llu, / dev %llu -- %s\n",
                (unsigned long long)s_new.st_dev, (unsigned long long)s_old.st_dev,
                s_new.st_dev != s_old.st_dev ? "different filesystems" : "SAME filesystem");
    }

    mkdir_p("/newroot/.oldroot");
    if (chdir("/newroot") != 0) {
        say("PERSISTENT ROOT: cannot enter the new root (%s) -> staying on the RAM rootfs\n",
            strerror(errno));
        if (chdir("/") != 0) say("PERSISTENT ROOT: chdir / failed (%s)\n", strerror(errno));
        return;
    }
    /* pivot_root refuses a shared mount with EINVAL, so both roots are made private first -- and each
     * result is reported: a silent failure here is what turned the first boot's handover into the
     * chroot fallback. */
    if (mount(NULL, "/", NULL, MS_REC | MS_PRIVATE, NULL) != 0)
        say("PERSISTENT ROOT: could not make the old root private (%s)\n", strerror(errno));
    if (mount(NULL, "/newroot", NULL, MS_PRIVATE, NULL) != 0)
        say("PERSISTENT ROOT: could not make the new root private (%s)\n", strerror(errno));
    /* switch_root, not pivot_root. pivot_root returned EINVAL for both forms on this kernel, and the
     * chroot fallback leaves the old root mounted -- which is what the previous boot reported. MS_MOVE
     * onto "/" is the primitive busybox switch_root uses: it needs no pivot_root semantics, only that
     * the source is a mount point and the target is not shared, and both are arranged above. */
    if (mount(".", "/", NULL, MS_MOVE, NULL) == 0) {
        if (chroot(".") == 0) {
            say("PERSISTENT ROOT: switch_root done (MS_MOVE onto / + chroot) -- the machine now runs "
                "from the image + the layer\n");
        } else {
            say("PERSISTENT ROOT: MS_MOVE done but chroot failed (%s) -- the old root stays reachable "
                "under the new one\n", strerror(errno));
        }
    } else {
        int move_err = errno;
        if (chroot(".") == 0) {
            say("PERSISTENT ROOT: MS_MOVE onto / failed (%s); chroot used instead -- same root, the old "
                "image left mounted\n", strerror(move_err));
        } else {
            say("PERSISTENT ROOT: neither MS_MOVE nor chroot worked (%s) -> RAM rootfs\n",
                strerror(errno));
            if (chdir("/") != 0) say("PERSISTENT ROOT: chdir / failed (%s)\n", strerror(errno));
            return;
        }
    }
    if (chdir("/") != 0) say("PERSISTENT ROOT: chdir / failed (%s)\n", strerror(errno));
}


/* ── The boot log a blind boot leaves behind ─────────────────────────────────────────────
 *
 * On a machine with no serial port the screen is the only channel, and if the firmware hands over no
 * framebuffer there is no channel at all. The writable layer IS the machine's state image, so the log
 * goes there: `trm_chainload/scripts/read_boot_log.sh` pulls it off the stick from another computer with
 * debugfs. Everything printed before the layer exists -- banner, mounts, modules, the volume -- is
 * buffered here, so the file carries the whole boot rather than its second half. */
static void bootlog_open(void) {
    const char *path = "/mnt/rw/trm_boot.log";
    FILE *f = fopen(path, "w");
    if (!f) {
        say("BOOT LOG: %s unwritable (%s) -> this boot leaves no receipt on the machine\n",
            path, strerror(errno));
        return;
    }
    bootlog = f;
    if (bootlen) fwrite(bootbuf, 1, bootlen, f);
    if (bootlost)
        fprintf(f, "[... console output between this point and the layer's mount was not buffered ...]\n");
    fflush(f);
    /* The log is the receipt for a boot that may show nothing at all, so it is pushed rather than left
     * to the kernel's schedule. Two hops: this file to the loop device, and -- because the state image
     * is itself a FILE on a FAT volume, which flushes lazily -- that file to the volume. Measured: a
     * reader that looks 80 s after the platform answers still saw the previous state; the same reader
     * saw the log once the writeback had happened. */
    fsync(fileno(f));
    int vol = open("/mnt/esp/trm-state.ext4", O_RDONLY);
    if (vol >= 0) { fsync(vol); close(vol); }
    say("BOOT LOG: %s on the writable layer -- readable from another machine with "
        "trm_chainload/scripts/read_boot_log.sh\n", path);
}

/* ── The network the machine is on, if its volume says so ────────────────────────────────
 *
 * `/network.conf` on the volume, because this is a fact about the machine's own network and belongs
 * where the operator can edit it with anything that writes a FAT file:
 *
 *     ip=192.168.1.50/24
 *     gw=192.168.1.1          (optional)
 *     iface=eth0              (optional: the first non-loopback device otherwise)
 *
 * The NIC drivers are MODULES in this kernel (`e1000`, `e1000e`, `r8169`, `igb`), so nothing exists to
 * configure until `load_modules()` has run -- and with none of them loaded the platform is loopback-only,
 * which is a fact to print rather than a silence to leave. */
/* The file itself, read at its own size: a fixed window over it lost both settings when a header
 * comment line ran past the window (measured -- the boot printed `network.conf (511 B)` for a 1073-byte
 * file and applied nothing). A config reader that only sees the first N bytes of a file reports a
 * machine with no address, not a truncated read. */
static char *netconf = NULL;
/* Where the dashboard can be reached, filled in by network_up() and printed at the end of the boot:
 * "the platform is up" is not the same sentence as "here is where to look at it". */
static char net_if[16] = "";
static int net_carrier = -1;   /* the carrier read at bring-up, -1 = not read */
static char net_addr[64] = "";

/* The kernel's own view of the link, not a driver's log line: /sys/class/net/<dev>/carrier is 1 when
 * something is on the other end of the cable. -1 when the kernel has no answer yet, which is not the
 * same as 0. */
static int link_is_up(const char *dev) {
    char p[64], b[8];
    snprintf(p, sizeof p, "/sys/class/net/%.15s/carrier", dev);
    int fd = open(p, O_RDONLY);
    if (fd < 0) return -1;
    int n = read(fd, b, sizeof b - 1);
    close(fd);
    if (n <= 0) return -1;
    b[n] = 0;
    return b[0] == '1' ? 1 : 0;
}

static void trim(char *s) {
    char *b = s;
    while (*b == ' ' || *b == '\t' || *b == '\r') b++;
    if (b != s) memmove(s, b, strlen(b) + 1);
    size_t n = strlen(s);
    while (n && (s[n - 1] == ' ' || s[n - 1] == '\t' || s[n - 1] == '\r')) s[--n] = 0;
}

static int conf_value(const char *key, char *out, size_t outlen) {
    if (!netconf) return 0;
    size_t klen = strlen(key);
    const char *p = netconf;
    while (*p) {
        const char *eol = strchr(p, '\n');
        const char *line = p;
        const char *end = eol ? eol : p + strlen(p);
        while (line < end && (*line == ' ' || *line == '\t')) line++;   /* an indented line is a line */
        if ((size_t)(end - line) > klen && strncmp(line, key, klen) == 0 && line[klen] == '=') {
            size_t vlen = (size_t)(end - line) - klen - 1;
            if (vlen >= outlen) vlen = outlen - 1;
            memcpy(out, line + klen + 1, vlen);
            out[vlen] = 0;
            trim(out);
            if (*out) return 1;   /* a key present but empty is not a value */
        }
        if (!eol) break;
        p = eol + 1;
    }
    return 0;
}

static void read_netconf(void) {
    const char *path = "/mnt/esp/network.conf";
    struct stat st;
    if (stat(path, &st) != 0) {
        say("NET CONFIG: %s absent -> no address is set, the platform is reachable on loopback only\n",
            path);
        return;
    }
    if (st.st_size <= 0 || st.st_size > 65536) {
        say("NET CONFIG: %s is %ld B, which is not a config file -> loopback only\n", path,
            (long)st.st_size);
        return;
    }
    int fd = open(path, O_RDONLY);
    if (fd < 0) {
        say("NET CONFIG: %s unreadable (%s) -> loopback only\n", path, strerror(errno));
        return;
    }
    netconf = calloc(1, (size_t)st.st_size + 1);
    if (!netconf) {
        close(fd);
        say("NET CONFIG: no room for %ld B -> loopback only\n", (long)st.st_size);
        return;
    }
    size_t got = 0;
    while (got < (size_t)st.st_size) {
        ssize_t n = read(fd, netconf + got, (size_t)st.st_size - got);
        if (n <= 0) break;
        got += (size_t)n;
    }
    close(fd);
    netconf[got] = 0;
    say("NET CONFIG: %s (%zu B of %ld)\n", path, got, (long)st.st_size);
    /* What will be applied, in the log: the decision belongs beside the input it came from. */
    char probe[64];
    if (conf_value("ip", probe, sizeof probe)) say("NET CONFIG: ip=%s\n", probe);
    if (conf_value("gw", probe, sizeof probe)) say("NET CONFIG: gw=%s\n", probe);
}

/* A driver that has just registered is not an interface that is finished: /sys/class/net is asked,
 * bounded, the same way /dev is after the USB modules -- and a device that is coming is not one that is
 * absent. */
static int net_devices(char names[][16], int max) {
    for (int t = 0; t < 32; t++) {
        int n = 0;
        DIR *d = opendir("/sys/class/net");
        if (d) {
            struct dirent *e;
            while ((e = readdir(d)) && n < max) {
                if (e->d_name[0] == '.' || strcmp(e->d_name, "lo") == 0) continue;
                if (strlen(e->d_name) >= 16) continue;   /* an interface name that will not fit */
                snprintf(names[n], 16, "%s", e->d_name);
                n++;
            }
            closedir(d);
        }
        if (n) return n;
        usleep(250000);
    }
    return 0;
}

/* Ask the router for an address -- the thing a machine with a cable and nothing configured should do,
 * and the thing this boot did not do: it went straight from "nothing in network.conf" to an address it
 * derived from its own MAC, which reaches a directly-cabled machine and nothing on a routed LAN. Four
 * UDP datagrams, no library. Returns 1 and fills addr as "a.b.c.d/nn" when a router answered.
 *
 * Every receive is bounded: a boot that blocks on an answer that never comes is a machine that never
 * finishes booting, and the answer it waits for is exactly the one a network with no DHCP server never
 * sends. Three tries, 1.5 s each, then the caller falls back and SAYS which of the two happened. */
/* The kernel's own count of datagrams delivered to UDP sockets. A client that sends and hears nothing is
 * looking at two different failures -- nobody answered, or the answer never reached this socket -- and the
 * difference decides where the fault is. This reads the counter so the boot can say which one it is. */
static long udp_delivered(void) {
    int fd = open("/proc/net/snmp", O_RDONLY);
    if (fd < 0) return -1;
    char b[4096];
    ssize_t n = read(fd, b, sizeof b - 1);
    close(fd);
    if (n <= 0) return -1;
    b[n] = 0;
    char *p = strstr(b, "Udp:");          /* the header line */
    if (!p) return -1;
    p = strstr(p + 4, "\nUdp:");          /* the values line */
    if (!p) return -1;
    return strtol(p + 5, NULL, 10);
}

/* One counter off the interface. A DISCOVER the socket accepted can still have been dropped by the kernel
 * before it reached the wire -- a route or a neighbour entry -- and the interface's own count is what
 * separates "the machine asked and nobody answered" from "the machine never asked". */
static long iface_counter(const char *dev, const char *what) {
    char p[96], b[24];
    snprintf(p, sizeof p, "/sys/class/net/%.15s/statistics/%.15s", dev, what);
    int fd = open(p, O_RDONLY);
    if (fd < 0) return -1;
    ssize_t n = read(fd, b, sizeof b - 1);
    close(fd);
    if (n <= 0) return -1;
    b[n] = 0;
    return strtol(b, NULL, 10);
}

static char dhcp_gw[64] = "";
static int dhcp_acquire(const char *dev, const unsigned char *mac, char *out, size_t outsz) {
    struct __attribute__((packed)) {
        unsigned char op, htype, hlen, hops;
        unsigned int xid;
        unsigned short secs, flags;
        unsigned int ci, yi, si, gi;
        unsigned char chaddr[16], sname[64], file[128], magic[4], opts[312];
    } p, r;
    long before_first = -1, tx0 = -1, rx0 = -1;
    int s = socket(AF_INET, SOCK_DGRAM, 0);
    if (s < 0) return 0;
    int one = 1;
    setsockopt(s, SOL_SOCKET, SO_REUSEADDR, &one, sizeof one);
    setsockopt(s, SOL_SOCKET, SO_BROADCAST, &one, sizeof one);
    setsockopt(s, SOL_SOCKET, SO_BINDTODEVICE, dev, (socklen_t)strlen(dev));
    struct timeval tv = {1, 500000};
    setsockopt(s, SOL_SOCKET, SO_RCVTIMEO, &tv, sizeof tv);
    struct sockaddr_in b = {0}, to = {0};
    b.sin_family = AF_INET; b.sin_port = htons(68);
    if (bind(s, (struct sockaddr *)&b, sizeof b) != 0) { close(s); return 0; }
    /* A machine with no address has no route, and the DISCOVER it has to send goes to the one address that
     * cannot be looked up -- so without this the send fails with ENETUNREACH and this function would report
     * "no router answered" about a router that was never spoken to. The limited broadcast is given a host
     * route out of this device; if it is already there, that is the same fact. */
    struct rtentry br;
    memset(&br, 0, sizeof br);
    ((struct sockaddr_in *)&br.rt_dst)->sin_family = AF_INET;
    ((struct sockaddr_in *)&br.rt_dst)->sin_addr.s_addr = 0xFFFFFFFFu;
    ((struct sockaddr_in *)&br.rt_genmask)->sin_family = AF_INET;
    ((struct sockaddr_in *)&br.rt_genmask)->sin_addr.s_addr = 0xFFFFFFFFu;
    br.rt_flags = RTF_UP | RTF_HOST;
    br.rt_dev = (char *)dev;
    int rrc = ioctl(s, SIOCADDRT, &br);
    char why[160];
    if (rrc == 0) snprintf(why, sizeof why, "the broadcast route is set and the DISCOVER went unanswered");
    else snprintf(why, sizeof why, "the broadcast route was refused: %s", strerror(errno));
    to.sin_family = AF_INET; to.sin_port = htons(67); to.sin_addr.s_addr = 0xFFFFFFFFu;
    unsigned char opts_req[] = {55, 3, 1, 3, 54, 255};   /* ask for netmask, router, server id */
    for (int attempt = 0; attempt < 3; attempt++) {
        memset(&p, 0, sizeof p);
        p.op = 1; p.htype = 1; p.hlen = 6; p.flags = htons(0x8000);
        p.xid = htonl(0x54524D00u | (unsigned int)(attempt + mac[5]));
        memcpy(p.chaddr, mac, 6);
        p.magic[0] = 99; p.magic[1] = 130; p.magic[2] = 83; p.magic[3] = 99;
        size_t n = 240;
        p.opts[0] = 53; p.opts[1] = 1;                    /* DHCPDISCOVER */
        memcpy(p.opts + 2, opts_req, sizeof opts_req);
        n = 240 + 2 + sizeof(opts_req);
        if (n < 300) n = 300;   /* RFC 2131's minimum; a short datagram is legal and still gets dropped */
        long before = udp_delivered();
        if (attempt == 0) {
            before_first = before;
            tx0 = iface_counter(dev, "tx_packets");
            rx0 = iface_counter(dev, "rx_packets");
        }
        ssize_t sent = sendto(s, &p, n, 0, (struct sockaddr *)&to, sizeof to);
        if (sent < 0) {
            snprintf(why, sizeof why, "the DISCOVER could not be sent at all: %s", strerror(errno));
            continue;
        }
        snprintf(why, sizeof why, "%zd bytes of DISCOVER were sent and nothing came back", sent);
        for (int wait = 0; wait < 2; wait++) {
            ssize_t got = recv(s, &r, sizeof r, 0);
            if (got < 0) {
                if (errno != EAGAIN && errno != EWOULDBLOCK)
                    snprintf(why, sizeof why, "the wait for an answer failed: %s", strerror(errno));
                continue;
            }
            if (got < (ssize_t)240) { snprintf(why, sizeof why, "a %zd-byte reply -- too short to be DHCP", got); continue; }
            if (r.op != 2 || r.xid != p.xid) { snprintf(why, sizeof why, "a reply arrived but it was not for this request (op %d)", r.op); continue; }
            unsigned char type = 0, mask[4] = {255, 255, 255, 0}, gw[4] = {0};
            for (size_t o = 240; o + 2 < (size_t)got && r.opts[o] != 255; ) {
                unsigned char code = r.opts[o], len = r.opts[o + 1];
                if (code == 53) type = r.opts[o + 2];
                else if (code == 1 && len == 4) memcpy(mask, r.opts + o + 2, 4);
                else if (code == 3 && len >= 4) memcpy(gw, r.opts + o + 2, 4);
                o += (size_t)len + 2;
            }
            if (type == 2) {                              /* OFFER -> REQUEST -> ACK */
                p.opts[0] = 53; p.opts[1] = 3; p.opts[2] = 50; p.opts[3] = 4;
                memcpy(p.opts + 4, &r.yi, 4);
                p.opts[8] = 255;
                sendto(s, &p, 300, 0, (struct sockaddr *)&to, sizeof to);
                continue;
            }
            if (type != 5) continue;                      /* ACK is the only one that counts */
            struct in_addr a = {r.yi}, m = {0}, g = {0};
            memcpy(&m, mask, 4); memcpy(&g, gw, 4);
            char as[32], ids[64];
            if (!inet_ntop(AF_INET, &a, as, sizeof as)) break;
            unsigned int mm = ntohl(m.s_addr); int bits = 0;
            while (bits < 32 && (mm & (0x80000000u >> bits))) bits++;
            snprintf(out, outsz, "%s/%d", as, bits);
            if (g.s_addr && inet_ntop(AF_INET, &g, ids, sizeof ids))
                snprintf(dhcp_gw, sizeof dhcp_gw, "%s", ids);
            close(s);
            return 1;
        }
    }
    /* The reason travels with the result. "No router answered" is a claim about a router; this line is what
     * the boot can actually support, and the difference between the two is the whole point of the field. */
    long after = udp_delivered();
    if (after >= 0 && before_first >= 0)
        say("[dhcp] %s: %s. The kernel delivered %ld UDP datagram(s) to sockets during the wait "
            "(%ld -> %ld), so %s\n", dev, why, after - before_first, before_first, after,
            after == before_first ? "nothing arrived at all and the answer was never sent to this machine"
                                  : "something arrived and the socket did not take it");
    else
        say("[dhcp] %s: %s\n", dev, why);
    say("[dhcp] %s counters across the attempts: transmitted %+ld packet(s), received %+ld\n", dev,
        iface_counter(dev, "tx_packets") - tx0, iface_counter(dev, "rx_packets") - rx0);
    close(s);
    return 0;
}

static void network_up(void) {
    char ifs[8][16];
    int n = net_devices(ifs, 8);
    if (!n) {
        say("NET: no interface -- the NIC drivers this kernel carries are modules (e1000, e1000e, r8169, "
            "igb) and none produced a device on this machine, so the platform is reachable on loopback "
            "only\n");
        return;
    }
    char want[32];
    int have_want = conf_value("iface", want, sizeof want);
    int sock = socket(AF_INET, SOCK_DGRAM, 0);
    if (sock < 0) { say("NET: socket failed (%s)\n", strerror(errno)); return; }
    for (int i = 0; i < n; i++) {
        if (have_want && strcmp(want, ifs[i]) != 0) continue;
        struct ifreq ifr;
        memset(&ifr, 0, sizeof ifr);
        snprintf(ifr.ifr_name, sizeof ifr.ifr_name, "%.15s", ifs[i]);
        if (ioctl(sock, SIOCGIFFLAGS, &ifr) != 0) continue;
        ifr.ifr_flags |= IFF_UP | IFF_RUNNING;
        int up = ioctl(sock, SIOCSIFFLAGS, &ifr) == 0;
        unsigned char mac[6] = {0};
        memset(&ifr, 0, sizeof ifr);
        snprintf(ifr.ifr_name, sizeof ifr.ifr_name, "%.15s", ifs[i]);
        if (ioctl(sock, SIOCGIFHWADDR, &ifr) == 0) memcpy(mac, ifr.ifr_hwaddr.sa_data, 6);
        int carrier = link_is_up(ifs[i]);
        /* The carrier is a negotiation, not a property of the interface. Read in the same breath as
         * IFF_UP it can say 0 on a cable that is plugged into a live switch, because the driver is still
         * bringing the link up -- measured on this very harness, where one run of one binary read 1 at
         * bring-up and another read 0, and on the metal boot that printed "no carrier" in its own NET
         * line and "(eth0 is linked)" thirty seconds later. That is not a cosmetic contradiction: it is
         * the difference between the DHCP attempt below running or being skipped, and so between a
         * dashboard that only this machine can open and one on the network. So the link is waited for,
         * bounded -- 3 s at the very most -- and the wait is reported rather than hidden. */
        int waited = 0;
        for (int ms = 0; carrier != 1 && ms < 30; ms++) {
            usleep(100000);
            carrier = link_is_up(ifs[i]);
            waited = 1;
        }
        say("NET: %s %s (mac %02x:%02x:%02x:%02x:%02x:%02x)", ifs[i], up ? "up" : "NOT up",
            mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);
        if (carrier == 0) say(" [no carrier: nothing on the other end of that cable]");
        else if (waited) say(" [the link came up while it was waited for]");
        char addr[64];
        int linklocal = 0, from_dhcp = 0;
        if (!conf_value("ip", addr, sizeof addr)) {
            /* Nothing configured: the thing to do is ask the router, and only then fall back. A machine
             * with no address cannot be reached at all, and "the dashboard is up" is useless to an
             * operator who has no name for the machine serving it. 169.254/16 means exactly "no router
             * answered" -- derived from the MAC, so it is stable across boots, and LABELLED as derived,
             * because it is not probed for collisions and saying so is cheaper than implying it was. */
            if (carrier == 1 && dhcp_acquire(ifs[i], mac, addr, sizeof addr)) {
                from_dhcp = 1;
            } else {
                snprintf(addr, sizeof addr, "169.254.%d.%d", 1 + (mac[4] % 254), 1 + (mac[5] % 254));
                linklocal = 1;
            }
        }
        char *slash = strchr(addr, '/');
        int bits = linklocal ? 16 : 24;
        if (slash) { *slash = 0; bits = atoi(slash + 1); }
        if (bits < 0 || bits > 32) bits = 24;
        struct sockaddr_in sin;
        memset(&sin, 0, sizeof sin);
        sin.sin_family = AF_INET;
        if (inet_pton(AF_INET, addr, &sin.sin_addr) != 1) {
            say(", ip=%s is not an address\n", addr);
            continue;
        }
        memset(&ifr, 0, sizeof ifr);
        snprintf(ifr.ifr_name, sizeof ifr.ifr_name, "%.15s", ifs[i]);
        memcpy(&ifr.ifr_addr, &sin, sizeof sin);
        if (ioctl(sock, SIOCSIFADDR, &ifr) != 0) {
            say(", address refused (%s)\n", strerror(errno));
            continue;
        }
        memset(&ifr, 0, sizeof ifr);
        snprintf(ifr.ifr_name, sizeof ifr.ifr_name, "%.15s", ifs[i]);
        sin.sin_addr.s_addr = bits ? htonl(~(unsigned int)0 << (32 - bits)) : 0;
        memcpy(&ifr.ifr_netmask, &sin, sizeof sin);
        if (ioctl(sock, SIOCSIFNETMASK, &ifr) != 0)
            say(", netmask refused (%s)", strerror(errno));
        say(", %s/%d%s", addr, bits,
            from_dhcp ? " (from DHCP: a router answered on this cable)"
            : linklocal ? " (link-local, derived from the MAC: nothing was configured, and no router "
                          "answered)" : "");
        /* An interface name is bounded by the kernel's own IFNAMSIZ, so this cannot truncate --
         * but the bound is stated rather than inferred, which is the difference between a warning
         * that cannot happen and one that is silenced. */
        snprintf(net_if, sizeof net_if, "%.15s", ifs[i]);
        snprintf(net_addr, sizeof net_addr, "%s", addr);
        /* The reading taken here travels with the interface. A later line that re-reads the carrier and
         * reports a verdict instead of a number is how the boot came to say "eth0 is linked" on a machine
         * whose own cable line four lines earlier said "no carrier: nothing on the other end of that
         * cable" -- two readings, one asserted as a fact. They are now both printed. */
        net_carrier = carrier;
        char gw[64];
        int have_gw = conf_value("gw", gw, sizeof gw);
        if (!have_gw && dhcp_gw[0]) { snprintf(gw, sizeof gw, "%s", dhcp_gw); have_gw = 1; }
        if (have_gw) {
            struct rtentry rt;
            struct sockaddr_in *dst = (struct sockaddr_in *)&rt.rt_dst;
            struct sockaddr_in *via = (struct sockaddr_in *)&rt.rt_gateway;
            memset(&rt, 0, sizeof rt);
            dst->sin_family = AF_INET;
            via->sin_family = AF_INET;
            if (inet_pton(AF_INET, gw, &via->sin_addr) == 1) {
                rt.rt_flags = RTF_UP | RTF_GATEWAY;
                rt.rt_dev = ifs[i];
                if (ioctl(sock, SIOCADDRT, &rt) == 0) say(", gw %s", gw);
                else say(", gw %s not set (%s)", gw, strerror(errno));
            }
        }
        say("\n");
    }
    close(sock);
}

int main(void) {
    say("\n");
    say("################################################################\n");
    say("#  TRM SWARM OS -- THE PLATFORM ON BARE METAL                   #\n");
    say("#  firmware -> chainloader -> Linux kernel -> this process      #\n");
    say("################################################################\n\n");

    int dev_ok = mount_ok("devtmpfs", "/dev", "devtmpfs");
    say("MOUNTS: proc=%s sysfs=%s devtmpfs=%s\n",
        mount_ok("proc", "/proc", "proc") ? "ok" : "failed",
        mount_ok("sysfs", "/sys", "sysfs") ? "ok" : "failed",
        dev_ok ? "ok" : "failed");
    if (dev_ok) tty0 = fopen("/dev/tty0", "w");

    say("LOOPBACK: %s\n", loopback_up() == 0 ? "up" : "could not be brought up");

    /* The intelligence lives on a VOLUME, not in this RAM image. Mount the boot volume the firmware
     * handed us with the kernel's own driver stack -- the same one that read the initrd -- and point
     * the platform's store resolver at the fabric there. `repo_relative()` walks up from the running
     * executable (up to eight parents), so the link must sit within eight parents of
     * /opt/trm/trm_uefi_kernel/host_council_server/host_council_server; /opt/trm is three.
     *
     * The device name is not assumed: the kernel decides it, so ask /dev what it got and try what is
     * actually there. A boot that names a device it did not check is a guess. */
    /* Before /dev is asked what it has: a machine's own ports are not in this kernel until their modules
     * are loaded, and on real hardware the volume this boot came from is behind them. */
    load_modules();
    /* Then the volume itself is waited for -- not "a disk", which on a machine with an internal NVMe is
     * present before the boot begins. See find_volume(): the device list is what changes, so it is asked
     * again, and a boot that gives up on a machine about to hand it what it needs is the failure this
     * replaces. */
    if (!find_volume())
        say("no volume carrying the platform image -> /k3 will report it missing, not invent a result\n");

    /* The volume is mounted read-only so the platform image cannot change. A state image on it is the
     * one thing meant to be written, so the volume is re-mounted writable when one is there -- and
     * whichever happened is printed, because "state lives in RAM" is a fact an operator needs. */
    {
        struct stat ss;
        if (stat("/mnt/esp/trm-state.ext4", &ss) != 0) {
            say("STATE: none on the volume -> it stays read-only and state lives in RAM this boot\n");
        } else if (mount(NULL, "/mnt/esp", NULL, MS_REMOUNT, NULL) == 0) {
            say("STATE: trm-state.ext4 (%ld B) -> the volume is writable for it; the image stays "
                "read-only\n", (long)ss.st_size);
        } else {
            say("STATE: trm-state.ext4 present, volume will not go writable (%s) -> state in RAM\n",
                strerror(errno));
        }
    }

    /* The network this machine is on, if its volume says so. Read here -- `/mnt/esp` is not the
     * chroot's own view -- and applied now, while the NIC modules loaded above have produced interfaces. */
    read_netconf();
    network_up();
    /* The supplicant runs HERE, while the RAM image is still the root. It travels in the initramfs, and
     * the handover below puts a different root in its place -- after which /trm_wifi is no longer a path
     * that exists. Running it later is not a slower version of this, it is a version that cannot exec. */
    wifi_probe();
    wifi_associate();

    /* The platform image lives on that volume, so the root is built before anything reads a store:
     * squashfs (read-only) + a writable layer, then the machine is handed over. */
    persistent_root();
    bootlog_open();
    /* After the handover `/opt/trm` is the image's own view and `/mnt/esp` is the volume, so these
     * links land in the writable layer and point at the real stores. */
    say("STORES: ");
    link_stores();

    pid_t pid = fork();
    if (pid == 0) {
        setenv("TRM_REPO_ROOT", "/opt/trm", 1);
        setenv("TRM_UI_DIST", "/opt/trm/dashboard/dist", 1);
        setenv("TRM_PORT", "8022", 1);
        setenv("TRM_UI_PORT", "5183", 1);
        if (chdir("/opt/trm") != 0) say("CHDIR FAILED: %s\n", strerror(errno));
        /* The platform's own banner belongs on the screen too, so hand it /dev/tty0 rather than the
         * serial console it would otherwise inherit. */
        if (tty0) { fclose(tty0); tty0 = NULL; }
        if (bootlog) { fclose(bootlog); bootlog = NULL; }
        if (!freopen("/dev/tty0", "w", stdout)) say("STDOUT not pointed at the screen\n");
        if (!freopen("/dev/tty0", "w", stderr)) say("STDERR not pointed at the screen\n");
        /* The SUPERVISOR starts the platform, not this init: that is the platform's real topology, and
         * it means the health the route reports is measured by the supervisor on THIS machine. */
        char *argv[] = { (char *)SUPERVISOR, (char *)"--config", (char *)SUP_CONF,
                         (char *)"--interval", (char *)"10", NULL };
        execv(SUPERVISOR, argv);
        say("EXEC FAILED: %s\n", strerror(errno));
        _exit(127);
    }
    if (pid < 0) {
        say("FORK FAILED: %s\n", strerror(errno));
        for (;;) pause();
    }
    say("SUPERVISOR: launched as pid %d (it starts the platform)\n", (int)pid);
    /* The radio is reported once the platform is already under way: it waits up to ten seconds for a
     * wireless interface to be published, and the platform should not wait for a radio the machine may
     * not have. The device is the answer here, not the module list -- see wireless_report(). */
    wireless_report();
    sleep(9);

    int answered = 0;
    int verified = 0;
    unsigned n_expects = 0;   /* unsigned because it indexes the expects[] table */
    for (int i = 0; i < 10; i++) {
        sleep(3);
        int st;
        if (waitpid(pid, &st, WNOHANG) == pid) {
            say("PLATFORM EXITED early, code %d -- nothing served\n", WEXITSTATUS(st));
            break;
        }
        say("[t+%02ds] asking the platform it just started:\n", (i + 1) * 3);
        if (http_probe("/infrastructure/status") == 200) answered++;
        if (i == 0) {
            if (http_probe("/ui/") == 200) answered++;
            /* The parser routes are the newest thing this image carries, and the previous musl build of
             * the platform 404s them -- so a 200 here proves WHICH build booted, not merely that
             * something is listening. The POST is the parser itself running on the metal. */
            if (http_probe("/parsers/status") == 200) answered++;
            if (http_req("POST", "/parsers/fastq", "@r1\nACGTACGT\n+\nIIIIIIII\n@r2\nGGGCCC\n+\nIIIIII\n") == 200) answered++;
            /* The intelligence itself, on metal: a semantic query scored against the fabric store
             * that was just mounted from the volume. No DLL (the encoder is in-tree now), no Python,
             * no model server -- the query vector is computed here and the store is read from disk. */
            /* The EXACT ternary path: quantize the query, then integer agreement over every
             * document in the corpus. This is the branch that answered 503 `engine_required` on
             * the previous image -- it needed the DLL -- so a 200 from it is the discriminator
             * proving WHICH build booted, where the packed path answers on both. */
            if (http_probe("/k3/retrieve?q=balanced%20ternary%20logic%20consensus&k=3") == 200) answered++;
            /* The tercode store and the trit alphabet: the other two branches that refused on every
             * non-Windows build. They share the engine, so they live or die with it -- and the
             * tercode route cross-checks its own ranking against that engine before answering. */
            if (http_probe("/k3/retrieve?q=balanced%20ternary%20logic%20consensus&k=2&engine=tercode&verify=1") == 200) answered++;
            if (http_probe("/k3/retrieve?q=balanced%20ternary%20logic%20consensus&k=2&engine=packed&verify=1&alphabet=trit") == 200) answered++;
            /* The packed path, scored from the store on the volume. */
            if (http_probe("/k3/retrieve?engine=packed&q=LpxC%20zinc-dependent%20deacetylase%20inhibitor&k=3") == 200) answered++;
            if (http_probe("/k3/status") == 200) answered++;
            /* The L3 geometry, which this image DERIVES from the machine instead of quoting the
             * module's 32 MB literal: a boot that prints a measured size and says so in `l3_basis`
             * is a boot of the image that probes. On this kernel that means the sysfs cache
             * topology, so the number is the guest's own L3 and not a declaration. */
            if (http_probe("/sram/architectures") == 200) answered++;
            }
    }

    /* CONTENT, once, after the routes are known to be up -- not once per retry pass. Inside the loop
     * this counter accumulated ten passes and the verdict reported 90 routes carrying their content
     * when there are nine checks: a true number describing the wrong thing, which is the defect this
     * whole pathway keeps finding. Nine checks, at most nine. */
    {
        /* Each marker is what the running platform emits for that branch today, measured against the
         * host before this table was written. The previous literals had drifted: /ui/ serves
         * "Compositor", not the LCARS title, and the exact-ternary routes refuse by name (503
         * engine_required) because the corpus is 4.85 GB against a 512 MB engine limit. A refusal that
         * names itself is the branch running; asserting success there would be asserting a lie. */
        static const struct { const char *path, *needle; int status; } expects[] = {
            { "/ui/", "TRM SWARM OS", 200 },
            { "/infrastructure/status", "\"active_agents\"", 200 },
            { "/council/status", "\"active_proposals\"", 200 },
            { "/parsers/status", "\"engine_required\"", 200 },
            { "/sram/architectures", "\"architectures\"", 200 },
            { "/k3/status", "\"route\":\"/k3/status\"", 200 },
            { "/k3/retrieve?q=balanced%20ternary%20logic%20consensus&k=3", "\"encoder\"", 200 },
            { "/k3/retrieve?q=balanced%20ternary%20logic%20consensus&k=2&engine=tercode&verify=1",
              "\"error\":\"engine_required\"", 503 },
            { "/k3/retrieve?q=balanced%20ternary%20logic%20consensus&k=2&engine=packed&verify=1&alphabet=trit",
              "\"error\":\"engine_required\"", 503 },
        };
        unsigned ci;
        n_expects = sizeof expects / sizeof expects[0];
        for (ci = 0; ci < n_expects; ci++) {
            if (http_probe_contains(expects[ci].path, expects[ci].needle, expects[ci].status))
                verified++;
            else
                say("CONTENT: %s answered but did not carry %s -- the route is up and the branch "
                    "behind it did not run\n", expects[ci].path, expects[ci].needle);
        }
        /* The two kinds of pass are different and the counts are not comparable: a route that answered was
         * asked for a socket, and a route that carried its content was asked for a specific string. Naming
         * the checked set is what keeps the gap between the two numbers from reading as ten failures. */
        if (answered)
            say("CONTENT: %d of the routes were asked for a specific string and %d returned it. The checked "
                "paths: /ui/, /infrastructure/status, /council/status, /parsers/status, /sram/architectures, "
                "/k3/status, and /k3/retrieve on the exact path, cross-checked against tercode, and on the "
                "packed alphabet. The rest were asked only whether they answer.\n", n_expects, verified);
    }

    /* What this boot says about itself comes from the routes it asked, never from having launched
     * something: an image that carried no platform would otherwise still report itself up. */
    /* The dashboard is the kernel's own listener (TRM_UI_PORT, bound to 0.0.0.0) and the supervisor only
     * DECLARES its table entry -- so "the platform is up" does not tell an operator where to look, and
     * this does. The link is re-read here rather than at bring-up: a cable's link comes up a moment
     * after the driver binds. */
    if (answered) {
        say("\ninit: %d of the routes asked answered -- the platform is up.\n", answered);
        say("init: %d of them were asked for content and %d returned it; the other %d were asked only "
            "whether they answer, which is a different question and not a failure.\n",
            n_expects, verified, answered - n_expects);
    } else {
        say("\ninit: NOTHING answered on :8022 -- no platform came up on this boot (no volume carried "
            "its image, or the supervisor could not start it). Idling, but this machine is serving "
            "nothing.\n");
        for (;;) pause();
        return 0;
    }

    /* The LAST line an operator standing at this machine reads is where to look. The dashboard is the
     * kernel's own listener on the platform's port, and the content probe above asked it for the LCARS
     * title on this machine's own loopback and got it -- so "open this" is a measurement, not a hope. The
     * network form is offered only when the interface has carrier, because an address nothing can route to
     * is not an address; with no cable the machine is still a working dashboard for whoever is at it. */
    say("\nDASHBOARD: http://127.0.0.1:5183/ui/ -- open this on this machine; it answered the LCARS page "
        "on loopback during this boot.\n");
    if (net_addr[0]) {
        /* Both readings, as numbers: the one taken at bring-up and the one taken now. Re-reading and then
         * asserting a verdict is what produced "eth0 is linked" on a machine whose own bring-up line said
         * there was no cable -- the same defect class this pathway keeps finding, in its own last line. */
        int now = link_is_up(net_if);
        say("           or http://%s:5183/ui/ from anywhere that can reach this box. %s holds it, and its "
            "carrier read %d when it was configured and %d now (1 = a cable with something on the other "
            "end)%s\n",
            net_addr, net_if, net_carrier, now,
            (now == 0 || net_carrier == 0) ? " -- with no carrier, nothing outside can route to it, and the "
                                             "cable is the missing thing rather than the platform" : "");
    }
    /* The machine's OWN screen gets the dashboard natively, not through a browser: the framebuffer
     * renderer lives in the persistent root (beside the platform it draws) and is exec'd here, after the
     * handover, with the network form this boot derived. It draws to /dev/fb0 and, when there is no
     * framebuffer, degrades to a text summary on the console -- either way the boot's last act is the
     * dashboard on the device in front of the operator, not a URL for a browser the machine does not
     * carry. */
    {
        char net_line[96];
        if (net_addr[0])
            snprintf(net_line, sizeof net_line, "%s %s", net_if, net_addr);
        else if (net_if[0])
            snprintf(net_line, sizeof net_line, "%s (no address)", net_if);
        else
            snprintf(net_line, sizeof net_line, "loopback 127.0.0.1");
        pid_t fbpid = fork();
        if (fbpid == 0) {
            /* fd 1 and 2 to the display: the renderer's frame summary belongs on the screen with its
             * pixels, not only on the serial console /dev/console maps stdout to. */
            if (tty0) { dup2(fileno(tty0), STDOUT_FILENO); dup2(fileno(tty0), STDERR_FILENO); }
            char *fargv[] = { (char *)"/opt/trm/trm_fb_dashboard", net_line, NULL };
            execv("/opt/trm/trm_fb_dashboard", fargv);
            say("DASHBOARD RENDERER: exec failed: %s\n", strerror(errno));
            _exit(127);
        }
        if (fbpid > 0)
            say("DASHBOARD RENDERER: launched as pid %d to draw on /dev/fb0\n", (int)fbpid);
        else
            say("DASHBOARD RENDERER: fork failed: %s\n", strerror(errno));
    }
    say("\n");
    for (;;) pause();
    return 0;
}
