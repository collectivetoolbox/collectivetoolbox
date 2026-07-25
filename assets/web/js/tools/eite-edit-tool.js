// @license magnet:?xt=urn:btih:0b31508aeb0634b347b8270c7bee4d411b5d4109&dn=agpl-3.0.txt AGPL-3.0

/** @type {string} */
let globalCachedInputState = "";

/** @type {HTMLSelectElement | null} */
let inFormat = null;
/** @type {HTMLSelectElement | null} */
let outFormat = null;
/** @type {HTMLSelectElement | null} */
let editFormat = null;
/** @type {HTMLTextAreaElement | null} */
let inputarea = null;
/** @type {HTMLElement | null} */
let notificationOverlay = null;

// Extend Window interface for custom global variables
/**
 * @typedef {Object} GlobalWindow
 * @property {string[]} dcNames
 * @property {string} editFormatValue
 */

const globWin = /** @type {Window & typeof globalThis & GlobalWindow} */ (window);

globWin.dcNames = [];
globWin.editFormatValue = "";

/**
 * Call the backend EITE JSON RPC endpoint.
 *
 * @param {string} funcName The function name to call on the backend.
 * @param {any[]} [args=[]] The arguments for the function.
 * @returns {Promise<any>} The result returned by the backend.
 */
async function eiteCall(funcName, args = []) {
    const serializedArgs = args.map(arg => {
        if (arg instanceof Uint8Array) {
            return Array.from(arg);
        }
        return arg;
    });

    const response = await fetch('/api/eite/call', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Accept': 'application/json',
            'X-CollectiveToolbox-IsJsRequest': 'true'
        },
        body: JSON.stringify({
            function: funcName,
            args: serializedArgs
        })
    });

    if (!response.ok) {
        const text = await response.text();
        throw new Error(`EITE RPC call failed: ${text}`);
    }

    const result = await response.json();
    return result.value;
}

/**
 * Retrieve the format ID by string name.
 *
 * @param {string} format The format name.
 * @returns {Promise<number>} The format ID.
 */
async function getFormatId(format) {
    return await eiteCall('getFormatId', [format]);
}

/**
 * Print a representation of a DC array.
 *
 * @param {number[]} dcArray The array of DC IDs.
 * @returns {Promise<string>} The printed representation.
 */
async function printArr(dcArray) {
    return await eiteCall('printArr', [dcArray]);
}

/**
 * Import a document from a specific format.
 *
 * @param {string} format The format name.
 * @param {Uint8Array} bytes The raw document bytes.
 * @returns {Promise<number[]>} The imported DC array.
 */
async function dcaFromFormat(format, bytes) {
    return await eiteCall('importDocument', [format, bytes]);
}

/**
 * Convert a DC array to a dcbnb fragment UTF-8 representation.
 *
 * @param {number[]} dcArray The DC array.
 * @returns {Promise<number[]>} The converted bytes.
 */
async function dcaToDcbnbFragmentUtf8(dcArray) {
    return await eiteCall('dcaToDcbnbFragmentUtf8', [dcArray]);
}

/**
 * Convert a dcbnb fragment UTF-8 representation to a DC array.
 *
 * @param {Uint8Array} bytes The dcbnb fragment bytes.
 * @returns {Promise<number[]>} The DC array.
 */
async function dcaFromDcbnbFragmentUtf8(bytes) {
    return await eiteCall('dcaFromDcbnbFragmentUtf8', [bytes]);
}

/**
 * Get the first character from dcbnb fragment bytes.
 *
 * @param {Uint8Array} bytes The fragment bytes.
 * @returns {Promise<number|null>} The first character code.
 */
async function dcbnbGetFirstChar(bytes) {
    return await eiteCall('dcbnbGetFirstChar', [bytes]);
}

/**
 * Get the last character from dcbnb fragment bytes.
 *
 * @param {Uint8Array} bytes The fragment bytes.
 * @returns {Promise<number|null>} The last character code.
 */
async function dcbnbGetLastChar(bytes) {
    return await eiteCall('dcbnbGetLastChar', [bytes]);
}

/**
 * Check if a DC ID is known.
 *
 * @param {number} dc The DC ID.
 * @returns {Promise<boolean>} True if known.
 */
async function isKnownDc(dc) {
    return await eiteCall('isKnownDc', [dc]);
}

