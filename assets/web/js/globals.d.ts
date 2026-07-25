type CtbNoticeLevel = "debug" | "info" | "warning" | "error";
type ProxyNoticeLanguage = "en" | "ru" | "uk" | "zh";

interface CtbModalApi {
    close: () => void;
    popout: () => void;
}

interface CtbClientSettingsApi {
    get: (key: string) => string | null;
    set: (key: string, value: string) => void;
    getAll: () => Record<string, string>;
}

interface CtbGlobal {
    copyToClipboard: (text: string) => void;
    setLoading: () => void;
    unSetLoading: () => void;
    modal: CtbModalApi;
    toast: (
        message: string,
        level?: CtbNoticeLevel,
        timeoutMs?: number | null,
    ) => HTMLElement | null;
    alertBanner: (
        message: string,
        level?: CtbNoticeLevel,
        container?: string | HTMLElement | null,
        dismissible?: boolean,
    ) => HTMLElement | null;
    upgradeButtons: (root?: ParentNode) => void;
    debug: (message: string, timeoutMs?: number | null) => HTMLElement | null;
    info: (message: string, timeoutMs?: number | null) => HTMLElement | null;
    warn: (message: string, timeoutMs?: number | null) => HTMLElement | null;
    error: (message: string, timeoutMs?: number | null) => HTMLElement | null;
    alert: (message: string, title?: string) => Promise<void>;
    confirm: (message: string, title?: string) => Promise<boolean>;
    confirmHtml: (htmlMessage: string, title?: string) => Promise<boolean>;
    prompt: (message: string, title?: string, defaultValue?: string) => Promise<string | null>;
    showModal: (param: string | HTMLElement, options?: ModalOptions) => CtbModalHandle | null;
    remToPx: (rem: number) => number;
    settings: CtbClientSettingsApi;
}

interface ModalOptions {
    backdrop?: HTMLDivElement | HTMLElement;
    onClose?: () => void;
    closeOnEscape?: boolean;
    closeOnOutsideClick?: boolean;
}

interface CtbModalHandle {
    backdrop: HTMLDivElement | HTMLElement;
    content: HTMLElement;
    previousActiveElement: HTMLElement | SVGElement | null;
    placeholder?: Comment | null;
    originalDisplay: string;
    originalHidden: boolean;
    focusTrap: { destroy: () => void } | null;
    isExistingContainer: boolean;
    close: () => void;
    enableFocusTrap: () => void;
    closeOnEscape?: boolean;
    closeOnOutsideClick?: boolean;
}

declare let ctb: CtbGlobal;
declare let ctbClientSideSettings: CtbClientSettingsApi;

interface Window {
    ctb: CtbGlobal;
    ctbClientSideSettings: CtbClientSettingsApi;
}

// Provided by js_test.rs:
/**
 * Assert that the given condition is truthy.
 *
 * @param {unknown} condition
 * @param {string} [message]
 * @returns {void}
 */
declare function assert(condition: unknown, message?: string): void;

/**
 * Assert that actual strictly equals expected.
 *
 * @param {unknown} actual
 * @param {unknown} expected
 * @param {string} [message]
 * @returns {void}
 */
declare function assertSame(actual: unknown, expected: unknown, message?: string): void;

/**
 * Assert that the given function throws an error.
 *
 * @param {() => void} fn
 * @param {unknown} [expectedError]
 * @param {string} [message]
 * @returns {void}
 */
declare function assertThrows(fn: () => void, expectedError?: unknown, message?: string): void;

