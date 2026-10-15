#include "filter.h"
#include <ntstrsafe.h>

PFLT_FILTER gFilterHandle = NULL;
PFLT_PORT    gServerPort  = NULL;
PFLT_PORT    gClientPort  = NULL;
PDEVICE_OBJECT gDeviceObject = NULL;
LOCKED_LIST  gLockedList  = { 0 };
static LONG_PTR gNextReqId = 0;

static const FLT_OPERATION_REGISTRATION Callbacks[] = {
    { IRP_MJ_CREATE, 0, FilterPreCreate, NULL },
    { IRP_MJ_OPERATION_END }
};

static const FLT_REGISTRATION FilterRegistration = {
    sizeof(FLT_REGISTRATION),
    FLT_REGISTRATION_VERSION,
    0, NULL, Callbacks,
    FilterUnload,
    NULL, NULL, NULL, NULL,
    NULL, NULL, NULL, NULL, NULL
};

NTSTATUS DriverEntry(PDRIVER_OBJECT DriverObject, PUNICODE_STRING RegistryPath) {
    UNREFERENCED_PARAMETER(RegistryPath);
    NTSTATUS status;
    ExInitializeFastMutex(&gLockedList.Mutex);
    gLockedList.Enabled = 1;

    DriverObject->MajorFunction[IRP_MJ_DEVICE_CONTROL] = DeviceIoControlDispatch;

    status = FltRegisterFilter(DriverObject, &FilterRegistration, &gFilterHandle);
    if (!NT_SUCCESS(status)) return status;

    UNICODE_STRING portName;
    RtlInitUnicodeString(&portName, FILTER_PORT_NAME);
    PSECURITY_DESCRIPTOR sd;
    status = FltBuildDefaultSecurityDescriptor(&sd, FLT_PORT_ALL_ACCESS);
    if (!NT_SUCCESS(status)) {
        FltUnregisterFilter(gFilterHandle);
        return status;
    }

    OBJECT_ATTRIBUTES objAttr;
    InitializeObjectAttributes(&objAttr, &portName, OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE, NULL, sd);
    status = FltCreateCommunicationPort(gFilterHandle, &gServerPort, &objAttr, NULL,
        FilterConnectNotify, FilterDisconnectNotify, FilterMessageNotify, 0);
    FltFreeSecurityDescriptor(sd);
    if (!NT_SUCCESS(status)) {
        FltUnregisterFilter(gFilterHandle);
        return status;
    }

    UNICODE_STRING deviceName;
    RtlInitUnicodeString(&deviceName, FILTER_DEVICE_NAME);
    UNICODE_STRING symlinkName;
    RtlInitUnicodeString(&symlinkName, FILTER_SYMLINK_NAME);
    status = IoCreateDevice(DriverObject, 0, &deviceName, FILE_DEVICE_UNKNOWN, FILE_DEVICE_SECURE_OPEN, FALSE, &gDeviceObject);
    if (NT_SUCCESS(status)) {
        gDeviceObject->Flags |= DO_BUFFERED_IO;
        gDeviceObject->Flags &= ~DO_DEVICE_INITIALIZING;
        IoCreateSymbolicLink(&symlinkName, &deviceName);
    }

    status = FltStartFiltering(gFilterHandle);
    if (!NT_SUCCESS(status)) {
        FltCloseCommunicationPort(gServerPort);
        FltUnregisterFilter(gFilterHandle);
        if (gDeviceObject) {
            UNICODE_STRING sn;
            RtlInitUnicodeString(&sn, FILTER_SYMLINK_NAME);
            IoDeleteSymbolicLink(&sn);
            IoDeleteDevice(gDeviceObject);
            gDeviceObject = NULL;
        }
        return status;
    }
    return STATUS_SUCCESS;
}

NTSTATUS FilterUnload(FLT_FILTER_UNLOAD_FLAGS Flags) {
    UNREFERENCED_PARAMETER(Flags);
    FltCloseCommunicationPort(gServerPort);
    FltUnregisterFilter(gFilterHandle);
    if (gDeviceObject) {
        UNICODE_STRING symlinkName;
        RtlInitUnicodeString(&symlinkName, FILTER_SYMLINK_NAME);
        IoDeleteSymbolicLink(&symlinkName);
        IoDeleteDevice(gDeviceObject);
    }
    return STATUS_SUCCESS;
}

NTSTATUS FilterConnectNotify(PFLT_PORT ClientPort, PVOID ServerCookie, PVOID ConnectionContext, ULONG SizeOfContext, PVOID* ConnectionPortCookie) {
    UNREFERENCED_PARAMETER(ServerCookie); UNREFERENCED_PARAMETER(ConnectionContext);
    UNREFERENCED_PARAMETER(SizeOfContext); UNREFERENCED_PARAMETER(ConnectionPortCookie);
    if (gClientPort) { FltCloseClientPort(gFilterHandle, &gClientPort); }
    gClientPort = ClientPort;
    return STATUS_SUCCESS;
}

