import { test, expect, chromium } from "@playwright/test";
import { PNG } from "pngjs";
import fs from "node:fs";
import path from "node:path";

const out = path.resolve("../../artifacts/web-proof");

async function bridge(cdp) {
  const response = await cdp.send("Runtime.evaluate", {
    expression: "JSON.parse(window.name)",
    returnByValue: true,
  });
  return response.result.value;
}

async function expectBridge(cdp, expected) {
  await expect.poll(async () => await bridge(cdp)).toMatchObject(expected);
}

function assertScreenshotSanity(file) {
  const png = PNG.sync.read(fs.readFileSync(file));
  expect([png.width, png.height]).toEqual([900, 600]);
  const colors = new Set();
  for (let y = 0; y < png.height; y += 10) {
    for (let x = 0; x < png.width; x += 10) {
      const at = (y * png.width + x) * 4;
      colors.add(`${png.data[at]},${png.data[at + 1]},${png.data[at + 2]},${png.data[at + 3]}`);
    }
  }
  expect(colors.size).toBeGreaterThan(8);
  // Palette center and backdrop must be visibly different at the fixed viewport.
  const rgba = (x, y) => [...png.data.slice((y * png.width + x) * 4, (y * png.width + x) * 4 + 4)];
  expect(rgba(450, 135)).not.toEqual(rgba(20, 20));
}

test("document-owned GPUI canvas survives full real-keyboard palette flow", async () => {
  fs.mkdirSync(out, { recursive: true });
  const browser = await chromium.launch({
    channel: "chrome",
    headless: false,
    args: ["--use-gl=swiftshader", "--use-angle=swiftshader", "--disable-gpu-sandbox"],
  });
  const context = await browser.newContext({ viewport: { width: 900, height: 600 }, deviceScaleFactor: 1 });
  const page = await context.newPage();
  const cdp = await context.newCDPSession(page);
  await page.goto("http://127.0.0.1:8080", { waitUntil: "networkidle" });
  await expect.poll(() => page.locator("canvas").count()).toBe(1);
  const canvas = page.locator("canvas");
  await expect(canvas).toHaveCSS("width", "900px");
  await expect(canvas).toHaveCSS("height", "600px");
  await canvas.evaluate((element) => { element.dataset.productionProof = "persistent"; });
  await canvas.click({ position: { x: 450, y: 300 } });
  await canvas.focus();

  await page.keyboard.press("Control+k");
  await expectBridge(cdp, {
    open: true,
    query: "",
    selected: 0,
    depth: 0,
    results: ["file.open", "theme"],
  });

  await page.keyboard.type("theme");
  await expectBridge(cdp, { open: true, query: "theme", selected: 0, results: ["theme"] });
  await page.keyboard.press("Enter");
  await expectBridge(cdp, {
    open: true,
    query: "",
    selected: 0,
    depth: 1,
    results: ["theme.dark", "theme.light"],
  });

  await page.keyboard.press("Escape");
  await expectBridge(cdp, { open: true, query: "", selected: 0, depth: 0 });
  await page.keyboard.press("ArrowDown");
  await expectBridge(cdp, { open: true, selected: 1, results: ["file.open", "theme"] });
  await page.keyboard.press("Escape");
  await expectBridge(cdp, { open: false, depth: 0 });

  await page.keyboard.press("Control+k");
  await page.keyboard.type("open");
  await expectBridge(cdp, { open: true, query: "open", results: ["file.open"] });
  await page.keyboard.press("Enter");
  await expectBridge(cdp, { open: false, executed: "file.open" });

  await page.keyboard.press("Control+k");
  await expectBridge(cdp, { open: true, query: "" });
  const shot = path.join(out, "palette-900x600.png");
  await page.screenshot({ path: shot });
  assertScreenshotSanity(shot);
  await page.mouse.click(10, 10);
  await expectBridge(cdp, { open: false });

  await expect(page.locator('canvas[data-production-proof="persistent"]')).toHaveCount(1);
  await page.keyboard.press("Control+k");
  await expectBridge(cdp, { open: true });
  await page.keyboard.press("Escape");
  await expectBridge(cdp, { open: false });
  await browser.close();
});
