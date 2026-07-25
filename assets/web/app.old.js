import { makeChannel, writeMessage } from "./vendor/sync-message/index.js";
import workerRTC from './vendor/worker-webrtc/window.js';

const worker = new Worker('./vendor/worker-webrtc/worker.js', { type: "module" });
workerRTC(worker);

let makeRustWorker = function () {
    let rustWorker = new Worker('rust-worker.js', { type: "module" });

    const channel = makeChannel();

    // Send the channel to the web worker
    rustWorker.postMessage({ type: 'registerChannel', channel: channel });

    return [rustWorker, channel];
}

let makeRustListener = function (channel, whenReady, messageCallback) {
    return async function (event) {
        let message = event.data;
        console.log("Received message from Rust", message, whenReady);
        if (message.type === "call") {
            let allowedMethods = ['print_js', 'get_base_url', 'get_path', 'showView', 'log_js'];
            if (!allowedMethods.includes(message.method)) {
                console.error(`Method ${message.method} is not allowed`);
                return;
            }
            let args = JSON.parse(message.args);
            let result = await window[message.method](...args) ?? '';
            writeMessage(channel, result, message.id);
        } else if (message.type === "result") {
            // This is a response from calling the worker from JS
            messageCallback(message);
        } else if (message.type === "ready") {
            whenReady();
        } else if (message.type === "log") {
            console.log(message);
        }
    };
};


async function callRust(method, args) {
    let promise = new Promise((resolve, reject) => {
        let [rustWorker, channel] = makeRustWorker();
        let whenReady = () => {
            rustWorker.postMessage({ type: 'callFromJs', method: method, args: args });
        };
        let handleResult = (message) => {
            resolve(message);
        };
        rustWorker.addEventListener("message", makeRustListener(channel, whenReady, handleResult));
    });

    let result = await promise;

    return result.result;
}

window.startApp = async function(html) {
    window.appStarted = true;
    await callRust('start', []);
}

let checkAppStarted = () => {
    if (window.appStarted) {
        return;
    }
    navigator.serviceWorker.getRegistrations().then(registrations => {
        registrations.forEach(registration => {
            if (registration.active.scriptURL.includes("coi-serviceworker.js")) {
               startApp();
               clearInterval(interval);
            }
        });
    });
};
checkAppStarted();
let interval = setInterval(checkAppStarted, 250);






// Functions available to Rust

window.print_js = function (string) {
    document.getElementById("textFrame").textContent = string;
};
window.get_base_url = function (string) {
    let url = window.location.href;
    let base_url = url.substring(0, url.lastIndexOf('/'));
    return base_url;
};
window.get_path = async function (url) {
    let response = await fetch(`./${url}`);
    return await response.text();
};
window.showView = async function (view, parent) {
    let originalContainer = document.getElementById(parent);
    let replacementContainer = document.getElementById(parent).parentElement.appendChild(document.createElement('div'));
    replacementContainer.classList = originalContainer.classList;
    replacementContainer.classList.add('fadeIn-start');
    originalContainer.classList.add('fadeOut-start');
    let sanitized = vectostr(await sanitize_html(strtovec(view)));
    replacementContainer.innerHTML = sanitized;
    setTimeout(() => {
        replacementContainer.classList.add('fadeIn');
        originalContainer.classList.add('fadeOut');
    }, 10);
    setTimeout(() => {
        originalContainer.remove();
        replacementContainer.id = parent;
        replacementContainer.classList.remove('fadeIn-start', 'fadeIn');
        originalContainer.classList.remove('fadeOut-start', 'fadeOut');
    }, 320);
};
window.log_js = function(string) {
    console.log(string);
    document.getElementById("textLog").textContent += `${string}\n`;
}





// Other functions

function strtovec(string) {
    return new TextEncoder().encode(string);
}

function vectostr(vector) {
    return new TextDecoder().decode(vector);
}

async function sanitize_html(html) {
    return await callRust('sanitize_html', [html]);
}