/**
 * Get the name of a DC ID.
 *
 * @param {number} dc The DC ID.
 * @returns {Promise<string>} The DC name.
 */
async function dcGetName(dc) {
    return await eiteCall('dcGetName', [dc]);
}

/**
 * Alert an error message and throw an Error.
 *
 * @param {string} msg The error message.
 * @returns {void}
 */
function implDie(msg) {
    alert(msg);
    throw new Error(msg);
}

// Dynamic initialization run immediately on script injection
(async function init() {
    globWin.dcNames = [];
    await eiteCall('setupIfNeeded');
    globWin.dcNames = await eiteCall('dcGetColumn', ['DcData', 1]);
    const _datasetLength = await eiteCall('dcDatasetLength', ['DcData']);
    await handleSearchResultUpdate();

    // Attach event listeners to elements
    const searchDcs = /** @type {HTMLInputElement | null} */ (document.getElementById('searchDcs'));
    if (searchDcs) {
        searchDcs.addEventListener('input', function(){
            handleSearchResultUpdate();
        });
        searchDcs.addEventListener('keyup', function(ev){
            if (ev.key === "Escape") {
                clearDcFilters();
            }
        });
    }

    const dcsShowAllButton = document.getElementById('dcsShowAllButton');
    if (dcsShowAllButton) {
        dcsShowAllButton.addEventListener('click', function(){
            clearDcFilters();
        });
    }

    const importDocumentBtn = document.getElementById('ImportDocument');
    if (importDocumentBtn) {
        importDocumentBtn.onclick = function() {
            if (inputarea) {
                updateNearestDcLabel(inputarea);
            }
            openImportDialog();
        };
        importDocumentBtn.removeAttribute('disabled');
    }

    const exportDocumentBtn = document.getElementById('ExportDocument');
    if (exportDocumentBtn) {
        exportDocumentBtn.onclick = function() {
            if (inputarea) {
                updateNearestDcLabel(inputarea);
            }
            ExportDocument();
        };
        exportDocumentBtn.removeAttribute('disabled');
    }

    const runDocumentBtn = document.getElementById('RunDocument');
    if (runDocumentBtn) {
        runDocumentBtn.onclick = function() {
            if (inputarea) {
                updateNearestDcLabel(inputarea);
            }
            RunDocumentHandler();
        };
        runDocumentBtn.removeAttribute('disabled');
    }

    // Viewport Toggle Button listener
    const toggleViewportButton = document.getElementById('toggleViewportButton');
    if (toggleViewportButton) {
        toggleViewportButton.addEventListener('click', function() {
            const pageContent = document.querySelector('.page-content');
            if (pageContent) {
                pageContent.classList.toggle('full-viewport');
            }
        });
    }

    inputarea = /** @type {HTMLTextAreaElement | null} */ (document.getElementById('inputarea'));
    if (inputarea) {
        inputarea.disabled = false;
        document.addEventListener('input', function() {
            if (inputarea) {
                updateNearestDcLabel(inputarea, false);
            }
        }, false);
        document.addEventListener('keydown', function(e) {
            if (inputarea) {
                updateNearestDcLabel(inputarea, false);
            }
            globalCachedInputState = e.key;
        }, false);
        document.addEventListener('keyup', function() {
            if (inputarea) {
                updateNearestDcLabel(inputarea, false);
            }
        }, false);
        document.addEventListener('click', function() {
            if (inputarea) {
                updateNearestDcLabel(inputarea);
            }
        }, false);
        inputarea.addEventListener('input', function(event) {
            if (inputarea) {
                handleDcEditingKeystroke(event);
            }
        });
        inputarea.onkeydown = function(event) {
            if (inputarea) {
                return handleDcBackspaceOrDelKeystroke(event);
            }
            return true;
        };
    }

    inFormat = /** @type {HTMLSelectElement | null} */ (document.getElementById('inFormat'));
    if (inFormat) {
        inFormat.innerHTML = '';
        const formats = await eiteCall('listInputFormats');
        for (let i = 0; i < Object.keys(formats).length; i++) {
            const elem = document.createElement('option');
            elem.innerHTML = formats[i];
            inFormat.appendChild(elem);
        }
        inFormat.disabled = false;
    }

    outFormat = /** @type {HTMLSelectElement | null} */ (document.getElementById('outFormat'));
    if (outFormat) {
        outFormat.innerHTML = '';
        const formats = await eiteCall('listOutputFormats');
        for (let i = 0; i < Object.keys(formats).length; i++) {
            const elem = document.createElement('option');
            elem.innerHTML = formats[i];
            outFormat.appendChild(elem);
        }
        outFormat.disabled = false;
    }

    editFormat = /** @type {HTMLSelectElement | null} */ (document.getElementById('editFormat'));
    if (editFormat) {
        editFormat.innerHTML = '';
        const formats = [
            { value: 'utf8', text: 'UTF-8 (dcbasenb fragment)' },
            { value: 'integerList', text: 'integerList' }
        ];
        for (let i = 0; i < formats.length; i++) {
            const elem = document.createElement('option');
            elem.value = formats[i].value;
            elem.innerHTML = formats[i].text;
            editFormat.appendChild(elem);
        }
        globWin.editFormatValue = editFormat.value;
        editFormat.onchange = function() {
            startSpinner();
            window.setTimeout(async function() {
                const oldEditFormat = globWin.editFormatValue;
                const activeEditFormat = editFormat ? editFormat.value : "utf8";
                if (inputarea) {
                    await eiteCall('pushExportSettings', [await getFormatId('utf8'), 'variants:dcBasenb dcBasenbFragment,']);
                    const tempInputValue = await eiteCall('importAndExport', ['integerList', activeEditFormat, await getInputDoc(oldEditFormat)]);
                    await eiteCall('popExportSettings', [await getFormatId('utf8')]);
                    if (activeEditFormat === 'utf8') {
                        inputarea.value = new TextDecoder().decode(new Uint8Array(tempInputValue));
                    } else {
                        inputarea.value = await eiteCall('strFromByteArray', [tempInputValue]);
                    }
                }
                globWin.editFormatValue = activeEditFormat;
                removeSpinner(true);
            }, 500);
        };
        editFormat.disabled = false;
    }

    window.setTimeout(function() {
        const overlay = document.getElementById('overlay');
        const overlayLoadingSpinner = document.getElementById('overlayLoadingSpinner');
        if (overlay) {
            overlay.style.opacity = '0';
            overlay.style.transform = 'scale(3)';
        }
        if (overlayLoadingSpinner) {
            overlayLoadingSpinner.style.opacity = '0';
        }
        window.setTimeout(function() {
            const el = document.getElementById('overlay');
            if (el) el.remove();
        }, 1500);
    }, 500);
})();

