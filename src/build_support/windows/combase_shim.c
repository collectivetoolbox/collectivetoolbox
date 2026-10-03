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
    return CoInitializeEx(NULL, 0 /* COINIT_MULTITHREADED */);
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
    CoUninitialize();
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
