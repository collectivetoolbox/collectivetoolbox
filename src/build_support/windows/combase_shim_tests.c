#include <stdio.h>
#include <string.h>
#include <windows.h>

static BOOL fail_stack_query = FALSE;
static BOOL attempted_combase_load = FALSE;
static BOOL selected_ole32_before_combase_load = FALSE;

static HMODULE WINAPI test_get_module_handle(LPCSTR name) {
    if (strcmp(name, "combase.dll") == 0) {
        return NULL;
    }
    if (strcmp(name, "ole32.dll") == 0 && !attempted_combase_load) {
        selected_ole32_before_combase_load = TRUE;
    }
    return GetModuleHandleA(name);
}

static HMODULE WINAPI test_load_library(LPCSTR name) {
    if (strcmp(name, "combase.dll") == 0) {
        attempted_combase_load = TRUE;
    }
    return LoadLibraryA(name);
}

static SIZE_T WINAPI test_virtual_query(
    LPCVOID address, PMEMORY_BASIC_INFORMATION information, SIZE_T length) {
    if (fail_stack_query) {
        return 0;
    }
    return VirtualQuery(address, information, length);
}

#define VirtualQuery test_virtual_query
#define GetModuleHandleA test_get_module_handle
#define LoadLibraryA test_load_library
#include "combase_shim.c"
#undef VirtualQuery
#undef GetModuleHandleA
#undef LoadLibraryA

static COPYFILE2_MESSAGE_ACTION CALLBACK test_progress(
    const COPYFILE2_MESSAGE *message, PVOID context) {
    (void)message;
    (void)context;
    return COPYFILE2_PROGRESS_CONTINUE;
}

int main(void) {
    HMODULE selected_com = get_com_module();
    HMODULE native_com = GetModuleHandleA("combase.dll");
    if (!attempted_combase_load || selected_ole32_before_combase_load ||
        (native_com && selected_com != native_com)) {
        fprintf(stderr, "COM module lookup did not prefer available COMBase\n");
        return 1;
    }

    DWORD *allocation = fallback_CoTaskMemAlloc(sizeof(DWORD));
    if (!allocation) {
        fprintf(stderr, "Fallback COM allocation failed\n");
        return 1;
    }
    *allocation = 0x12345678;
    DWORD *resized = fallback_CoTaskMemRealloc(allocation, 4096);
    if (!resized) {
        fallback_CoTaskMemFree(allocation);
        fprintf(stderr, "Fallback COM reallocation failed\n");
        return 1;
    }
    if (*resized != 0x12345678) {
        fallback_CoTaskMemFree(resized);
        fprintf(stderr, "Fallback COM reallocation changed existing data\n");
        return 1;
    }
    if (fallback_CoTaskMemRealloc(resized, 0) != NULL) {
        fprintf(stderr, "Zero-size COM reallocation did not free the block\n");
        return 1;
    }
    allocation = fallback_CoTaskMemRealloc(NULL, sizeof(DWORD));
    if (!allocation) {
        fprintf(stderr, "Null COM reallocation did not allocate a block\n");
        return 1;
    }
    fallback_CoTaskMemFree(allocation);
    fallback_CoTaskMemFree(NULL);

    COPYFILE2_EXTENDED_PARAMETERS parameters = {0};
    parameters.dwSize = sizeof(parameters);
    parameters.pProgressRoutine = test_progress;
    if (fallback_CopyFile2(NULL, NULL, &parameters) !=
        HRESULT_FROM_WIN32(ERROR_NOT_SUPPORTED)) {
        fprintf(stderr, "CopyFile2 silently accepted an unsupported callback\n");
        return 1;
    }

    parameters.dwSize = 0;
    if (fallback_CopyFile2(NULL, NULL, &parameters) !=
        HRESULT_FROM_WIN32(ERROR_INVALID_PARAMETER)) {
        fprintf(stderr, "CopyFile2 accepted an invalid parameter size\n");
        return 1;
    }

    HMODULE kernel = GetModuleHandleW(L"kernel32.dll");
    if (!kernel) {
        fprintf(stderr, "Cannot inspect native kernel32 exports\n");
        return 1;
    }
    FARPROC native_copy = GetProcAddress(kernel, "CopyFile2");
    if (SUCCEEDED(CopyFile2(L"", L"", NULL))) {
        fprintf(stderr, "CopyFile2 accepted empty paths\n");
        return 1;
    }
    PFN_CopyFile2 expected_copy = native_copy
        ? (PFN_CopyFile2)(void *)native_copy : fallback_CopyFile2;
    if (__imp_CopyFile2 != expected_copy) {
        fprintf(stderr, "CopyFile2 did not prefer the available native export\n");
        return 1;
    }

    ULONG_PTR low = 0;
    ULONG_PTR high = 0;
    GetCurrentThreadStackLimits(&low, &high);
    ULONG_PTR stack_address = (ULONG_PTR)&low;
    if (low >= stack_address || high <= stack_address) {
        fprintf(stderr, "Resolved stack bounds do not contain the current frame\n");
        return 1;
    }
    FARPROC native_limits = GetProcAddress(kernel, "GetCurrentThreadStackLimits");
    PFN_GetCurrentThreadStackLimits expected_limits = native_limits
        ? (PFN_GetCurrentThreadStackLimits)(void *)native_limits
        : fallback_GetCurrentThreadStackLimits;
    if (__imp_GetCurrentThreadStackLimits != expected_limits) {
        fprintf(stderr, "Stack limits did not prefer the available native export\n");
        return 1;
    }

    low = 0;
    high = 0;
    fallback_GetCurrentThreadStackLimits(&low, &high);
    if (low >= stack_address || high <= stack_address) {
        fprintf(stderr, "Fallback stack bounds do not contain the current frame\n");
        return 1;
    }

    fail_stack_query = TRUE;
    fallback_GetCurrentThreadStackLimits(&low, &high);
    NT_TIB *thread;
    __asm__("movq %%gs:0x30, %0" : "=r"(thread));
    if (low != (ULONG_PTR)thread->StackLimit || high != (ULONG_PTR)thread->StackBase) {
        fprintf(stderr, "Failed stack query fabricated bounds instead of preserving the TEB\n");
        return 1;
    }
    puts("Windows shim parameter, native-export, and stack-bound checks passed");
    return 0;
}