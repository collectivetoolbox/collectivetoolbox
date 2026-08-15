// @license magnet:?xt=urn:btih:0b31508aeb0634b347b8270c7bee4d411b5d4109&dn=agpl-3.0.txt AGPL-3.0

/**
 * Call a backend JSON RPC endpoint.
 * Serializes typed arrays to regular arrays for JSON transport, sends the request with standard
 * headers, and unwraps the resulting response.
 *
 * @param {string} endpoint The RPC endpoint URL (e.g. '/api/calculator/call' or '/api/eite/call').
 * @param {string} funcName The function name to call on the backend.
 * @param {any[]} [args=[]] The arguments for the function.
 * @returns {Promise<any>} The result returned by the backend.
 */
export async function rpcCall(endpoint, funcName, args = []) {
    const serializedArgs = args.map((arg) => {
        if (arg instanceof Uint8Array) {
            return Array.from(arg);
        }
        return arg;
    });

    const response = await fetch(endpoint, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
            "Accept": "application/json",
            "X-CollectiveToolbox-IsJsRequest": "true"
        },
        body: JSON.stringify({
            function: funcName,
            args: serializedArgs
        })
    });

    if (!response.ok) {
        const text = await response.text();
        throw new Error(`RPC call to ${endpoint} (${funcName}) failed: ${text}`);
    }

    const result = await response.json();
    return result.value;
}