VOID FilterDisconnectNotify(PVOID ConnectionPortCookie) {
    UNREFERENCED_PARAMETER(ConnectionPortCookie);
    if (gClientPort) { FltCloseClientPort(gFilterHandle, &gClientPort); gClientPort = NULL; }
}

NTSTATUS FilterMessageNotify(PVOID PortCookie, PVOID InputBuffer, ULONG InputBufferLength,
    PVOID OutputBuffer, ULONG OutputBufferLength, ULONG* ReturnOutputBufferLength) {
    UNREFERENCED_PARAMETER(PortCookie); UNREFERENCED_PARAMETER(OutputBuffer);
    UNREFERENCED_PARAMETER(OutputBufferLength); UNREFERENCED_PARAMETER(InputBuffer);
    UNREFERENCED_PARAMETER(InputBufferLength);
    *ReturnOutputBufferLength = 0;
    return STATUS_SUCCESS;
}

LONG IsPathLocked(PUNICODE_STRING FilePath) {
    if (!FilePath || !FilePath->Buffer || FilePath->Length == 0) return 0;
    ExAcquireFastMutex(&gLockedList.Mutex);
    if (!gLockedList.Enabled) { ExReleaseFastMutex(&gLockedList.Mutex); return 0; }
    for (ULONG i = 0; i < gLockedList.Count; i++) {
        ULONG lockedChars = gLockedList.Entries[i].PathLen / sizeof(WCHAR);
        ULONG fileChars = FilePath->Length / sizeof(WCHAR);
        if (fileChars >= lockedChars) {
            if (RtlCompareMemory(FilePath->Buffer, gLockedList.Entries[i].Path, gLockedList.Entries[i].PathLen) == gLockedList.Entries[i].PathLen) {
                if (fileChars == lockedChars) { ExReleaseFastMutex(&gLockedList.Mutex); return 1; }
                WCHAR next = FilePath->Buffer[lockedChars];
                if (next == L'\\' || next == L'.' || next == L'\0') { ExReleaseFastMutex(&gLockedList.Mutex); return 1; }
            }
        }
    }
    ExReleaseFastMutex(&gLockedList.Mutex);
    return 0;
}

FLT_PREOP_CALLBACK_STATUS FilterPreCreate(PFLT_CALLBACK_DATA Data, PCFLT_RELATED_OBJECTS FltObjects, PVOID* CompletionContext) {
    UNREFERENCED_PARAMETER(FltObjects); UNREFERENCED_PARAMETER(CompletionContext);
    if (!gClientPort) return FLT_PREOP_SUCCESS_NO_CALLBACK;
    if (Data->RequestorMode == KernelMode) return FLT_PREOP_SUCCESS_NO_CALLBACK;

    PFLT_FILE_NAME_INFORMATION nameInfo;
    NTSTATUS status = FltGetFileNameInformation(Data, FLT_FILE_NAME_NORMALIZED | FLT_FILE_NAME_QUERY_DEFAULT, &nameInfo);
    if (!NT_SUCCESS(status)) return FLT_PREOP_SUCCESS_NO_CALLBACK;
    FltParseFileNameInformation(nameInfo);

    if (!IsPathLocked(&nameInfo->Name)) {
        FltReleaseFileNameInformation(nameInfo);
        return FLT_PREOP_SUCCESS_NO_CALLBACK;
    }
    FltReleaseFileNameInformation(nameInfo);

    FILTER_ACCESS_REQUEST request = { 0 };
    request.RequestId = (ULONGLONG)InterlockedExchangeAdd64(&gNextReqId, 1);
    request.ProcessId = (ULONG)(ULONG_PTR)PsGetCurrentProcessId();
    RtlStringCbPrintfW(request.ProcessName, sizeof(request.ProcessName), L"PID:%lu", request.ProcessId);

    PFLT_FILE_NAME_INFORMATION nameInfo2;
    status = FltGetFileNameInformationUnsafe(FltObjects->FileObject, FltObjects->Instance,
        FLT_FILE_NAME_NORMALIZED | FLT_FILE_NAME_QUERY_DEFAULT, &nameInfo2);
    if (NT_SUCCESS(status) && nameInfo2) {
        FltParseFileNameInformation(nameInfo2);
        ULONG copyLen = nameInfo2->Name.Length;
        if (copyLen > sizeof(request.FilePath) - sizeof(WCHAR)) copyLen = sizeof(request.FilePath) - sizeof(WCHAR);
        RtlCopyMemory(request.FilePath, nameInfo2->Name.Buffer, copyLen);
        FltReleaseFileNameInformation(nameInfo2);
    }

    PVOID replyBuffer = NULL;
    ULONG replyLength = 0;
    LARGE_INTEGER timeout;
    timeout.QuadPart = -1200000000LL;

    status = FltSendMessage(gFilterHandle, &gClientPort, &request, sizeof(request), &replyBuffer, &replyLength, &timeout);
    if (NT_SUCCESS(status) && replyBuffer && replyLength >= sizeof(FILTER_ACCESS_REPLY)) {
        FILTER_ACCESS_REPLY* reply = (FILTER_ACCESS_REPLY*)replyBuffer;
        LONG allow = reply->Allow;
        ExFreePoolWithTag(replyBuffer, 'klnI');
        if (allow) {
            Data->IoStatus.Status = STATUS_SUCCESS;
            return FLT_PREOP_SUCCESS_NO_CALLBACK;
        }
    }

    Data->IoStatus.Status = STATUS_ACCESS_DENIED;
    Data->IoStatus.Information = 0;
    return FLT_PREOP_COMPLETE;
}

