// SPDX-License-Identifier: AGPL-3.0-or-later
//
// This file is part of Collective Toolbox, a database and document workspace and utilities.
// Copyright (C) 2026 Collective Toolbox Developers
// Contact: info@collectivetoolbox.com
//
// Static shim embedded directly into Windows binaries to resolve combase.dll symbols
// (such as CoCreateFreeThreadedMarshaler, CoTaskMemAlloc, CoTaskMemFree) at runtime
// via ole32.dll on Windows 7, without creating a load-time PE import dependency on
// combase.dll (which only exists on Windows 8 and newer).

#include <windows.h>
#include <objbase.h>

#ifndef E_NOTIMPL
#define E_NOTIMPL ((HRESULT)0x80004001L)
#endif

#ifndef REGDB_E_CLASSNOTREG
#define REGDB_E_CLASSNOTREG ((HRESULT)0x80040154L)
#endif

static HMODULE get_com_module(void) {
    static HMODULE s_hComMod = NULL;
    if (!s_hComMod) {
        // On Windows 8+, combase.dll is available.
        // On Windows 7, only ole32.dll exists.
        s_hComMod = GetModuleHandleA("combase.dll");
        if (!s_hComMod) {
            s_hComMod = GetModuleHandleA("ole32.dll");
        }
        if (!s_hComMod) {
            s_hComMod = LoadLibraryA("combase.dll");
        }
        if (!s_hComMod) {
            s_hComMod = LoadLibraryA("ole32.dll");
        }
    }
    return s_hComMod;
}

// ============================================================================
// Function Prototypes and Resolvers
// ============================================================================

typedef LPVOID (WINAPI *PFN_CoTaskMemAlloc)(SIZE_T cb);
typedef void (WINAPI *PFN_CoTaskMemFree)(LPVOID pv);
typedef LPVOID (WINAPI *PFN_CoTaskMemRealloc)(LPVOID pv, SIZE_T cb);
typedef HRESULT (WINAPI *PFN_CoCreateFreeThreadedMarshaler)(LPUNKNOWN punkOuter, LPUNKNOWN *ppunkMarshal);
typedef HRESULT (WINAPI *PFN_CoIncrementMTAUsage)(CO_MTA_USAGE_COOKIE *pCookie);
typedef HRESULT (WINAPI *PFN_CoDecrementMTAUsage)(CO_MTA_USAGE_COOKIE cookie);
typedef HRESULT (WINAPI *PFN_RoGetAgileReference)(int options, const void *riid, void *punk, void **ppagilereference);
typedef HRESULT (WINAPI *PFN_RoGetActivationFactory)(void *activatableClassId, const void *iid, void **factory);
typedef HRESULT (WINAPI *PFN_RoInitialize)(int initType);
typedef void (WINAPI *PFN_RoUninitialize)(void);
typedef HRESULT (WINAPI *PFN_RoActivateInstance)(void *activatableClassId, void **instance);
typedef int (WINAPI *PFN_GetHostNameW)(WCHAR *name, int namelen);
typedef BOOL (WINAPI *PFN_RoOriginateErrorW)(HRESULT error, UINT cchMax, const WCHAR *message);
typedef BOOL (WINAPI *PFN_RoOriginateError)(HRESULT error, void *message);

// Forward declarations of initial resolver stubs
static LPVOID WINAPI resolve_CoTaskMemAlloc(SIZE_T cb);
static void WINAPI resolve_CoTaskMemFree(LPVOID pv);
static LPVOID WINAPI resolve_CoTaskMemRealloc(LPVOID pv, SIZE_T cb);
static HRESULT WINAPI resolve_CoCreateFreeThreadedMarshaler(LPUNKNOWN punkOuter, LPUNKNOWN *ppunkMarshal);
static HRESULT WINAPI resolve_CoIncrementMTAUsage(CO_MTA_USAGE_COOKIE *pCookie);
static HRESULT WINAPI resolve_CoDecrementMTAUsage(CO_MTA_USAGE_COOKIE cookie);
static HRESULT WINAPI resolve_RoGetAgileReference(int options, const void *riid, void *punk, void **ppagilereference);
static HRESULT WINAPI resolve_RoGetActivationFactory(void *activatableClassId, const void *iid, void **factory);
static HRESULT WINAPI resolve_RoInitialize(int initType);
static void WINAPI resolve_RoUninitialize(void);
static HRESULT WINAPI resolve_RoActivateInstance(void *activatableClassId, void **instance);
static int WINAPI resolve_GetHostNameW(WCHAR *name, int namelen);
static BOOL WINAPI resolve_RoOriginateErrorW(HRESULT error, UINT cchMax, const WCHAR *message);
static BOOL WINAPI resolve_RoOriginateError(HRESULT error, void *message);

