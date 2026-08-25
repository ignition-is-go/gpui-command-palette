import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { PNG } from "pngjs";

const out = path.resolve("../../artifacts/web-proof");
const demoUrl = process.env.WEB_PROOF_URL ?? "http://127.0.0.1:8080";
const deadline = Date.now() + 60_000;
const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
let target;
while (Date.now() < deadline) {
  try {
    const targets = await (await fetch("http://127.0.0.1:9222/json/list")).json();
    target = targets.find((entry) => entry.type === "page");
    if (target) break;
  } catch {}
  await sleep(250);
}
if (!target) throw new Error("Chrome page target was not exposed through DevTools");

const socket = new WebSocket(target.webSocketDebuggerUrl);
await new Promise((resolve, reject) => {
  socket.addEventListener("open", resolve, { once: true });
  socket.addEventListener("error", reject, { once: true });
});
let nextId = 0;
const pending = new Map();
socket.addEventListener("message", (event) => {
  const message = JSON.parse(event.data);
  const waiter = pending.get(message.id);
  if (!waiter) return;
  pending.delete(message.id);
  if (message.error) waiter.reject(new Error(JSON.stringify(message.error)));
  else waiter.resolve(message.result);
});
function command(method, params = {}) {
  const id = ++nextId;
  socket.send(JSON.stringify({ id, method, params }));
  return new Promise((resolve, reject) => pending.set(id, { resolve, reject }));
}
async function evaluate(expression) {
  const result = await command("Runtime.evaluate", { expression, returnByValue: true, awaitPromise: true });
  if (result.exceptionDetails) throw new Error(JSON.stringify(result.exceptionDetails));
  return result.result?.value;
}
async function state() {
  const value = await evaluate("window.name");
  return value ? JSON.parse(value) : {};
}
async function expectState(expected) {
  let actual;
  const until = Date.now() + 8_000;
  while (Date.now() < until) {
    actual = await state();
    try {
      for (const [key, value] of Object.entries(expected)) assert.deepEqual(actual[key], value);
      return;
    } catch {}
    await sleep(100);
  }
  throw new Error(`state mismatch: expected=${JSON.stringify(expected)} actual=${JSON.stringify(actual)}`);
}
async function key(keyName, { code = keyName, text, modifiers = 0 } = {}) {
  await command("Input.dispatchKeyEvent", { type: "keyDown", key: keyName, code, text, modifiers });
  await command("Input.dispatchKeyEvent", { type: "keyUp", key: keyName, code, modifiers });
}
async function shortcutK() {
  await command("Input.dispatchKeyEvent", { type: "keyDown", key: "Control", code: "ControlLeft", modifiers: 2 });
  await key("k", { code: "KeyK", modifiers: 2 });
  await command("Input.dispatchKeyEvent", { type: "keyUp", key: "Control", code: "ControlLeft" });
}
async function type(text) {
  for (const character of text) await key(character, { code: `Key${character.toUpperCase()}`, text: character });
}
async function click(x, y) {
  await command("Input.dispatchMouseEvent", { type: "mouseMoved", x, y });
  await command("Input.dispatchMouseEvent", { type: "mousePressed", x, y, button: "left", buttons: 1, clickCount: 1 });
  await command("Input.dispatchMouseEvent", { type: "mouseReleased", x, y, button: "left", clickCount: 1 });
}
function assertScreenshot(file) {
  const png = PNG.sync.read(fs.readFileSync(file));
  assert.deepEqual([png.width, png.height], [900, 600]);
  const colors = new Set();
  for (let y = 0; y < png.height; y += 10) for (let x = 0; x < png.width; x += 10) {
    const at = (y * png.width + x) * 4;
    colors.add(png.data.subarray(at, at + 4).join(","));
  }
  assert(colors.size > 8);
  const rgba = (x, y) => [...png.data.subarray((y * png.width + x) * 4, (y * png.width + x) * 4 + 4)];
  assert.notDeepEqual(rgba(450, 200), rgba(20, 20));
}

fs.mkdirSync(out, { recursive: true });
await command("Page.enable");
await command("Runtime.enable");
await command("Emulation.setDeviceMetricsOverride", { width: 900, height: 600, deviceScaleFactor: 1, mobile: false });
await command("Page.navigate", { url: demoUrl });
while (Date.now() < deadline && !await evaluate("Boolean(document.querySelector('canvas'))")) await sleep(250);
assert.equal(await evaluate("document.querySelectorAll('canvas').length"), 1);
assert.deepEqual(await evaluate("[getComputedStyle(document.querySelector('canvas')).width, getComputedStyle(document.querySelector('canvas')).height]"), ["900px", "600px"]);
await evaluate("document.querySelector('canvas').dataset.productionProof = 'persistent'");
await click(450, 300);
await shortcutK();
await expectState({ open: true, query: "", selected: 0, depth: 0, results: ["file.open", "theme"] });
await type("theme");
await expectState({ open: true, query: "theme", selected: 0, results: ["theme"] });
await key("Enter");
await expectState({ open: true, query: "", selected: 0, depth: 1, results: ["theme.dark", "theme.light"] });
await key("Escape");
await expectState({ open: true, query: "", selected: 0, depth: 0 });
await key("ArrowDown");
await expectState({ open: true, selected: 1, results: ["file.open", "theme"] });
await key("Escape");
await expectState({ open: false, depth: 0 });
await shortcutK();
await type("open");
await expectState({ open: true, query: "open", results: ["file.open"] });
await key("Enter");
await expectState({ open: false, executed: "file.open" });
await shortcutK();
await expectState({ open: true, query: "" });
const shot = path.join(out, "palette-900x600.png");
const screenshot = await command("Page.captureScreenshot", { format: "png", fromSurface: true });
fs.writeFileSync(shot, Buffer.from(screenshot.data, "base64"));
assertScreenshot(shot);
await click(10, 10);
await expectState({ open: false });
assert.equal(await evaluate("document.querySelectorAll('canvas[data-production-proof=persistent]').length"), 1);
await shortcutK();
await expectState({ open: true });
await key("Escape");
await expectState({ open: false });
socket.close();
console.log("real CDP keyboard, canvas, state, and screenshot proof passed");
