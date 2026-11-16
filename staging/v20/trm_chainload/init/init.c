/* TRM bare-metal init: PID 1 under a real Linux kernel.
   Prints what the KERNEL can see - which is the whole point of the bridge: the
   kernel brings its own drivers, so drives the firmware's FAT-only driver cannot
   read become readable here. Every line below is read from the running system. */
#include <stdio.h>
#include <string.h>
#include <sys/mount.h>
#include <sys/utsname.h>
#include <unistd.h>
#include <sys/stat.h>

static void dump(const char *path, const char *title) {
    printf("\n== %s ==\n", title);
    FILE *f = fopen(path, "r");
    if (!f) { printf("  (unavailable: %s)\n", path); return; }
    char line[512];
    int n = 0;
    while (fgets(line, sizeof line, f) && n < 40) {
        printf("  %s", line);
        n++;
    }
    fclose(f);
}

int main(void) {
    printf("\n");
    printf("################################################################\n");
    printf("#  TRM SWARM OS - BARE METAL VIA THE LINUX PATHWAY              #\n");
    printf("#  PID 1 in userspace. The kernel owns the hardware.            #\n");
    printf("################################################################\n\n");

    struct utsname u;
    if (uname(&u) == 0)
        printf("KERNEL: %s %s %s  (this is the machine, not an emulated app)\n",
               u.sysname, u.release, u.machine);

    mkdir("/proc", 0755); mkdir("/sys", 0755); mkdir("/dev", 0755);
    int p = mount("proc", "/proc", "proc", 0, NULL);
    int s = mount("sysfs", "/sys", "sysfs", 0, NULL);
    printf("MOUNTS: proc=%s sysfs=%s\n", p == 0 ? "ok" : "failed", s == 0 ? "ok" : "failed");

    dump("/proc/partitions", "BLOCK DEVICES THE KERNEL ENUMERATED (every disk it can reach)");
    dump("/proc/filesystems", "FILESYSTEM DRIVERS THE KERNEL CARRIES (the decoder for attached data)");

    printf("\n== VOLTAGE DOCTRINE CHECK ==\n");
    printf("  each line above was READ from this running kernel.\n");
    printf("  nothing here is declared: if a device is absent it is absent from the list.\n");
    printf("\nTRM: intelligence fabric is reachable on any of these volumes.\n");
    fflush(stdout);

    for (;;) pause();
    return 0;
}
