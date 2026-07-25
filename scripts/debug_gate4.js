const { chromium } = require("/usr/local/lib/node_modules/playwright");

(async () => {
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();
  console.log("Navigating to v86 VM on server port 18999...");
  await page.goto("http://127.0.0.1:18999/index.html?profile=guix", { waitUntil: "networkidle" });

  console.log("Waiting 30 seconds for VM to boot...");
  await page.waitForTimeout(30000);

  const diag = await page.evaluate(() => {
    const vga = window.emulator ? window.emulator.vga : null;
    const canvas = document.querySelector("canvas");
    let ctxData = null;
    if (canvas) {
      const ctx = canvas.getContext("2d");
      const img = ctx.getImageData(0, 0, canvas.width, canvas.height).data;
      let nonZero = 0;
      for (let i = 0; i < img.length; i += 4) {
        if (img[i] > 0 || img[i+1] > 0 || img[i+2] > 0) nonZero++;
      }
      ctxData = { width: canvas.width, height: canvas.height, styleDisplay: canvas.style.display, nonZero };
    }
    return {
      hasEmulator: !!window.emulator,
      hasVga: !!vga,
      graphical_mode: vga ? vga.graphical_mode : null,
      svga_enabled: vga ? vga.svga_enabled : null,
      svga_width: vga ? vga.svga_width : null,
      svga_height: vga ? vga.svga_height : null,
      svga_bpp: vga ? vga.svga_bpp : null,
      canvasInfo: ctxData
    };
  });

  console.log("=== V86 VGA DIAGNOSTIC ===");
  console.log(JSON.stringify(diag, null, 2));
  console.log("==========================");

  await browser.close();
})();