/**
 * Clear the DC search filter.
 *
 * @returns {void}
 */
function clearDcFilters() {
    const searchDcs = /** @type {HTMLInputElement | null} */ (document.getElementById('searchDcs'));
    if (searchDcs) {
        searchDcs.value = "";
    }
    handleSearchResultUpdate();
}

/**
 * Check if editing integers format.
 *
 * @param {string} [overrideEditFormat] Optional override format name.
 * @returns {boolean} True if editing integer list.
 */
function editInts(overrideEditFormat) {
    let activeFormat = overrideEditFormat;
    if (activeFormat === undefined) {
        activeFormat = editFormat ? editFormat.value : "utf8";
    }
    return 'integerList' === activeFormat;
}

/**
 * Handle updates to search results.
 *
 * @returns {Promise<void>}
 */
async function handleSearchResultUpdate() {
    const searchDcs = /** @type {HTMLInputElement | null} */ (document.getElementById('searchDcs'));
    const dcsShowAllButton = document.getElementById('dcsShowAllButton');
    const dcSelection = document.getElementById('DcSelection');

    let searchQuery = "";
    if (searchDcs) {
        searchQuery = searchDcs.value;
    }

    let re = new RegExp('.*');
    const isFiltered = searchQuery.trim().length !== 0;
    if (isFiltered) {
        re = new RegExp(searchQuery, 'i');
    }
    if (dcsShowAllButton) {
        dcsShowAllButton.classList.toggle('hidden', !isFiltered);
    }

    const datasetLength = await eiteCall('dcDatasetLength', ['DcData']);
    Array.from(document.getElementsByClassName('dcInsertButton')).forEach(function(e) {
        const target = (e.parentElement && e.parentElement.tagName === 'CTB-BUTTON') ? e.parentElement : e;
        target.remove();
    });

    for (let i = 0; i < datasetLength; i++) {
        const dcName = globWin.dcNames[i] || "";
        if (dcName.match(re)) {
            const elem = document.createElement('button');
            elem.onclick = async function() {
                if (editInts()) {
                    editAreaInsert(i + '');
                } else {
                    const temp = await dcaToDcbnbFragmentUtf8([i]);
                    editAreaInsert(new TextDecoder().decode(new Uint8Array(temp)));
                }
            };
            elem.innerHTML = '<span class="dc-name">' + dcName + '</span><small class="dc-number">(' + i + ')</small>';
            elem.className = 'dcInsertButton btn';
            if (dcSelection) {
                dcSelection.appendChild(elem);
            }
        }
    }
}

