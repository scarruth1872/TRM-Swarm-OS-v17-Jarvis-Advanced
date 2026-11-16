// trm_ec_smbus.c — ring-0 EC/SMBus voltage reader (minimal WDM driver)
//
// Exposes three IOCTLs over \\.\trm_ec_smbus:
//   TRM_IOCTL_EC_READ    — read one EC RAM byte (index in the input buffer)
//   TRM_IOCTL_EC_SCAN    — read a raw EC RAM block (start+len in the input buffer)
//   TRM_IOCTL_SMBUS_READ — one SMBus byte transaction (addr+cmd in the input buffer)
//
// Uses only HAL port primitives. No PnP, no IRP queue — the minimal ring-0 surface.

#include <ntddk.h>
#include <wdm.h>

#define TRM_DEVICE_NAME   L"\\Device\\trm_ec_smbus"
#define TRM_DOS_NAME      L"\\DosDevices\\trm_ec_smbus"
#define TRM_DEVICE_TYPE   FILE_DEVICE_UNKNOWN

#define TRM_IOCTL_EC_READ    CTL_CODE(TRM_DEVICE_TYPE, 0x800, METHOD_BUFFERED, FILE_ANY_ACCESS)
#define TRM_IOCTL_EC_SCAN    CTL_CODE(TRM_DEVICE_TYPE, 0x801, METHOD_BUFFERED, FILE_ANY_ACCESS)
#define TRM_IOCTL_SMBUS_READ CTL_CODE(TRM_DEVICE_TYPE, 0x802, METHOD_BUFFERED, FILE_ANY_ACCESS)

// ── Embedded Controller (PNP0C09) ─────────────────────────────────────────────
#define EC_DATA_PORT  0x62
#define EC_CMD_PORT   0x66
#define EC_READ_CMD   0x80
#define EC_WRITE_CMD  0x81

static UCHAR ec_read(UCHAR index) {
    // Wait for the input buffer to be empty (IBF == 0). The port read itself is the
    // microsecond-scale delay, so no explicit stall call is needed.
    int guard = 0;
    while ((READ_PORT_UCHAR((PUCHAR)EC_CMD_PORT) & 0x02) && guard++ < 10000) {
    }
    WRITE_PORT_UCHAR((PUCHAR)EC_CMD_PORT, EC_READ_CMD);
    WRITE_PORT_UCHAR((PUCHAR)EC_DATA_PORT, index);
    // Wait for the output buffer to be full (OBF == 1).
    guard = 0;
    while (!(READ_PORT_UCHAR((PUCHAR)EC_CMD_PORT) & 0x01) && guard++ < 10000) {
    }
    return READ_PORT_UCHAR((PUCHAR)EC_DATA_PORT);
}

// ── AMD SMBus (FCH 790B) — fixed I/O window, base discovered by the caller ────
// The caller passes the SMBus base + the device address + command in the input
// buffer; the driver performs the Host Status/Data transaction at that window.
#define SMB_HOST_STATUS 0x00
#define SMB_HOST_CTRL   0x02
#define SMB_HOST_CMD    0x03
#define SMB_HOST_ADDR   0x04
#define SMB_HOST_DATA0  0x05

static NTSTATUS smbus_read_word(ULONG base, UCHAR addr, UCHAR cmd, ULONG *out) {
    if (base == 0) return STATUS_INVALID_PARAMETER;
    WRITE_PORT_UCHAR((PUCHAR)(base + SMB_HOST_STATUS), 0x1E); // clear status
    WRITE_PORT_UCHAR((PUCHAR)(base + SMB_HOST_CMD), cmd);
    WRITE_PORT_UCHAR((PUCHAR)(base + SMB_HOST_ADDR), (UCHAR)(addr << 1));
    WRITE_PORT_UCHAR((PUCHAR)(base + SMB_HOST_CTRL), 0x48);   // start, byte data
    int guard = 0;
    while ((READ_PORT_UCHAR((PUCHAR)(base + SMB_HOST_STATUS)) & 0x02) && guard++ < 10000) {
    }
    *out = READ_PORT_UCHAR((PUCHAR)(base + SMB_HOST_DATA0));
    return STATUS_SUCCESS;
}

