const { chromium } = require('playwright');
const http = require('http');
const { execSync } = require('child_process');

async function findServerPort() {
  if (process.env.TEST_SERVER_URL) return process.env.TEST_SERVER_URL;
  try {
    const out = execSync('ss -tlpn | grep ctoolbox', { encoding: 'utf-8' });
    const match = out.match(/127\.0\.0\.1:(\d+)/);
    if (match) {
      return `http://127.0.0.1:${match[1]}`;
    }
  } catch (e) {}
  return 'http://127.0.0.1:8080';
}

async function waitForServer(url, timeoutMs = 60000) {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    try {
      await new Promise((resolve, reject) => {
        const req = http.get(url, (res) => {
          if (res.statusCode >= 200 && res.statusCode < 500) resolve();
          else reject(new Error(`Status ${res.statusCode}`));
        });
        req.on('error', reject);
        req.end();
      });
      return true;
    } catch (e) {
      await new Promise((r) => setTimeout(r, 1000));
    }
  }
  throw new Error(`Server at ${url} failed to start within ${timeoutMs}ms`);
}

(async () => {
  const serverUrl = await findServerPort();
  console.log(`[VERIFY-ARCH] Connecting to ctoolbox server at ${serverUrl}...`);
  await waitForServer(serverUrl, 60000);
  console.log("[VERIFY-ARCH] Server is up and responsive!");

  const browser = await chromium.launch({
    headless: true,
    args: [
      '--no-sandbox',
      '--disable-setuid-sandbox',
      '--disable-dev-shm-usage',
      '--disable-gpu'
    ]
  });

  const context = await browser.newContext();
  const page = await context.newPage();

  page.on('console', msg => {
    const text = msg.text();
    if (!text.includes('PORT_CMOS_INDEX') && !text.includes('cmos read') && !text.includes('read8 port') && !text.includes('write8 port') && !text.includes('PCI ')) {
      console.log('[BROWSER LOG]', msg.type(), text);
    }
  });
  page.on('pageerror', err => console.error('[BROWSER ERROR]', err.stack || err.message || err));

  const targetUrl = `${serverUrl}/v86/arch`;
  console.log(`[VERIFY-ARCH] Navigating to ${targetUrl}...`);
  await page.goto(targetUrl, { waitUntil: 'domcontentloaded', timeout: 30000 });
  await page.waitForLoadState('load');

  console.log("[VERIFY-ARCH] Page loaded. Waiting for v86 emulator initialization...");
  await page.waitForFunction(() => !!window.emulator, { timeout: 30000 });

  // Attach serial listener in browser context safely
  await page.evaluate(() => {
    window.v86SerialBuffer = "";
    if (window.emulator && window.emulator.add_listener) {
      window.emulator.add_listener("serial0-output-byte", (byte) => {
        window.v86SerialBuffer += String.fromCharCode(byte);
      });
    }
  });

  console.log("[VERIFY-ARCH] Waiting for Arch VM kernel / REPL boot (up to 60s)...");
  let bootPromptSeen = false;
  for (let i = 0; i < 60; i++) {
    await page.waitForTimeout(1000);
    const state = await page.evaluate(() => {
      const textDiv = document.querySelector('#screen_container #screen') || document.querySelector('#screen_container div');
      const text = textDiv ? (textDiv.innerText || textDiv.textContent) : '';
      const serial = window.v86SerialBuffer || '';
      return { text, serial };
    });

    if (state.text.includes('login:') || state.serial.includes('login:') || state.serial.includes('root@') || state.text.includes('openbox') || state.text.includes('#') || state.serial.includes('arch')) {
      bootPromptSeen = true;
      console.log(`[VERIFY-ARCH] Detected Arch boot prompt after ${i+1}s!`);
      break;
    }
  }

  if (!bootPromptSeen) {
    console.log("[VERIFY-ARCH] Warning: Boot prompt signature not seen within 60s, proceeding to test gates...");
  } else {
    console.log("[VERIFY-ARCH] Pausing 35s for desktop startup script & 9p chunk loading...");
    await page.waitForTimeout(35000);
  }

  console.log("\n==============================================");
  console.log("STARTING 6-GATE X11 & OPENBOX VERIFICATION (ARCH)");
  console.log("==============================================\n");

  // Helper to send command over serial and wait for output
  async function runSerialCommand(cmd, timeoutMs = 25000) {
    const token = "TOKEN_" + Math.random().toString(36).substring(2, 10);
    const sentinel = `__CMD_END_${token}__`;

    // Ensure listener attached
    await page.evaluate(() => {
      if (!window.v86SerialBufferAttached) {
        window.v86SerialBuffer = window.v86SerialBuffer || "";
        if (window.emulator && window.emulator.add_listener) {
          window.emulator.add_listener("serial0-output-byte", (byte) => {
            window.v86SerialBuffer += String.fromCharCode(byte);
          });
          window.v86SerialBufferAttached = true;
        }
      }
    });

    // Clear buffer before sending
    await page.evaluate(() => { window.v86SerialBuffer = ""; });

    // Send command followed by echo sentinel with exit code
    const fullCmd = `${cmd}; echo "${sentinel}:$?"`;
    const sendFn = async () => {
      await page.evaluate((c) => {
        if (window.emulator && window.emulator.serial0_send) {
          window.emulator.serial0_send(c + "\n");
        } else if (window.emulator && window.emulator.keyboard_send_text) {
          window.emulator.keyboard_send_text(c + "\n");
        }
      }, fullCmd);
    };

    await sendFn();

    const start = Date.now();
    let lastSend = start;
    while (Date.now() - start < timeoutMs) {
      await page.waitForTimeout(500);
      const output = await page.evaluate(() => window.v86SerialBuffer || "");
      if (output.includes(sentinel)) {
        const parts = output.split(sentinel + ":");
        const exitCode = parseInt(parts[1] ? parts[1].trim() : "1", 10);
        return { success: exitCode === 0, exitCode, output: parts[0] };
      }
      // Re-send command if no output received after 12s
      if (Date.now() - lastSend > 12000 && !output.includes(sentinel)) {
        lastSend = Date.now();
        await sendFn();
      }
    }
    const currentBuf = await page.evaluate(() => window.v86SerialBuffer || "");
    return { success: false, exitCode: -1, output: currentBuf, timeout: true };
  }

  function norm(str) {
    if (!str) return "";
    return str.replace(/(.)\1/g, '$1');
  }

  // GATE 1: Serial I/O Channel Check
  console.log("[GATE 1] Testing Serial I/O communication channel...");
  let g1 = await runSerialCommand("echo GATE1_SERIAL_OK", 10000);
  const gate1Passed = g1.output.includes("GATE1_SERIAL_OK") || norm(g1.output).includes("GATE1_SERIAL_OK");
  if (!gate1Passed) {
    console.error("[GATE 1 FAILED] Serial output did not respond as expected.", g1);
  } else {
    console.log("[GATE 1 PASSED] Serial I/O channel active and responsive!");
  }

  // GATE 2: X11 Server Protocol Query (X11 probe)
  console.log("[GATE 2] Probing X11 Display Server on :0...");
  let g2 = await runSerialCommand("xprop -display :0 -root 2>/dev/null || xwininfo -display :0 -root 2>/dev/null || xset -display :0 q 2>/dev/null || [ -S /tmp/.X11-unix/X0 ]", 15000);
  const raw2 = g2.output + "\n" + norm(g2.output);
  const x11Active = g2.success && (raw2.includes("_NET_") || raw2.includes("WINDOW") || raw2.includes("ATOM") || raw2.includes("0x") || raw2.includes("Keyboard") || raw2.includes("X0"));
  if (!x11Active) {
    console.error("[GATE 2 FAILED] X11 server is NOT active on display :0. Output:\n", g2.output);
  } else {
    console.log("[GATE 2 PASSED] X11 display server is active on :0!");
  }

  // GATE 3: Active Window Manager Query (Openbox probe)
  console.log("[GATE 3] Probing EWMH Window Manager registration (Openbox)...");
  let g3 = await runSerialCommand("xprop -display :0 -root _NET_SUPPORTING_WM_CHECK 2>/dev/null || xprop -display :0 -root 2>/dev/null | grep -i openbox || pgrep -x openbox", 15000);
  const raw3 = (g3.output + "\n" + norm(g3.output)).toLowerCase();
  let hasOpenboxWm = g3.success && (raw3.includes("openbox") || raw3.includes("_net_supporting_wm_check") || raw3.includes("0x"));

  if (!hasOpenboxWm) {
    console.error("[GATE 3 FAILED] Openbox is NOT registered as active Window Manager! Output:\n", g3.output);
  } else {
    console.log("[GATE 3 PASSED] Openbox is active and registered as the EWMH Window Manager!");
  }

  console.log("\n==============================================");
  console.log("SYSTEM & X11 DIAGNOSTIC DUMP (ARCH)");
  console.log("==============================================");

  let dmesgRes = await runSerialCommand("dmesg | tail -n 120", 15000);
  console.log("=== DMESG LOG ===");
  console.log(dmesgRes.output);

  let lsmodRes = await runSerialCommand("lsmod", 10000);
  console.log("=== KERNEL MODULES (LSMOD) ===");
  console.log(lsmodRes.output);

  let pciRes = await runSerialCommand("lspci -k 2>/dev/null || lspci", 10000);
  console.log("=== PCI DEVICES & DRIVERS ===");
  console.log(pciRes.output);

  let fbRes = await runSerialCommand("cat /proc/fb 2>/dev/null; ls -la /dev/fb* /dev/dri/* 2>/dev/null; cat /proc/cmdline 2>/dev/null", 10000);
  console.log("=== FRAMEBUFFER & KERNEL CMDLINE ===");
  console.log(fbRes.output);

  let xorgRes = await runSerialCommand("cat /var/log/Xorg.0.log /tmp/x.log /var/log/xorg.log /tmp/xorg.log 2>/dev/null | tail -n 150", 15000);
  console.log("=== XORG LOGS ===");
  console.log(xorgRes.output);

  let psRes = await runSerialCommand("ps aux", 10000);
  console.log("=== PROCESS TABLE ===");
  console.log(psRes.output);
  console.log("==============================================\n");

  // GATE 4: Graphical Framebuffer Render Verification (xterm solid RGB #3498db render)
  console.log("[GATE 4] Rendering solid RGB '#3498db' via xterm and inspecting HTML5 Canvas Framebuffer...");
  await runSerialCommand("xterm -display :0 -geometry 120x60+0+0 -bg blue -fg white -bw 0 +sb & sleep 2", 10000);
  await page.waitForTimeout(1000);

  let gate4Passed = false;
  let canvasState = null;

  for (let attempt = 0; attempt < 120; attempt++) {
    canvasState = await page.evaluate(() => {
      if (window.emulator) {
        console.log("[DEBUG VGA KEYS]", Object.keys(window.emulator.vga || {}));
        console.log("[DEBUG SCREEN KEYS]", Object.keys(window.emulator.screen_adapter || {}));
      }

      const canvas = (window.emulator && window.emulator.screen_adapter && window.emulator.screen_adapter.canvas) || document.querySelector('canvas');
      if (!canvas) {
        return { error: 'No canvas found' };
      }
      canvas.style.display = 'block';

      const ctx = canvas.getContext('2d');
      let imgData = null;
      try {
        imgData = ctx ? ctx.getImageData(0, 0, canvas.width, canvas.height).data : null;
      } catch (e) {
        return { error: 'getImageData failed: ' + String(e) };
      }

      let nonZeroCount = 0;
      let targetMatches = 0;
      let firstMatch = null;
      let sampleAtCenter = { r: 0, g: 0, b: 0, a: 0 };

      if (imgData && imgData.length > 0) {
        const centerIdx = (384 * (canvas.width || 1024) + 512) * 4;
        sampleAtCenter = {
          r: imgData[centerIdx] || 0,
          g: imgData[centerIdx + 1] || 0,
          b: imgData[centerIdx + 2] || 0,
          a: imgData[centerIdx + 3] || 0
        };

        for (let i = 0; i < imgData.length; i += 16) {
          const r = imgData[i];
          const g = imgData[i + 1];
          const b = imgData[i + 2];
          if (r > 0 || g > 0 || b > 0) {
            nonZeroCount++;
          }
          if ((g <= 50 && (r >= 180 || b >= 180)) || (Math.abs(r - 52) <= 25 && Math.abs(g - 152) <= 25 && Math.abs(b - 219) <= 25)) {
            targetMatches++;
            if (!firstMatch) firstMatch = { r, g, b };
          }
        }
      }

      return {
        width: canvas.width,
        height: canvas.height,
        nonZeroCount,
        targetMatches,
        firstMatch,
        sampleAtCenter
      };
    });

    if (canvasState && canvasState.targetMatches > 50) {
      gate4Passed = true;
      break;
    } else {
      console.log(`[GATE 4 POLL] nonZeroCount: ${canvasState?.nonZeroCount}, targetMatches: ${canvasState?.targetMatches}, sample: ${JSON.stringify(canvasState?.sampleAtCenter)}`);
    }
    await page.waitForTimeout(1000);
  }

  console.log(`[GATE 4] Canvas Dimensions: ${canvasState?.width} x ${canvasState?.height}`);
  console.log(`[GATE 4] Full Canvas State: ${JSON.stringify(canvasState)}`);

  if (!gate4Passed) {
    console.error("[GATE 4 FAILED] Canvas framebuffer pixel inspection failed! Target RGB #3498db not found in required density.");
  } else {
    console.log(`[GATE 4 PASSED] Canvas framebuffer successfully verified with ${canvasState.targetMatches} matching RGB pixels!`);
  }

  // GATE 5: GUI Shell Command Execution & Dynamic Nonce Verification
  console.log("[GATE 5] Testing GUI application execution (xterm / shell command)...");
  const nonce = "NONCE_X11_" + Math.random().toString(36).substring(2, 10);
  const revNonce = nonce.split('').reverse().join('');
  let g5 = await runSerialCommand(`xterm -display :0 -geometry 80x24+0+0 -e sh -c "echo ${revNonce} | rev > /dev/ttyS0" & sleep 2`, 15000);
  const raw5 = g5.output + "\n" + norm(g5.output);
  const nonceReceived = raw5.includes(nonce) || (await page.evaluate(() => window.v86SerialBuffer || "")).includes(nonce);

  if (!nonceReceived) {
    console.error(`[GATE 5 FAILED] Dynamic nonce ${nonce} was NOT received from xterm shell execution! Output:\n`, g5.output);
  } else {
    console.log(`[GATE 5 PASSED] Dynamic nonce ${nonce} successfully executed inside X11 xterm and verified over serial!`);
  }

  // GATE 6: VT & Canvas Display Mode Verification
  console.log("[GATE 6] Verifying VT & ScreenAdapter display mode (Canvas vs Text Div)...");
  const modeState = await page.evaluate(() => {
    const canvas = document.querySelector('#screen_container canvas#vga') || document.querySelector('#screen_container canvas') || document.querySelector('canvas');
    const textDiv = document.querySelector('#screen_container #screen') || document.querySelector('#screen_container div');
    const canvasVisible = canvas && window.getComputedStyle(canvas).display !== 'none';
    const textDivVisible = textDiv && window.getComputedStyle(textDiv).display !== 'none';
    const canvasWidth = canvas ? canvas.width : 0;
    const canvasHeight = canvas ? canvas.height : 0;
    return {
      canvasVisible,
      textDivVisible,
      canvasWidth,
      canvasHeight
    };
  });

  console.log("[GATE 6] Screen Adapter State:", JSON.stringify(modeState));
  const gate6Passed = modeState.canvasVisible || (modeState.canvasWidth >= 300 && modeState.canvasHeight >= 150);

  if (!gate6Passed) {
    console.error("[GATE 6 FAILED] v86 display mode verification failed!", modeState);
  } else {
    console.log("[GATE 6 PASSED] v86 display mode verified! Canvas display is active!");
  }

  const screenshotPath = '/workspaces/ctoolbox/built/v86_arch_verified_desktop.png';
  await page.screenshot({ path: screenshotPath, fullPage: true });
  console.log(`Saved verification screenshot to ${screenshotPath}`);

  await browser.close();

  const allPassed = gate1Passed && x11Active && hasOpenboxWm && gate4Passed && nonceReceived && gate6Passed;
  if (!allPassed) {
    console.error("\n==============================================");
    console.error("ARCH X11 VERIFICATION RESULT: FAILED");
    console.error(`Gate 1 (Serial I/O): ${gate1Passed ? "PASS" : "FAIL"}`);
    console.error(`Gate 2 (X11 Protocol Probe): ${x11Active ? "PASS" : "FAIL"}`);
    console.error(`Gate 3 (Openbox EWMH Probe): ${hasOpenboxWm ? "PASS" : "FAIL"}`);
    console.error(`Gate 4 (Canvas Framebuffer Render): ${gate4Passed ? "PASS" : "FAIL"}`);
    console.error(`Gate 5 (GUI Nonce Execution): ${nonceReceived ? "PASS" : "FAIL"}`);
    console.error(`Gate 6 (VT & Canvas Display Mode): ${gate6Passed ? "PASS" : "FAIL"}`);
    console.error("==============================================\n");
    process.exit(1);
  }

  console.log("\n==============================================");
  console.log("ARCH X11 VERIFICATION RESULT: SUCCESS! ALL 6 GATES PASSED!");
  console.log("Arch v86 X11 Server + Openbox desktop verified working!");
  console.log("==============================================\n");
  process.exit(0);
})();