/**
 * Handle keyboard input events on DC editing.
 *
 * @param {Event} _event The keyboard event.
 * @returns {void}
 */
function handleDcEditingKeystroke(_event) {
    if (editInts()) {
        if (globalCachedInputState.length === 1) {
            if (globalCachedInputState !== " " && isNaN(parseInt(globalCachedInputState))) {
                if (inputarea && inputarea.value.includes(globalCachedInputState)) {
                    (async function(elem, char) {
                        const start = elem.selectionStart;
                        const end = elem.selectionEnd;
                        elem.value = elem.value.replace(char, '');
                        elem.selectionStart = start - 1;
                        elem.selectionEnd = end - 1;
                        typeInTextareaSpaced(elem, await printArr(await dcaFromFormat('utf8', new TextEncoder().encode(char))));
                    })(inputarea, globalCachedInputState);
                }
            }
        }
    }
}

/**
 * Handle backspace/delete keystrokes on DC input.
 *
 * @param {KeyboardEvent} event The keyboard event.
 * @returns {boolean} True to continue event propagation.
 */
function handleDcBackspaceOrDelKeystroke(event) {
    if (editInts()) {
        const key = event.keyCode || event.charCode;
        if (key === 8 || key === 46) {
            const el = /** @type {HTMLTextAreaElement | null} */ (document.getElementById('inputarea'));
            if (el) {
                const start = el.selectionStart;
                const end = el.selectionEnd;
                const text = el.value;
                if (start === end) {
                    if (key === 8) {
                        const before = text.substring(0, start);
                        const after = text.substring(end, text.length);
                        const words = before.trim().split(' ');
                        const len = words[words.length - 1].length;
                        el.value = before.trim().substring(0, before.trim().length - len) + ' ' + after;
                        el.selectionStart = el.selectionEnd = start - len;
                    } else if (key === 46) {
                        const before = text.substring(0, start);
                        const after = text.substring(end, text.length).trim();
                        const words = after.split(' ');
                        const len = words[0].length;
                        el.value = before + ' ' + after.substring(len).trim();
                        el.selectionStart = el.selectionEnd = start;
                    }
                    return false;
                }
            }
        }
    }
    return true;
}

/**
 * Start the loading spinner overlay.
 *
 * @returns {void}
 */
function startSpinner() {
    const eiteDocumentRoot = document.getElementById('eiteDocumentRoot');
    const documentRootLoadingSpinnerTemplate = /** @type {HTMLTemplateElement | null} */ (
        document.getElementById('documentRootLoadingSpinnerTemplate')
    );
    if (eiteDocumentRoot && documentRootLoadingSpinnerTemplate) {
        const elem = document.importNode(documentRootLoadingSpinnerTemplate.content, true);
        eiteDocumentRoot.appendChild(elem);
    }
}

/**
 * Remove the loading spinner overlay.
 *
 * @param {boolean} [clear=false] Whether to clear document root spinner.
 * @returns {Promise<void>}
 */
async function removeSpinner(clear = false) {
    const documentRootOverlay = document.getElementsByClassName('documentRootOverlay');
    while (documentRootOverlay.length > 0) {
        const parent = documentRootOverlay[0].parentNode;
        if (parent) {
            parent.removeChild(documentRootOverlay[0]);
        }
    }
    if (clear) {
        const res = await eiteCall('importAndExport', ['integerList', 'utf8', await getInputDoc()]);
        const eiteDocumentRoot = document.getElementById('eiteDocumentRoot');
        if (eiteDocumentRoot) {
            eiteDocumentRoot.innerHTML = new TextDecoder().decode(new Uint8Array(res));
        }
    }
}