// ============================================================================
// __imp_ Function Pointers (imported by raw-dylib / linkers)
// ============================================================================

PFN_CoTaskMemAlloc __imp_CoTaskMemAlloc = resolve_CoTaskMemAlloc;
PFN_CoTaskMemFree __imp_CoTaskMemFree = resolve_CoTaskMemFree;
PFN_CoTaskMemRealloc __imp_CoTaskMemRealloc = resolve_CoTaskMemRealloc;
PFN_CoCreateFreeThreadedMarshaler __imp_CoCreateFreeThreadedMarshaler = resolve_CoCreateFreeThreadedMarshaler;
PFN_CoIncrementMTAUsage __imp_CoIncrementMTAUsage = resolve_CoIncrementMTAUsage;
PFN_CoDecrementMTAUsage __imp_CoDecrementMTAUsage = resolve_CoDecrementMTAUsage;
PFN_RoGetAgileReference __imp_RoGetAgileReference = resolve_RoGetAgileReference;
PFN_RoGetActivationFactory __imp_RoGetActivationFactory = resolve_RoGetActivationFactory;
PFN_RoInitialize __imp_RoInitialize = resolve_RoInitialize;
PFN_RoUninitialize __imp_RoUninitialize = resolve_RoUninitialize;
PFN_RoActivateInstance __imp_RoActivateInstance = resolve_RoActivateInstance;
PFN_GetHostNameW __imp_GetHostNameW = resolve_GetHostNameW;
PFN_RoOriginateErrorW __imp_RoOriginateErrorW = resolve_RoOriginateErrorW;
PFN_RoOriginateError __imp_RoOriginateError = resolve_RoOriginateError;

// ============================================================================
// Direct Function Stubs (for call <symbol>)
// ============================================================================

LPVOID WINAPI CoTaskMemAlloc(SIZE_T cb) {
    return __imp_CoTaskMemAlloc(cb);
}

void WINAPI CoTaskMemFree(LPVOID pv) {
    __imp_CoTaskMemFree(pv);
}

LPVOID WINAPI CoTaskMemRealloc(LPVOID pv, SIZE_T cb) {
    return __imp_CoTaskMemRealloc(pv, cb);
}

HRESULT WINAPI CoCreateFreeThreadedMarshaler(LPUNKNOWN punkOuter, LPUNKNOWN *ppunkMarshal) {
    return __imp_CoCreateFreeThreadedMarshaler(punkOuter, ppunkMarshal);
}

HRESULT WINAPI CoIncrementMTAUsage(CO_MTA_USAGE_COOKIE *pCookie) {
    return __imp_CoIncrementMTAUsage(pCookie);
}

HRESULT WINAPI CoDecrementMTAUsage(CO_MTA_USAGE_COOKIE cookie) {
    return __imp_CoDecrementMTAUsage(cookie);
}

HRESULT WINAPI RoGetAgileReference(int options, const void *riid, void *punk, void **ppagilereference) {
    return __imp_RoGetAgileReference(options, riid, punk, ppagilereference);
}

HRESULT WINAPI RoGetActivationFactory(void *activatableClassId, const void *iid, void **factory) {
    return __imp_RoGetActivationFactory(activatableClassId, iid, factory);
}

HRESULT WINAPI RoInitialize(int initType) {
    return __imp_RoInitialize(initType);
}

void WINAPI RoUninitialize(void) {
    __imp_RoUninitialize();
}

HRESULT WINAPI RoActivateInstance(void *activatableClassId, void **instance) {
    return __imp_RoActivateInstance(activatableClassId, instance);
}

int WINAPI GetHostNameW(WCHAR *name, int namelen) {
    return __imp_GetHostNameW(name, namelen);
}

