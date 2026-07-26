// Drive the PoC page in headless Chromium and report the result
import { chromium } from "playwright";

let browser;
try {
  browser = await chromium.launch({ headless: true });
} catch {
  browser = await chromium.launch({ headless: true, channel: "chrome" });
}
const page = await browser.newPage();
page.on("console", (m) => {
  if (m.type() === "error") console.error("[page-error]", m.text());
});
page.on("pageerror", (e) => console.error("[pageerror]", e.message));

await page.goto("http://127.0.0.1:8787/");
const result = await Promise.race([
  page.waitForSelector("#done", { timeout: 300000 }).then(() => "done"),
  page.waitForSelector("#failed", { timeout: 300000 }).then(() => "failed"),
]).catch(() => "timeout");

console.log(await page.locator("#log").textContent());
console.log("=== RESULT:", result, "===");
await browser.close();
process.exit(result === "done" ? 0 : 1);