/**
 * Insert text at the current cursor position in the edit area.
 *
 * @param {string} text The text to insert.
 * @returns {void}
 */
function editAreaInsert(text) {
    const el = /** @type {HTMLTextAreaElement | null} */ (document.getElementById('inputarea'));
    if (el) {
        if (editInts()) {
            typeInTextareaSpaced(el, text);
        } else {
            typeInTextarea(el, text);
        }
        updateNearestDcLabel(el);
    }
}

/**
 * Set the label content showing the DC name.
 *
 * @param {string|number} id The DC ID.
 * @param {string} text The DC name.
 * @returns {void}
 */
function setNearestDcLabel(id, text) {
    const currentDcLabel = document.getElementById('currentDcLabel');
    const currentDcId = document.getElementById('currentDcId');
    if (currentDcLabel) {
        currentDcLabel.innerHTML = text;
    }
    if (currentDcId) {
        currentDcId.innerHTML = id + '';
    }
}

/**
 * Autoformat the text in the input area.
 *
 * @param {HTMLTextAreaElement} el The textarea element.
 * @returns {void}
 */
function autoformatInputArea(el) {
    if (editInts()) {
        const start = el.selectionStart;
        const end = el.selectionEnd;
        const oldValue = el.value;
        const temp = oldValue.replace(/\s+/g, ' ');
        el.value = temp;
        if (oldValue !== el.value) {
            el.selectionStart = el.selectionEnd = start;
        } else {
            el.selectionStart = start;
            el.selectionEnd = end;
        }
    }
}

/**
 * Trigger update of the nearest DC label.
 *
 * @param {HTMLTextAreaElement} el The textarea element.
 * @param {boolean} [autoformat=true] Whether to autoformat.
 * @returns {void}
 */
function updateNearestDcLabel(el, autoformat = true) {
    if (autoformat) {
        autoformatInputArea(el);
    } else {
        setTimeout(function(){ autoformatInputArea(el); }, 750);
    }
    updateNearestDcLabelInner(el);
}

/**
 * Inner logic to update the nearest DC label based on cursor position.
 *
 * @param {HTMLTextAreaElement} el The textarea element.
 * @returns {Promise<void>}
 */
async function updateNearestDcLabelInner(el) {
    const start = el.selectionStart;
    const end = el.selectionEnd;
    if (start !== end) {
        setNearestDcLabel('', '');
        return;
    }
    const text = el.value;
    let before = text.substring(0, start);
    let after = text.substring(end, text.length);
    let currentDc = 0;
    if (editInts()) {
        const afterIdx = after.indexOf(' ');
        if (afterIdx !== -1) {
            after = after.substring(0, afterIdx);
        }
        before = before + after;
        const parts = before.trim().split(' ');
        currentDc = parseInt(parts[parts.length - 1], 10);
    } else {
        const lastCharBytes = await dcbnbGetLastChar(new TextEncoder().encode(before));
        let lastChar = "";
        if (lastCharBytes !== null) {
            lastChar = new TextDecoder().decode(new Uint8Array(lastCharBytes));
        }
        if (lastChar.length === 0) {
            const firstCharBytes = await dcbnbGetFirstChar(new TextEncoder().encode(after));
            if (firstCharBytes !== null) {
                lastChar = new TextDecoder().decode(new Uint8Array(firstCharBytes));
            }
        }
        if (lastChar !== undefined && lastChar.length > 0) {
            const fragment = await dcaFromDcbnbFragmentUtf8(new TextEncoder().encode(lastChar));
            currentDc = fragment[0];
        }
    }
    if (isNaN(currentDc) || (! await isKnownDc(currentDc))) {
        setNearestDcLabel('', '');
        return;
    }
    setNearestDcLabel(currentDc, await dcGetName(currentDc));
}

/**
 * Helper to insert text directly at cursor in textarea.
 *
 * @param {HTMLTextAreaElement} el The textarea element.
 * @param {string} newText The text to type.
 * @returns {void}
 */
function typeInTextarea(el, newText) {
    const start = el.selectionStart;
    const end = el.selectionEnd;
    const text = el.value;
    const before = text.substring(0, start);
    const after = text.substring(end, text.length);
    el.value = (before + newText + after);
    el.selectionStart = el.selectionEnd = start + newText.length;
    el.focus();
}

