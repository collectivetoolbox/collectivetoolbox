/**
 * v86 Emulator WebUI Integration
 *
 * Handles v86 emulator instantiation, state loading/saving, disk image
 * mounting, canvas/terminal layout switching, and statistics monitoring.
 */

// @ts-nocheck: Third-party v86 module types are dynamically loaded at runtime
import { V86 } from "/vendor/v86/src/browser/starter.js?v=1";

/**
 * Trigger a browser file download from raw binary data or string.
 *
 * @param {BlobPart} data Binary data or Blob to download.
 * @param {string} filename Name for the downloaded file.
 * @returns {void}
 */
function dump_file(data, filename) {
    const a = document.createElement("a");
    a.download = filename;
    a.href = window.URL.createObjectURL(new Blob([data]));
    a.dataset.downloadurl = "application/octet-stream:" + a.download + ":" + a.href;
    a.click();
}

/**
 * Convert a Uint8Array or byte array into a hex string for packet dumps.
 *
 * @param {Uint8Array | number[]} data Array of bytes.
 * @returns {string} Space-separated hex byte dump string.
 */
function hex_dump(data) {
    return Array.from(data).map(b => b.toString(16).padStart(2, "0")).join(" ");
}

/**
 * Format timestamp in seconds to Mm SSs string.
 *
 * @param {number} s Time in seconds.
 * @returns {string} Formatted timestamp string.
 */
function format_timestamp(s) {
    const min = Math.floor(s / 60);
    const sec = s % 60;
    return min + "m " + (sec < 10 ? "0" : "") + sec + "s";
}

/**
 * Open a native file picker dialog and resolve selected files.
 *
 * @param {boolean} multiple Whether to allow selecting multiple files.
 * @returns {Promise<FileList | File[]>} Promise resolving to file array.
 */
function pick_file(multiple) {
    return new Promise(resolve => {
        const file_input = document.createElement("input");
        file_input.type = "file";
        file_input.multiple = multiple;
        file_input.onchange = function() { resolve(file_input.files); };
        file_input.oncancel = function() { resolve([]); };
        file_input.click();
    });
}

/**
 * Initialize the v86 emulator, layout containers, and UI controls.
 *
 * @returns {void}
 */
