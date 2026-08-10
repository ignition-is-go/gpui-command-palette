import { defineConfig } from "@playwright/test";
export default defineConfig({
  testDir: ".",
  testMatch: "palette.spec.mjs",
  timeout: 60_000,
  retries: 1,
  workers: 1,
  reporter: [["line"]],
});