/**
 * Helper to insert text with spacing.
 *
 * @param {HTMLTextAreaElement} el The textarea element.
 * @param {string} newText The text to type.
 * @returns {void}
 */
function typeInTextareaSpaced(el, newText) {
    const start = el.selectionStart;
    const end = el.selectionEnd;
    const text = el.value;
    const before = text.substring(0, start);
    const after = text.substring(end, text.length);
    let spacedText = newText;
    if (before.substring(before.length - 1) === ' ' || before.substring(before.length - 1) === '') {
        spacedText = spacedText + ' ';
        el.value = (before + spacedText + after);
    } else {
        spacedText = ' ' + spacedText;
        el.value = (before + spacedText + after);
    }
    el.selectionStart = el.selectionEnd = start + spacedText.length;
    el.focus();
}

/**
 * Get the current document content as raw bytes.
 *
 * @param {string} [overrideEditFormat] Optional override format name.
 * @returns {Promise<Uint8Array>} The raw document bytes.
 */
async function getInputDoc(overrideEditFormat) {
    let res;
    const activeInputArea = /** @type {HTMLTextAreaElement | null} */ (document.getElementById('inputarea'));
    const inputVal = activeInputArea ? activeInputArea.value : "";
    if (editInts(overrideEditFormat)) {
        res = inputVal;
    } else {
        const encoded = new TextEncoder().encode(inputVal);
        await eiteCall('pushImportSettings', [await getFormatId('utf8'), 'variants:dcBasenb dcBasenbFragment,']);
        res = await eiteCall('printArr', [await eiteCall('importDocument', ['utf8', encoded])]);
        await eiteCall('popImportSettings', [await getFormatId('utf8')]);
    }
    return await eiteCall('strToByteArray', [res]);
}

/**
 * Run the current document computation.
 *
 * @param {function} [callback] Optional callback on complete.
 * @returns {void}
 */
function RunDocumentHandler(callback) {
    startSpinner();
    window.setTimeout(async function() {
        await eiteCall('runDocument', [await eiteCall('importDocument', ['Dc Integer List', await getInputDoc()])]);
        if (callback !== undefined) {
            window.setTimeout(async function() {
                await callback();
                await removeSpinner();
            }, 500);
        } else {
            await removeSpinner();
        }
    }, 500);
}

/**
 * Close the import dialog modal.
 *
 * @returns {void}
 */
function closeImportDialog() {
    if (notificationOverlay) {
        notificationOverlay.removeEventListener('keyup', importDialogEscapeListener);
    }
    const e = document.getElementsByClassName('importDialog');
    let i = 0;
    while (e.length > 0 && i < e.length) {
        if (notificationOverlay) {
            notificationOverlay.style.opacity = '0';
        }
        const dialog = /** @type {HTMLElement} */ (e[i]);
        dialog.style.opacity = '0';
        dialog.style.transform = 'translate(-50%, -50%) scale(0.75)';
        i = i + 1;
    }
    setTimeout(function() {
        const dialogs = document.getElementsByClassName('importDialog');
        while (dialogs.length > 0) {
            const parent = dialogs[0].parentNode;
            if (parent) parent.removeChild(dialogs[0]);
        }
        if (notificationOverlay) {
            notificationOverlay.style.display = 'none';
        }
    }, 750);
}

/**
 * Close the alert dialog modal.
 *
 * @returns {void}
 */
function closeAlertDialog() {
    if (notificationOverlay) {
        notificationOverlay.removeEventListener('keyup', alertDialogEscapeListener);
    }
    const e = document.getElementsByClassName('importDialog');
    let i = 0;
    while (e.length > 0 && i < e.length) {
        if (notificationOverlay) {
            notificationOverlay.style.opacity = '0';
        }
        const dialog = /** @type {HTMLElement} */ (e[i]);
        dialog.style.opacity = '0';
        dialog.style.transform = 'translate(-50%, -50%) scale(0.75)';
        i = i + 1;
    }
    setTimeout(function() {
        const dialogs = document.getElementsByClassName('importDialog');
        while (dialogs.length > 0) {
            const parent = dialogs[0].parentNode;
            if (parent) parent.removeChild(dialogs[0]);
        }
        if (notificationOverlay) {
            notificationOverlay.style.display = 'none';
        }
    }, 750);
}

/**
 * Escape key listener for import dialog.
 *
 * @param {KeyboardEvent} event The keyboard event.
 * @returns {void}
 */
