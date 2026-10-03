#include <windows.h>

HRESULT WINAPI RoGetAgileReference(int options, const void *riid, void *punk, void **ppagilereference) {
    if (ppagilereference) *ppagilereference = NULL;
    return 0x80004001; // E_NOTIMPL
}

BOOL WINAPI DllMain(HINSTANCE hinstDLL, DWORD fdwReason, LPVOID lpvReserved) {
    return TRUE;
}