BOOL WINAPI RoOriginateErrorW(HRESULT error, UINT cchMax, const WCHAR *message) {
    return __imp_RoOriginateErrorW(error, cchMax, message);
}

BOOL WINAPI RoOriginateError(HRESULT error, void *message) {
    return __imp_RoOriginateError(error, message);
}

// ============================================================================
// Resolver Implementations
// ============================================================================

static LPVOID WINAPI resolve_CoTaskMemAlloc(SIZE_T cb) {
    HMODULE hMod = get_com_module();
    FARPROC proc = hMod ? GetProcAddress(hMod, "CoTaskMemAlloc") : NULL;
    PFN_CoTaskMemAlloc fn;
    if (proc) {
        fn = (PFN_CoTaskMemAlloc)(void *)proc;
    } else {
        fn = (PFN_CoTaskMemAlloc)(void *)LocalAlloc;
    }
    __imp_CoTaskMemAlloc = fn;
    return fn(cb);
}

static void WINAPI resolve_CoTaskMemFree(LPVOID pv) {
    HMODULE hMod = get_com_module();
    FARPROC proc = hMod ? GetProcAddress(hMod, "CoTaskMemFree") : NULL;
    PFN_CoTaskMemFree fn;
    if (proc) {
        fn = (PFN_CoTaskMemFree)(void *)proc;
    } else {
        fn = (PFN_CoTaskMemFree)(void *)LocalFree;
    }
    __imp_CoTaskMemFree = fn;
    fn(pv);
}

static LPVOID WINAPI resolve_CoTaskMemRealloc(LPVOID pv, SIZE_T cb) {
    HMODULE hMod = get_com_module();
    FARPROC proc = hMod ? GetProcAddress(hMod, "CoTaskMemRealloc") : NULL;
    PFN_CoTaskMemRealloc fn;
    if (proc) {
        fn = (PFN_CoTaskMemRealloc)(void *)proc;
    } else {
        fn = (PFN_CoTaskMemRealloc)(void *)LocalReAlloc;
    }
    __imp_CoTaskMemRealloc = fn;
    return fn(pv, cb);
}

static HRESULT WINAPI resolve_CoCreateFreeThreadedMarshaler(LPUNKNOWN punkOuter, LPUNKNOWN *ppunkMarshal) {
    HMODULE hMod = get_com_module();
    FARPROC proc = hMod ? GetProcAddress(hMod, "CoCreateFreeThreadedMarshaler") : NULL;
    if (proc) {
        PFN_CoCreateFreeThreadedMarshaler fn = (PFN_CoCreateFreeThreadedMarshaler)(void *)proc;
        __imp_CoCreateFreeThreadedMarshaler = fn;
        return fn(punkOuter, ppunkMarshal);
    }
    if (ppunkMarshal) {
        *ppunkMarshal = NULL;
    }
    return E_NOTIMPL;
}

static HRESULT WINAPI fallback_CoIncrementMTAUsage(CO_MTA_USAGE_COOKIE *pCookie) {
    if (pCookie) {
        *pCookie = (CO_MTA_USAGE_COOKIE)(UINT_PTR)1;
    }
    return S_OK;
}

static HRESULT WINAPI resolve_CoIncrementMTAUsage(CO_MTA_USAGE_COOKIE *pCookie) {
    HMODULE hMod = get_com_module();
    FARPROC proc = hMod ? GetProcAddress(hMod, "CoIncrementMTAUsage") : NULL;
    PFN_CoIncrementMTAUsage fn;
    if (proc) {
        fn = (PFN_CoIncrementMTAUsage)(void *)proc;
    } else {
        fn = fallback_CoIncrementMTAUsage;
    }
    __imp_CoIncrementMTAUsage = fn;
    return fn(pCookie);
}

static HRESULT WINAPI fallback_CoDecrementMTAUsage(CO_MTA_USAGE_COOKIE cookie) {
    (void)cookie;
    return S_OK;
}

static HRESULT WINAPI resolve_CoDecrementMTAUsage(CO_MTA_USAGE_COOKIE cookie) {
    HMODULE hMod = get_com_module();
    FARPROC proc = hMod ? GetProcAddress(hMod, "CoDecrementMTAUsage") : NULL;
    PFN_CoDecrementMTAUsage fn;
    if (proc) {
        fn = (PFN_CoDecrementMTAUsage)(void *)proc;
    } else {
        fn = fallback_CoDecrementMTAUsage;
    }
    __imp_CoDecrementMTAUsage = fn;
    return fn(cookie);
}

