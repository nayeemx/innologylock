#pragma once
#include <fltKernel.h>
#include <dontuse.h>
#include <suppress.h>

#define FILTER_PORT_NAME L"\\InnLockFilterComm"
#define FILTER_DEVICE_NAME L"\\Device\\InnLockDev"
#define FILTER_SYMLINK_NAME L"\\DosDevices\\InnLockDev"

#define FILTER_IOCTL_ADD_PATH    CTL_CODE(0x8000, 0x800, METHOD_BUFFERED, FILE_ANY_ACCESS)
#define FILTER_IOCTL_REMOVE_PATH CTL_CODE(0x8000, 0x801, METHOD_BUFFERED, FILE_ANY_ACCESS)
#define FILTER_IOCTL_CLEAR_ALL   CTL_CODE(0x8000, 0x802, METHOD_BUFFERED, FILE_ANY_ACCESS)
#define FILTER_IOCTL_SET_ENABLED CTL_CODE(0x8000, 0x803, METHOD_BUFFERED, FILE_ANY_ACCESS)

#pragma pack(push, 8)
typedef struct _FILTER_ACCESS_REQUEST {
    ULONGLONG RequestId;
    ULONG     ProcessId;
    WCHAR     ProcessName[64];
    WCHAR     FilePath[512];
} FILTER_ACCESS_REQUEST;

typedef struct _FILTER_ACCESS_REPLY {
    ULONGLONG RequestId;
    LONG      Allow;
} FILTER_ACCESS_REPLY;
#pragma pack(pop)

#define MAX_LOCKED_PATHS 256
#define MAX_PATH_LEN     512

typedef struct _LOCKED_ENTRY {
    WCHAR Path[MAX_PATH_LEN];
    ULONG PathLen;
} LOCKED_ENTRY;

typedef struct _LOCKED_LIST {
    LOCKED_ENTRY Entries[MAX_LOCKED_PATHS];
    ULONG Count;
    LONG  Enabled;
    FAST_MUTEX Mutex;
} LOCKED_LIST;

extern PFLT_FILTER gFilterHandle;
extern PFLT_PORT    gServerPort;
extern PFLT_PORT    gClientPort;
extern LOCKED_LIST  gLockedList;

DRIVER_INITIALIZE DriverEntry;
FLT_PREOP_CALLBACK_STATUS FilterPreCreate(PFLT_CALLBACK_DATA, PCFLT_RELATED_OBJECTS, PVOID*);
NTSTATUS FilterUnload(FLT_FILTER_UNLOAD_FLAGS);
NTSTATUS FilterConnectNotify(PFLT_PORT, PVOID, PVOID, ULONG, PVOID*);
VOID FilterDisconnectNotify(PVOID);
NTSTATUS FilterMessageNotify(PVOID, PVOID, ULONG, PVOID, ULONG, ULONG*);
LONG IsPathLocked(PUNICODE_STRING);
NTSTATUS DeviceIoControlDispatch(PDEVICE_OBJECT, PIRP);