function importDialogEscapeListener(event) {
    if (event.key === 'Escape') {
        closeImportDialog();
    }
}

/**
 * Escape key listener for alert dialog.
 *
 * @param {KeyboardEvent} event The keyboard event.
 * @returns {void}
 */
function alertDialogEscapeListener(event) {
    if (event.key === 'Escape') {
        closeAlertDialog();
    }
}

/**
 * Open the import dialog modal.
 *
 * @returns {void}
 */
function openImportDialog() {
    const importDialogTemplate = /** @type {HTMLTemplateElement | null} */ (
        document.getElementById('importDialogTemplate')
    );
    if (!importDialogTemplate) return;

    const elem = document.importNode(importDialogTemplate.content, true);
    const dialogElem = /** @type {HTMLElement | null} */ (
        elem.querySelector('.importDialog') || elem.firstElementChild
    );
    if (!dialogElem) return;

    const importFromFileBtn = /** @type {HTMLElement | null} */ (dialogElem.querySelector('.importFromFileBtn'));
    if (importFromFileBtn) {
        importFromFileBtn.onclick = function() { importDocumentFromFile(); };
    }

    const importFromUrlBtn = /** @type {HTMLElement | null} */ (dialogElem.querySelector('.importFromUrlBtn'));
    if (importFromUrlBtn) {
        importFromUrlBtn.onclick = function() {
            importDocumentFromURL(prompt('What URL do you want?'));
        };
    }

    const closeImportDiaBtn = /** @type {HTMLElement | null} */ (dialogElem.querySelector('.closeImportDiaBtn'));
    if (closeImportDiaBtn) {
        closeImportDiaBtn.onclick = function() { closeImportDialog(); };
    }

    document.addEventListener('keyup', importDialogEscapeListener);
    notificationOverlay = document.getElementById('notificationOverlay');
    if (notificationOverlay) {
        notificationOverlay.addEventListener('click', function() {
            closeImportDialog();
        });
        notificationOverlay.style.display = 'block';
        notificationOverlay.style.opacity = '1';
    }
    document.body.appendChild(dialogElem);
}

/**
 * Open the alert dialog modal.
 *
 * @param {string} message The message to show in the alert.
 * @returns {void}
 */
function _openAlertDialog(message) {
    const alertDialogTemplate = /** @type {HTMLTemplateElement | null} */ (
        document.getElementById('alertDialogTemplate')
    );
    if (!alertDialogTemplate) return;

    const elem = document.importNode(alertDialogTemplate.content, true);
    const dialogElem = /** @type {HTMLElement | null} */ (
        elem.querySelector('.importDialog') || elem.firstElementChild
    );
    if (!dialogElem) return;

    const closeAlertDiaBtn = /** @type {HTMLElement | null} */ (dialogElem.querySelector('.closeAlertDiaBtn'));
    if (closeAlertDiaBtn) {
        closeAlertDiaBtn.onclick = function() { closeAlertDialog(); };
    }

    const messageRegion = dialogElem.querySelector('.alertDialogMessageRegion');
    if (messageRegion) {
        messageRegion.innerHTML = message;
    }

    document.addEventListener('keyup', alertDialogEscapeListener);
    notificationOverlay = document.getElementById('notificationOverlay');
    if (notificationOverlay) {
        notificationOverlay.addEventListener('click', function() {
            closeAlertDialog();
        });
        notificationOverlay.style.display = 'block';
        notificationOverlay.style.opacity = '1';
    }
    document.body.appendChild(dialogElem);
}

/**
 * Import a document from a selected file.
 *
 * @returns {void}
 */
function importDocumentFromFile() {
    startSpinner();
    closeImportDialog();
    window.setTimeout(async function() {
        const activeInFormat = inFormat ? inFormat.value : "utf8";
        if (!await eiteCall('isSupportedInputFormat', [activeInFormat])) {
            await implDie(activeInFormat + ' is not a supported input format!');
        }
        const picker = /** @type {HTMLInputElement | null} */ (document.getElementById('filepicker'));
        if (picker && picker.files) {
            picker.click();
            const file = picker.files[0];
            if (file !== undefined && file !== null) {
                const fr = new FileReader();
                await new Promise(resolve => {
                    fr.onload = function() {
                        resolve(undefined);
                    };
                    fr.readAsArrayBuffer(file);
                });
                if (fr.result instanceof ArrayBuffer) {
                    const u8 = new Uint8Array(fr.result);
                    if (inputarea) {
                        inputarea.value = await eiteCall('strFromByteArray', [await eiteCall('importAndExport', [activeInFormat, 'integerList', u8])]);
                    }
                }
            }
        }
        removeSpinner(true);
    }, 500);
}