// ── Dispatch ──────────────────────────────────────────────────────────────────
NTSTATUS TrmDeviceControl(PDEVICE_OBJECT dev, PIRP irp) {
    PIO_STACK_LOCATION s = IoGetCurrentIrpStackLocation(irp);
    ULONG code = s->Parameters.DeviceIoControl.IoControlCode;
    PUCHAR in = (PUCHAR)irp->AssociatedIrp.SystemBuffer;
    ULONG in_len = s->Parameters.DeviceIoControl.InputBufferLength;
    PUCHAR out = (PUCHAR)irp->AssociatedIrp.SystemBuffer;
    ULONG out_len = s->Parameters.DeviceIoControl.OutputBufferLength;
    NTSTATUS st = STATUS_SUCCESS;

    switch (code) {
    case TRM_IOCTL_EC_READ:
        if (in_len < 1 || out_len < 1) { st = STATUS_BUFFER_TOO_SMALL; break; }
        out[0] = ec_read(in[0]);
        irp->IoStatus.Information = 1;
        break;
    case TRM_IOCTL_EC_SCAN: {
        if (in_len < 2 || out_len < 1) { st = STATUS_BUFFER_TOO_SMALL; break; }
        UCHAR start = in[0], len = in[1];
        ULONG n = 0;
        for (UCHAR i = 0; i < len && n < out_len; i++) out[n++] = ec_read(start + i);
        irp->IoStatus.Information = n;
        break;
    }
    case TRM_IOCTL_SMBUS_READ: {
        if (in_len < 12 || out_len < 4) { st = STATUS_BUFFER_TOO_SMALL; break; }
        ULONG base = *(ULONG *)in;
        UCHAR addr = in[4], cmd = in[5];
        ULONG val = 0;
        st = smbus_read_word(base, addr, cmd, &val);
        *(ULONG *)out = val;
        irp->IoStatus.Information = 4;
        break;
    }
    default:
        st = STATUS_INVALID_DEVICE_REQUEST;
        break;
    }
    irp->IoStatus.Status = st;
    IoCompleteRequest(irp, IO_NO_INCREMENT);
    return st;
}

NTSTATUS TrmDispatch(PDEVICE_OBJECT dev, PIRP irp) {
    PIO_STACK_LOCATION s = IoGetCurrentIrpStackLocation(irp);
    switch (s->MajorFunction) {
    case IRP_MJ_CREATE:
    case IRP_MJ_CLOSE:
        irp->IoStatus.Status = STATUS_SUCCESS;
        irp->IoStatus.Information = 0;
        break;
    case IRP_MJ_DEVICE_CONTROL:
        return TrmDeviceControl(dev, irp);
    default:
        irp->IoStatus.Status = STATUS_INVALID_DEVICE_REQUEST;
        irp->IoStatus.Information = 0;
        break;
    }
    IoCompleteRequest(irp, IO_NO_INCREMENT);
    return irp->IoStatus.Status;
}

VOID TrmUnload(PDRIVER_OBJECT drv) {
    UNICODE_STRING dos;
    RtlInitUnicodeString(&dos, TRM_DOS_NAME);
    IoDeleteSymbolicLink(&dos);
    if (drv->DeviceObject) IoDeleteDevice(drv->DeviceObject);
}

NTSTATUS DriverEntry(PDRIVER_OBJECT drv, PUNICODE_STRING reg) {
    UNREFERENCED_PARAMETER(reg);
    UNICODE_STRING name, dos;
    RtlInitUnicodeString(&name, TRM_DEVICE_NAME);
    RtlInitUnicodeString(&dos, TRM_DOS_NAME);
    PDEVICE_OBJECT dev = NULL;
    NTSTATUS st = IoCreateDevice(drv, 0, &name, TRM_DEVICE_TYPE, 0, FALSE, &dev);
    if (!NT_SUCCESS(st)) return st;
    st = IoCreateSymbolicLink(&dos, &name);
    if (!NT_SUCCESS(st)) { IoDeleteDevice(dev); return st; }
    for (int i = 0; i <= IRP_MJ_MAXIMUM_FUNCTION; i++) drv->MajorFunction[i] = TrmDispatch;
    drv->DriverUnload = TrmUnload;
    return STATUS_SUCCESS;
}