static HRESULT WINAPI fallback_RoGetAgileReference(int options, const void *riid, void *punk, void **ppagilereference) {
    (void)options;
    (void)riid;
    (void)punk;
    if (ppagilereference) {
        *ppagilereference = NULL;
    }
    return E_NOTIMPL;
}

static HRESULT WINAPI resolve_RoGetAgileReference(int options, const void *riid, void *punk, void **ppagilereference) {
    HMODULE hMod = get_com_module();
    FARPROC proc = hMod ? GetProcAddress(hMod, "RoGetAgileReference") : NULL;
    PFN_RoGetAgileReference fn;
    if (proc) {
        fn = (PFN_RoGetAgileReference)(void *)proc;
    } else {
        fn = fallback_RoGetAgileReference;
    }
    __imp_RoGetAgileReference = fn;
    return fn(options, riid, punk, ppagilereference);
}

static HRESULT WINAPI fallback_RoGetActivationFactory(void *activatableClassId, const void *iid, void **factory) {
    (void)activatableClassId;
    (void)iid;
    if (factory) {
        *factory = NULL;
    }
    return REGDB_E_CLASSNOTREG;
}

static HRESULT WINAPI resolve_RoGetActivationFactory(void *activatableClassId, const void *iid, void **factory) {
    HMODULE hMod = get_com_module();
    FARPROC proc = hMod ? GetProcAddress(hMod, "RoGetActivationFactory") : NULL;
    PFN_RoGetActivationFactory fn;
    if (proc) {
        fn = (PFN_RoGetActivationFactory)(void *)proc;
    } else {
        fn = fallback_RoGetActivationFactory;
    }
    __imp_RoGetActivationFactory = fn;
    return fn(activatableClassId, iid, factory);
}

static HRESULT WINAPI fallback_RoInitialize(int initType) {
    (void)initType;
    HMODULE hMod = get_com_module();
    if (hMod) {
        typedef HRESULT (WINAPI *PFN_CoInitEx)(LPVOID, DWORD);
        PFN_CoInitEx fn = (PFN_CoInitEx)(void *)GetProcAddress(hMod, "CoInitializeEx");
        if (fn) {
            return fn(NULL, 0 /* COINIT_MULTITHREADED */);
        }
    }
    return S_OK;
}

static HRESULT WINAPI resolve_RoInitialize(int initType) {
    HMODULE hMod = get_com_module();
    FARPROC proc = hMod ? GetProcAddress(hMod, "RoInitialize") : NULL;
    PFN_RoInitialize fn;
    if (proc) {
        fn = (PFN_RoInitialize)(void *)proc;
    } else {
        fn = fallback_RoInitialize;
    }
    __imp_RoInitialize = fn;
    return fn(initType);
}

static void WINAPI fallback_RoUninitialize(void) {
    HMODULE hMod = get_com_module();
    if (hMod) {
        typedef void (WINAPI *PFN_CoUninit)(void);
        PFN_CoUninit fn = (PFN_CoUninit)(void *)GetProcAddress(hMod, "CoUninitialize");
        if (fn) {
            fn();
        }
    }
}

static void WINAPI resolve_RoUninitialize(void) {
    HMODULE hMod = get_com_module();
    FARPROC proc = hMod ? GetProcAddress(hMod, "RoUninitialize") : NULL;
    PFN_RoUninitialize fn;
    if (proc) {
        fn = (PFN_RoUninitialize)(void *)proc;
    } else {
        fn = fallback_RoUninitialize;
    }
    __imp_RoUninitialize = fn;
    fn();
}

static HRESULT WINAPI fallback_RoActivateInstance(void *activatableClassId, void **instance) {
    (void)activatableClassId;
    if (instance) {
        *instance = NULL;
    }
    return E_NOTIMPL;
}