NTSTATUS DeviceIoControlDispatch(PDEVICE_OBJECT DeviceObject, PIRP Irp) {
    UNREFERENCED_PARAMETER(DeviceObject);
    PIO_STACK_LOCATION stack = IoGetCurrentIrpStackLocation(Irp);
    NTSTATUS status = STATUS_INVALID_DEVICE_REQUEST;
    ULONG info = 0;

    if (stack->Parameters.DeviceIoControl.IoControlCode == FILTER_IOCTL_ADD_PATH) {
        ULONG inLen = stack->Parameters.DeviceIoControl.InputBufferLength;
        if (inLen >= sizeof(WCHAR) && inLen <= MAX_PATH_LEN * sizeof(WCHAR)) {
            WCHAR* path = (WCHAR*)Irp->AssociatedIrp.SystemBuffer;
            ULONG pathLen = inLen - sizeof(WCHAR);
            ExAcquireFastMutex(&gLockedList.Mutex);
            if (gLockedList.Count < MAX_LOCKED_PATHS) {
                RtlCopyMemory(gLockedList.Entries[gLockedList.Count].Path, path, pathLen);
                gLockedList.Entries[gLockedList.Count].PathLen = pathLen;
                gLockedList.Count++;
                status = STATUS_SUCCESS;
            } else {
                status = STATUS_BUFFER_OVERFLOW;
            }
            ExReleaseFastMutex(&gLockedList.Mutex);
        } else {
            status = STATUS_INVALID_PARAMETER;
        }
    } else if (stack->Parameters.DeviceIoControl.IoControlCode == FILTER_IOCTL_REMOVE_PATH) {
        ULONG inLen = stack->Parameters.DeviceIoControl.InputBufferLength;
        if (inLen >= sizeof(WCHAR)) {
            WCHAR* path = (WCHAR*)Irp->AssociatedIrp.SystemBuffer;
            ULONG pathLen = inLen - sizeof(WCHAR);
            ExAcquireFastMutex(&gLockedList.Mutex);
            for (ULONG i = 0; i < gLockedList.Count; i++) {
                if (gLockedList.Entries[i].PathLen == pathLen &&
                    RtlCompareMemory(gLockedList.Entries[i].Path, path, pathLen) == pathLen) {
                    if (i < gLockedList.Count - 1) {
                        RtlMoveMemory(&gLockedList.Entries[i], &gLockedList.Entries[i+1],
                            (gLockedList.Count - i - 1) * sizeof(LOCKED_ENTRY));
                    }
                    gLockedList.Count--;
                    status = STATUS_SUCCESS;
                    break;
                }
            }
            ExReleaseFastMutex(&gLockedList.Mutex);
            if (!NT_SUCCESS(status)) status = STATUS_SUCCESS;
        }
    } else if (stack->Parameters.DeviceIoControl.IoControlCode == FILTER_IOCTL_CLEAR_ALL) {
        ExAcquireFastMutex(&gLockedList.Mutex);
        gLockedList.Count = 0;
        ExReleaseFastMutex(&gLockedList.Mutex);
        status = STATUS_SUCCESS;
    } else if (stack->Parameters.DeviceIoControl.IoControlCode == FILTER_IOCTL_SET_ENABLED) {
        if (stack->Parameters.DeviceIoControl.InputBufferLength >= sizeof(LONG)) {
            LONG* val = (LONG*)Irp->AssociatedIrp.SystemBuffer;
            ExAcquireFastMutex(&gLockedList.Mutex);
            gLockedList.Enabled = *val;
            ExReleaseFastMutex(&gLockedList.Mutex);
            status = STATUS_SUCCESS;
        }
    }

    Irp->IoStatus.Status = status;
    Irp->IoStatus.Information = info;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
    return status;
}