function initV86() {
    const containerEl = document.getElementById("v86_container");
    const selectEl = document.getElementById("os-profile-select");
    const profile = (containerEl && containerEl.dataset.profile) || (selectEl && selectEl.value) || "guix";

    const imageConfig = {
        guix: {
            baseurl: "/vendor/v86_images/guix/guix-rootfs-flat/",
            basefs: "/vendor/v86_images/guix/guix-fs.json",
            cmdline: "rw root=host9p rootfstype=9p rootflags=trans=virtio,cache=loose modules=virtio_pci tsc=reliable vga=0x344 video=vesafb:ypan,vremap:8 console=ttyS0 console=tty0 init=/init"
        },
        arch: {
            baseurl: "/vendor/v86_images/arch/",
            state: "/vendor/v86_images/arch/arch_state-v3.bin.zst"
        }
    };

    const cfg = imageConfig[profile] || imageConfig.guix;

    const v86Options = {
        wasm_path: "/vendor/v86/build/v86.wasm",
        memory_size: 512 * 1024 * 1024,
        vga_memory_size: 8 * 1024 * 1024,
        screen_container: document.getElementById("screen_container"),
        bios: { url: "/vendor/v86/bios/seabios.bin" },
        vga_bios: { url: "/vendor/v86/bios/vgabios.bin" },
        net_device_type: "virtio",
        filesystem: cfg.basefs ? {
            baseurl: cfg.baseurl,
            basefs: cfg.basefs
        } : {
            baseurl: cfg.baseurl
        },
        autostart: true
    };

    if (cfg.state) {
        delete v86Options.net_device_type;
        v86Options.initial_state = { url: cfg.state };
    } else {
        v86Options.bzimage_initrd_from_filesystem = true;
        v86Options.cmdline = cfg.cmdline;
        if (profile === "guix") {
            v86Options.initrd = { url: "/vendor/v86_images/guix/guix_posix_initrd.cpio.gz" };
        }
    }

    const emulator = window.emulator = new V86(v86Options);

    const screenContainer = document.getElementById("screen_container");
    if (screenContainer) screenContainer.style.display = "block";
    const runtimeOptions = document.getElementById("runtime_options");
    if (runtimeOptions) runtimeOptions.style.display = "block";
    const runtimeInfos = document.getElementById("runtime_infos");
    if (runtimeInfos) runtimeInfos.style.display = "block";
    const loadingEl = document.getElementById("loading");
    if (loadingEl) loadingEl.style.display = "none";

    emulator.add_listener("emulator-ready", function() {
        if (emulator.screen_adapter) {
            emulator.screen_adapter.set_mode(true);
        }
    });

    let _isRunning = true;
    const runBtn = document.getElementById("run");
    if (runBtn) {
        runBtn.onclick = function() {
            if (emulator.is_running()) {
                emulator.stop();
                runBtn.textContent = "Run";
                _isRunning = false;
            } else {
                emulator.run();
                runBtn.textContent = "Pause";
                _isRunning = true;
            }
            runBtn.blur();
        };
    }

    const resetBtn = document.getElementById("reset");
    if (resetBtn) {
        resetBtn.onclick = function() {
            emulator.restart();
            resetBtn.blur();
        };
    }

    const exitBtn = document.getElementById("exit");
    if (exitBtn) {
        exitBtn.onclick = function() {
            emulator.destroy();
            window.location.href = "/v86/" + profile;
        };
    }

    const ctrlAltDelBtn = document.getElementById("ctrlaltdel");
    if (ctrlAltDelBtn) {
        ctrlAltDelBtn.onclick = function() {
            emulator.keyboard_send_scancodes([
                0x1D, 0x38, 0x53,
                0x1D | 0x80, 0x38 | 0x80, 0x53 | 0x80
            ]);
            ctrlAltDelBtn.blur();
        };
    }

    const altTabBtn = document.getElementById("alttab");
    if (altTabBtn) {
        altTabBtn.onclick = function() {
            emulator.keyboard_send_scancodes([0x38, 0x0F]);
            setTimeout(function() {
                emulator.keyboard_send_scancodes([0x38 | 0x80, 0x0F | 0x80]);
            }, 100);
            altTabBtn.blur();
        };
    }

    const fsBtn = document.getElementById("fullscreen");
    if (fsBtn) {
        fsBtn.onclick = function() {
            emulator.screen_go_fullscreen();
        };
    }

    const lockMouseBtn = document.getElementById("lock_mouse");
    let mouse_is_enabled = true;
    const toggleMouseBtn = document.getElementById("toggle_mouse");
    if (toggleMouseBtn) {
        toggleMouseBtn.onclick = function() {
            mouse_is_enabled = !mouse_is_enabled;
            emulator.mouse_set_enabled(mouse_is_enabled);
            toggleMouseBtn.textContent = (mouse_is_enabled ? "Dis" : "En") + "able mouse";
            toggleMouseBtn.blur();
        };
    }
    if (lockMouseBtn) {
        lockMouseBtn.onclick = function() {
            if (!mouse_is_enabled && toggleMouseBtn) {
                toggleMouseBtn.click();
            }
            emulator.lock_mouse();
            lockMouseBtn.blur();
        };
    }

    const screenshotBtn = document.getElementById("take_screenshot");
    if (screenshotBtn) {
        screenshotBtn.onclick = function() {
            const image = emulator.screen_make_screenshot();
            try {
                const w = window.open("");
                if (w) w.document.write(image.outerHTML);
            } catch (_err) {
                /* Screenshot popup window blocked or ignored */
            }
            screenshotBtn.blur();
        };
    }

    const scaleInput = document.getElementById("scale");
    if (scaleInput) {
        scaleInput.onchange = function() {
            const n = parseFloat(this.value);
            if (n && n > 0) {
                emulator.screen_set_scale(n, n);
            }
        };
    }

    const saveBtn = document.getElementById("save_state");
    if (saveBtn) {
        saveBtn.onclick = async function() {
            const new_state = await emulator.save_state();
            dump_file(new_state, (profile || "v86") + "-state.bin");
            saveBtn.blur();
        };
    }

    const loadBtn = document.getElementById("load_state");
    if (loadBtn) {
        loadBtn.onclick = async function() {
            loadBtn.blur();
            const files = await pick_file(false);
            const file = files[0];
            if (!file) return;
            const was_running = emulator.is_running();
            if (was_running) await emulator.stop();
            const filereader = new FileReader();
            filereader.onload = async function(e) {
                try {
                    await emulator.restore_state(e.target.result);
                } catch (err) {
                    alert("Failed to restore state:\n" + err);
                    throw err;
                }
                if (was_running) {
                    emulator.run();
                    if (runBtn) runBtn.textContent = "Pause";
                    _isRunning = true;
                }
            };
            filereader.readAsArrayBuffer(file);
        };
    }

    const memDumpBtn = document.getElementById("memory_dump");
    if (memDumpBtn) {
        memDumpBtn.onclick = function() {
            if (emulator.v86 && emulator.v86.cpu && emulator.v86.cpu.mem8) {
                const mem8 = emulator.v86.cpu.mem8;
                dump_file(new Uint8Array(mem8.buffer, mem8.byteOffset, mem8.length), "v86memory.bin");
            }
            memDumpBtn.blur();
        };
    }

    /**
     * Download binary buffer for a given emulator disk device.
     *
     * @param {object} disk Virtual disk device instance.
     * @param {string} default_name Fallback filename.
     * @returns {void}
     */
    function download_disk(disk, default_name) {
        if (!disk) return;
        const buffer = disk.buffer;
        if (!buffer) return;
        const filename = (buffer.file && buffer.file.name) || default_name;
        if (buffer.get_as_file) {
            const file = buffer.get_as_file(filename);
            dump_file(file, filename);
        } else if (buffer.get_buffer) {
            buffer.get_buffer(function(b) {
                if (b) dump_file(b, filename);
                else alert("The file could not be loaded.");
            });
        } else {
            dump_file(buffer, filename);
        }
    }

    /**
     * Configure event listener on a disk image download button.
     *
     * @param {string} type Drive type name ("fda", "hda", etc.).
     * @param {function(): object} get_device_fn Function returning the device.
     * @param {string} default_filename Default filename suffix.
     * @returns {void}
     */
    function setup_disk_button(type, get_device_fn, default_filename) {
        const elem = document.getElementById("get_" + type + "_image");
        if (elem) {
            elem.onclick = function() {
                const dev = get_device_fn();
                download_disk(dev, profile + "-" + type + default_filename);
                elem.blur();
            };
        }
    }
    setup_disk_button("fda", () => emulator.v86?.cpu?.devices?.fdc?.drives[0], ".img");
    setup_disk_button("fdb", () => emulator.v86?.cpu?.devices?.fdc?.drives[1], ".img");
    setup_disk_button("hda", () => emulator.v86?.cpu?.devices?.ide?.primary?.master, ".img");
    setup_disk_button("hdb", () => emulator.v86?.cpu?.devices?.ide?.primary?.slave, ".img");
    setup_disk_button("cdrom", () => emulator.v86?.cpu?.devices?.cdrom, ".iso");

    /**
     * Configure insert/eject button and drag-and-drop handlers for a disk drive.
     *
     * @param {string} type Drive type name.
     * @param {function(): void} eject_fn Function to eject drive.
     * @param {function(File): Promise<void>} set_fn Function to mount file.
     * @param {function(): boolean} has_disk_fn Function checking if disk mounted.
     * @returns {void}
     */
    function setup_disk_change(type, eject_fn, set_fn, has_disk_fn) {
        const btn = document.getElementById("change_" + type + "_image");
        const getBtn = document.getElementById("get_" + type + "_image");
        if (!btn) return;
        btn.ondragover = e => e.preventDefault();
        btn.ondrop = async function(e) {
            e.preventDefault();
            if (has_disk_fn && has_disk_fn()) eject_fn();
            const file = e.dataTransfer.files[0];
            if (file) {
                await set_fn(file);
                btn.textContent = "Eject " + type + " image";
                if (getBtn) getBtn.style.display = "block";
            }
        };
        btn.onclick = async function() {
            if (has_disk_fn && has_disk_fn()) {
                eject_fn();
                btn.textContent = "Insert " + type + " image";
                if (getBtn) getBtn.style.display = "none";
            } else {
                const files = await pick_file(false);
                if (files[0]) {
                    await set_fn(files[0]);
                    btn.textContent = "Eject " + type + " image";
                    if (getBtn) getBtn.style.display = "block";
                }
            }
            btn.blur();
        };
    }

    setup_disk_change(
        "fda",
        () => emulator.eject_fda && emulator.eject_fda(),
        file => emulator.set_fda({ buffer: file }),
        () => emulator.get_disk_fda && emulator.get_disk_fda()
    );
    setup_disk_change(
        "fdb",
        () => emulator.eject_fdb && emulator.eject_fdb(),
        file => emulator.set_fdb({ buffer: file }),
        () => emulator.get_disk_fdb && emulator.get_disk_fdb()
    );
    setup_disk_change(
        "cdrom",
        () => emulator.eject_cdrom && emulator.eject_cdrom(),
        file => emulator.set_cdrom({ buffer: file }),
        () => emulator.v86?.cpu?.devices?.cdrom?.has_disk && emulator.v86.cpu.devices.cdrom.has_disk()
    );

    const netCapBtn = document.getElementById("capture_network_traffic");
    if (netCapBtn) {
        let capture = [];
        let capturing = false;

        const do_capture = (direction, data) => {
            capture.push({ direction, time: performance.now() / 1000, hex_dump: hex_dump(data) });
            netCapBtn.textContent = capture.length + " packets";
        };

        netCapBtn.onclick = function() {
            if (!capturing) {
                capturing = true;
                netCapBtn.textContent = "0 packets";
                if (emulator.emulator_bus) {
                    emulator.emulator_bus.register("net0-receive", do_capture.bind(null, "I"));
                }
                emulator.add_listener("net0-send", do_capture.bind(null, "O"));
            } else {
                const capture_raw = capture.map(({ direction, time, hex_dump }) => {
                    return direction + " " + time.toFixed(6) + " " + hex_dump + "\n";
                }).join("");
                dump_file(capture_raw, "traffic.hex");
                capture = [];
                capturing = false;
                netCapBtn.textContent = "Capture network traffic";
            }
            netCapBtn.blur();
        };
    }

    const muteBtn = document.getElementById("mute");
    if (muteBtn) {
        if (emulator.speaker_adapter) {
            let is_muted = false;
            muteBtn.onclick = function() {
                if (is_muted) {
                    emulator.speaker_adapter.mixer.set_volume(1, undefined);
                    is_muted = false;
                    muteBtn.textContent = "Mute";
                } else {
                    emulator.speaker_adapter.mixer.set_volume(0, undefined);
                    is_muted = true;
                    muteBtn.textContent = "Unmute";
                }
                muteBtn.blur();
            };
        } else {
            muteBtn.style.display = "none";
        }
    }

    let theatre_mode = false;
    let theatre_ui = true;
    let theatre_zoom_to_fit = false;
    const theatreBtn = document.getElementById("toggle_theatre");
    const toggleUiBtn = document.getElementById("toggle_ui");
    const toggleZoomBtn = document.getElementById("toggle_zoom_to_fit");
    const theatreBackground = document.getElementById("theatre_background");

    function zoom_to_fit() {
        emulator.screen_set_scale(1, 1);
        const emulator_screen = screenContainer ? screenContainer.getBoundingClientRect() : { width: 1024, height: 768 };
        const n = Math.min(window.innerWidth / (emulator_screen.width || 1024), window.innerHeight / (emulator_screen.height || 768));
        emulator.screen_set_scale(n, n);
    }

    function enable_zoom_to_fit(enabled) {
        theatre_zoom_to_fit = enabled;
        if (scaleInput) scaleInput.disabled = theatre_zoom_to_fit;
        if (theatre_zoom_to_fit) {
            window.addEventListener("resize", zoom_to_fit, true);
            emulator.add_listener("screen-set-size", zoom_to_fit);
            zoom_to_fit();
        } else {
            window.removeEventListener("resize", zoom_to_fit, true);
            emulator.remove_listener("screen-set-size", zoom_to_fit);
            const n = parseFloat(scaleInput ? scaleInput.value : "1.0") || 1;
            emulator.screen_set_scale(n, n);
        }
        if (toggleZoomBtn) toggleZoomBtn.textContent = (theatre_zoom_to_fit ? "Dis" : "En") + "able zoom to fit";
    }

    function enable_theatre_ui(enabled) {
        theatre_ui = enabled;
        if (runtimeOptions) runtimeOptions.style.display = theatre_ui ? "block" : "none";
        if (runtimeInfos) runtimeInfos.style.display = theatre_ui ? "block" : "none";
        const fsPanel = document.getElementById("filesystem_panel");
        if (fsPanel) fsPanel.style.display = theatre_ui ? "block" : "none";
        if (toggleUiBtn) toggleUiBtn.textContent = (theatre_ui ? "Hide" : "Show") + " UI";
    }

    function enable_theatre_mode(enabled) {
        theatre_mode = enabled;
        if (!theatre_ui) enable_theatre_ui(true);
        if (!theatre_mode && theatre_zoom_to_fit) enable_zoom_to_fit(false);
        for (const el of ["screen_container", "runtime_options", "runtime_infos", "filesystem_panel"]) {
            const node = document.getElementById(el);
            if (node) node.classList.toggle("theatre_" + el);
        }
        if (theatreBackground) theatreBackground.style.display = theatre_mode ? "block" : "none";
        if (toggleZoomBtn) toggleZoomBtn.style.display = theatre_mode ? "inline" : "none";
        if (toggleUiBtn) toggleUiBtn.style.display = theatre_mode ? "block" : "none";
        document.body.style.overflow = theatre_mode ? "hidden" : "visible";
        if (theatreBtn) theatreBtn.textContent = (theatre_mode ? "Dis" : "En") + "able theatre mode";
    }

    if (theatreBtn) theatreBtn.onclick = () => { enable_theatre_mode(!theatre_mode); theatreBtn.blur(); };
    if (toggleUiBtn) toggleUiBtn.onclick = () => { enable_theatre_ui(!theatre_ui); toggleUiBtn.blur(); };
    if (toggleZoomBtn) toggleZoomBtn.onclick = () => { enable_zoom_to_fit(!theatre_zoom_to_fit); toggleZoomBtn.blur(); };

    const sendFileEl = document.getElementById("filesystem_send_file");
    if (sendFileEl) {
        sendFileEl.onchange = function() {
            for (const file of this.files) {
                emulator.create_file("/" + file.name, new Uint8Array(file));
            }
        };
    }
    const getFileEl = document.getElementById("filesystem_get_file");
    if (getFileEl) {
        getFileEl.onchange = async function() {
            const path = this.value;
            if (path) {
                try {
                    const data = await emulator.read_file(path);
                    const filename = path.split("/").pop() || "file";
                    dump_file(data, filename);
                } catch (e) {
                    alert("Failed to get file: " + e);
                }
            }
        };
    }
    emulator.add_listener("9p-attach", function() {
        const fsPanel = document.getElementById("filesystem_panel");
        if (fsPanel) fsPanel.style.display = "block";
    });

    if (screenContainer) {
        screenContainer.onclick = function(e) {
            if (emulator.is_running() && emulator.speaker_adapter?.audio_context?.state === "suspended") {
                emulator.speaker_adapter.audio_context.resume();
            }
            if (window.getSelection().isCollapsed) {
                const phone_keyboard = document.getElementsByClassName("phone_keyboard")[0];
                if (phone_keyboard) {
                    phone_keyboard.style.top = window.scrollY + e.clientY + 20 + "px";
                    phone_keyboard.style.left = window.scrollX + e.clientX + "px";
                    phone_keyboard.value = "";
                    phone_keyboard.focus();
                }
            }
        };
    }

    let last_tick = Date.now();
    let last_instr_counter = 0;
    let total_instructions = 0;
    let running_time = 0;
    let statsInterval = null;

    function update_info() {
        const now = Date.now();
        const instruction_counter = emulator.get_instruction_counter ? emulator.get_instruction_counter() : 0;
        if (instruction_counter < last_instr_counter) {
            last_instr_counter -= 0x100000000;
        }
        const last_ips = instruction_counter - last_instr_counter;
        last_instr_counter = instruction_counter;
        total_instructions += last_ips;
        const delta_time = now - last_tick;
        if (delta_time) {
            running_time += delta_time;
            last_tick = now;
            const speedEl = document.getElementById("speed");
            const avgSpeedEl = document.getElementById("avg_speed");
            const runningTimeEl = document.getElementById("running_time");
            if (speedEl) speedEl.textContent = (last_ips / 1000 / delta_time).toFixed(1);
            if (avgSpeedEl) avgSpeedEl.textContent = (total_instructions / 1000 / running_time).toFixed(1);
            if (runningTimeEl) runningTimeEl.textContent = format_timestamp(Math.floor(running_time / 1000));
        }
    }

    emulator.add_listener("emulator-started", function() {
        last_tick = Date.now();
        statsInterval = setInterval(update_info, 1000);
    });

    emulator.add_listener("emulator-stopped", function() {
        update_info();
        if (statsInterval !== null) {
            clearInterval(statsInterval);
        }
    });

    emulator.add_listener("mouse-enable", function(is_enabled) {
        const el = document.getElementById("info_mouse_enabled");
        if (el) el.textContent = is_enabled ? "Yes" : "No";
    });

    emulator.add_listener("screen-set-size", function(args) {
        const [w, h, bpp] = args;
        const resEl = document.getElementById("info_res");
        const modeEl = document.getElementById("info_vga_mode");
        if (resEl) resEl.textContent = w + "x" + h + (bpp ? "x" + bpp : "");
        if (modeEl) modeEl.textContent = bpp ? "Graphical" : "Text";
    });

    const stats_9p = { read: 0, write: 0, files: [] };
    emulator.add_listener("9p-read-start", function(args) {
        const file = args[0];
        stats_9p.files.push(file);
        const infoFs = document.getElementById("info_filesystem");
        if (infoFs) infoFs.style.display = "block";
        const statusEl = document.getElementById("info_filesystem_status");
        if (statusEl) statusEl.textContent = "Loading ...";
        const fileEl = document.getElementById("info_filesystem_last_file");
        if (fileEl) fileEl.textContent = file;
    });
    emulator.add_listener("9p-read-end", function(args) {
        stats_9p.read += args[1];
        const bytesReadEl = document.getElementById("info_filesystem_bytes_read");
        if (bytesReadEl) bytesReadEl.textContent = stats_9p.read;
        const file = args[0];
        stats_9p.files = stats_9p.files.filter(f => f !== file);
        const fileEl = document.getElementById("info_filesystem_last_file");
        const statusEl = document.getElementById("info_filesystem_status");
        if (stats_9p.files[0]) {
            if (fileEl) fileEl.textContent = stats_9p.files[0];
        } else {
            if (statusEl) statusEl.textContent = "Idle";
        }
    });
    emulator.add_listener("9p-write-end", function(args) {
        stats_9p.write += args[1];
        const bytesWrittenEl = document.getElementById("info_filesystem_bytes_written");
        if (bytesWrittenEl) bytesWrittenEl.textContent = stats_9p.write;
    });

    const stats_storage = { read: 0, read_sectors: 0, write: 0, write_sectors: 0 };
    const ideTypeEl = document.getElementById("ide_type");
    if (ideTypeEl) ideTypeEl.textContent = cfg.state ? " (state image)" : " (virtio / host9p)";
    emulator.add_listener("ide-read-start", function() {
        const el = document.getElementById("info_storage");
        if (el) el.style.display = "block";
        const statusEl = document.getElementById("info_storage_status");
        if (statusEl) statusEl.textContent = "Loading ...";
    });
    emulator.add_listener("ide-read-end", function(args) {
        stats_storage.read += args[1];
        stats_storage.read_sectors += args[2];
        const statusEl = document.getElementById("info_storage_status");
        if (statusEl) statusEl.textContent = "Idle";
        const bytesReadEl = document.getElementById("info_storage_bytes_read");
        if (bytesReadEl) bytesReadEl.textContent = stats_storage.read;
        const sectorsReadEl = document.getElementById("info_storage_sectors_read");
        if (sectorsReadEl) sectorsReadEl.textContent = stats_storage.read_sectors;
    });
    emulator.add_listener("ide-write-end", function(args) {
        stats_storage.write += args[1];
        stats_storage.write_sectors += args[2];
        const bytesWrittenEl = document.getElementById("info_storage_bytes_written");
        if (bytesWrittenEl) bytesWrittenEl.textContent = stats_storage.write;
        const sectorsWrittenEl = document.getElementById("info_storage_sectors_written");
        if (sectorsWrittenEl) sectorsWrittenEl.textContent = stats_storage.write_sectors;
    });

    const stats_net = { bytes_transmitted: 0, bytes_received: 0 };
    emulator.add_listener("eth-receive-end", function(args) {
        stats_net.bytes_received += args[0];
        const infoNet = document.getElementById("info_network");
        if (infoNet) infoNet.style.display = "block";
        const recEl = document.getElementById("info_network_bytes_received");
        if (recEl) recEl.textContent = stats_net.bytes_received;
    });
    emulator.add_listener("eth-transmit-end", function(args) {
        stats_net.bytes_transmitted += args[0];
        const infoNet = document.getElementById("info_network");
        if (infoNet) infoNet.style.display = "block";
        const transEl = document.getElementById("info_network_bytes_transmitted");
        if (transEl) transEl.textContent = stats_net.bytes_transmitted;
    });

    console.log("Guix/Arch v86 system initialized via /js/v86.js.");
}

if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", initV86);
} else {
    initV86();
}
/*
# LICENSE:


Copyright (c) 2012, The v86 contributors
All rights reserved.

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this
   list of conditions and the following disclaimer.
2. Redistributions in binary form must reproduce the above copyright notice,
   this list of conditions and the following disclaimer in the documentation
   and/or other materials provided with the distribution.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR CONTRIBUTORS BE LIABLE FOR
ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
(INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND
ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.


# LICENSE.MIT:

QEMU Floppy disk emulator (Intel 82078)

Copyright (c) 2003, 2007 Jocelyn Mayer
Copyright (c) 2008 Hervé Poussineau

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL
THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.

*/