static HRESULT WINAPI resolve_RoActivateInstance(void *activatableClassId, void **instance) {
    HMODULE hMod = get_com_module();
    FARPROC proc = hMod ? GetProcAddress(hMod, "RoActivateInstance") : NULL;
    PFN_RoActivateInstance fn;
    if (proc) {
        fn = (PFN_RoActivateInstance)(void *)proc;
    } else {
        fn = fallback_RoActivateInstance;
    }
    __imp_RoActivateInstance = fn;
    return fn(activatableClassId, instance);
}

static HMODULE get_ws2_module(void) {
    static HMODULE s_hWs2 = NULL;
    if (!s_hWs2) {
        s_hWs2 = GetModuleHandleA("ws2_32.dll");
        if (!s_hWs2) {
            s_hWs2 = LoadLibraryA("ws2_32.dll");
        }
    }
    return s_hWs2;
}

static int WINAPI fallback_GetHostNameW(WCHAR *name, int namelen) {
    if (!name || namelen <= 0) {
        SetLastError(10014L); // WSAEFAULT
        return -1;
    }
    HMODULE hWs2 = get_ws2_module();
    if (!hWs2) {
        return -1;
    }
    typedef int (WINAPI *PFN_gethostname)(char *name, int namelen);
    PFN_gethostname p_gethostname = (PFN_gethostname)(void *)GetProcAddress(hWs2, "gethostname");
    if (!p_gethostname) {
        return -1;
    }
    char ansi[256];
    int res = p_gethostname(ansi, sizeof(ansi));
    if (res != 0) {
        return res;
    }
    int wide_len = MultiByteToWideChar(CP_ACP, 0, ansi, -1, name, namelen);
    if (wide_len == 0) {
        return -1;
    }
    return 0;
}

static int WINAPI resolve_GetHostNameW(WCHAR *name, int namelen) {
    HMODULE hWs2 = get_ws2_module();
    FARPROC proc = hWs2 ? GetProcAddress(hWs2, "GetHostNameW") : NULL;
    PFN_GetHostNameW fn;
    if (proc) {
        fn = (PFN_GetHostNameW)(void *)proc;
    } else {
        fn = fallback_GetHostNameW;
    }
    __imp_GetHostNameW = fn;
    return fn(name, namelen);
}

static HMODULE get_winrt_error_module(void) {
    static HMODULE s_hWinRt = NULL;
    if (!s_hWinRt) {
        s_hWinRt = GetModuleHandleA("api-ms-win-core-winrt-error-l1-1-0.dll");
        if (!s_hWinRt) {
            s_hWinRt = GetModuleHandleA("combase.dll");
        }
        if (!s_hWinRt) {
            s_hWinRt = LoadLibraryA("api-ms-win-core-winrt-error-l1-1-0.dll");
        }
        if (!s_hWinRt) {
            s_hWinRt = LoadLibraryA("combase.dll");
        }
    }
    return s_hWinRt;
}

static BOOL WINAPI fallback_RoOriginateErrorW(HRESULT error, UINT cchMax, const WCHAR *message) {
    (void)error;
    (void)cchMax;
    (void)message;
    return FALSE;
}

static BOOL WINAPI resolve_RoOriginateErrorW(HRESULT error, UINT cchMax, const WCHAR *message) {
    HMODULE hMod = get_winrt_error_module();
    FARPROC proc = hMod ? GetProcAddress(hMod, "RoOriginateErrorW") : NULL;
    PFN_RoOriginateErrorW fn;
    if (proc) {
        fn = (PFN_RoOriginateErrorW)(void *)proc;
    } else {
        fn = fallback_RoOriginateErrorW;
    }
    __imp_RoOriginateErrorW = fn;
    return fn(error, cchMax, message);
}

static BOOL WINAPI fallback_RoOriginateError(HRESULT error, void *message) {
    (void)error;
    (void)message;
    return FALSE;
}

static BOOL WINAPI resolve_RoOriginateError(HRESULT error, void *message) {
    HMODULE hMod = get_winrt_error_module();
    FARPROC proc = hMod ? GetProcAddress(hMod, "RoOriginateError") : NULL;
    PFN_RoOriginateError fn;
    if (proc) {
        fn = (PFN_RoOriginateError)(void *)proc;
    } else {
        fn = fallback_RoOriginateError;
    }
    __imp_RoOriginateError = fn;
    return fn(error, message);
}