/**
 * Import a document from a URL.
 *
 * @param {string|null} path The URL path.
 * @returns {void}
 */
function importDocumentFromURL(path) {
    if (path === null) {
        return;
    }
    if (path === undefined) {
        closeImportDialog();
        return;
    }
    startSpinner();
    closeImportDialog();
    window.setTimeout(async function() {
        const activeInFormat = inFormat ? inFormat.value : "utf8";
        if (!await eiteCall('isSupportedInputFormat', [activeInFormat])) {
            await implDie(activeInFormat + ' is not a supported input format!');
        }
        try {
            if (inputarea) {
                inputarea.value = await eiteCall('strFromByteArray', [await eiteCall('importAndExport', [activeInFormat, 'integerList', await eiteCall('getFileFromPath', [path])])]);
            }
        } catch (_e) {
            removeSpinner(true);
        }
        removeSpinner(true);
    }, 500);
}

/**
 * Notify the user about an exported file.
 *
 * @param {string} name The exported file name.
 * @returns {void}
 */
function exportNotify(name) {
    const exportNotifyTemplate = /** @type {HTMLTemplateElement | null} */ (
        document.getElementById('exportNotifyTemplate')
    );
    if (!exportNotifyTemplate) return;

    const elem = document.importNode(exportNotifyTemplate.content, true);
    const firstChild = /** @type {HTMLElement} */ (elem.firstChild);
    firstChild.innerHTML = name;
    firstChild.id = 'exportNotifyTempId';

    notificationOverlay = document.getElementById('notificationOverlay');
    if (notificationOverlay) {
        notificationOverlay.style.display = 'block';
        notificationOverlay.style.opacity = '1';
    }
    document.body.appendChild(firstChild);

    const elemAppended = document.getElementById('exportNotifyTempId');
    if (elemAppended) {
        elemAppended.removeAttribute('id');
    }

    setTimeout(function() {
        const e = document.getElementsByClassName('exportNotification');
        let i = 0;
        while (e.length > 0 && i < e.length) {
            if (notificationOverlay) {
                notificationOverlay.style.opacity = '0';
            }
            const el = /** @type {HTMLElement} */ (e[0]);
            el.style.opacity = '0';
            el.style.transform = 'translate(-50%, -50%) scale(0.75)';
            i = i + 1;
        }
        setTimeout(function() {
            const list = document.getElementsByClassName('exportNotification');
            while (list.length > 0) {
                const parent = list[0].parentNode;
                if (parent) parent.removeChild(list[0]);
            }
            if (notificationOverlay) {
                notificationOverlay.style.display = 'none';
            }
        }, 2000);
    }, 1250);
}

/**
 * Export the current document.
 *
 * @returns {void}
 */
function ExportDocument() {
    startSpinner();
    window.setTimeout(async function() {
        const activeOutFormat = outFormat ? outFormat.value : "utf8";
        if (!await eiteCall('isSupportedOutputFormat', [activeOutFormat])) {
            await implDie(activeOutFormat + ' is not a supported output format!');
        }
        const exported = Uint8Array.from(await eiteCall('importAndExport', ['sems', activeOutFormat, await getInputDoc()]));
        const blob = new Blob([exported], { type: 'application/octet-stream' });
        const link = document.createElement('a');
        link.href = window.URL.createObjectURL(blob);
        const date = new Date();
        const outName = 'Export-' + date.getUTCFullYear() + 'm' + (date.getUTCMonth() + 1) + 'd' + date.getUTCDate() + '-' + date.getUTCHours() + '-' + date.getUTCMinutes() + '-' + date.getUTCSeconds() + '-' + date.getUTCMilliseconds() + '-' + date.getTimezoneOffset() + '.' + await eiteCall('getExportExtension', [activeOutFormat]);
        exportNotify(outName);
        link.download = outName;
        link.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true, view: window }));
        removeSpinner(true);
    }, 500);
}

// @license-end
