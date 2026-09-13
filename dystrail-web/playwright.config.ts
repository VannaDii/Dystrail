import { defineConfig, devices } from '@playwright/test';
import { resolve } from 'node:path';

const port = Number(process.env.PLAYTEST_PORT || 8080);
const baseURL = `http://127.0.0.1:${port}/play/`;
const channel = process.env.PLAYTEST_CHANNEL;
const chromeLauncher = process.platform === 'darwin' && channel === 'chrome'
  ? { executablePath: resolve(__dirname, 'tests-e2e/chrome-launcher.py') }
  : undefined;

export default defineConfig({
  testDir: './tests-e2e',
  fullyParallel: true,
  workers: 2,
  reporter: 'list',
  use: { channel, launchOptions: chromeLauncher, baseURL, trace: 'retain-on-failure', video: process.env.CI ? 'retain-on-failure' : 'off', headless: true },
  projects: [
    { name: 'chromium', use: { ...devices['Desktop Chrome'] } },
    { name: 'mobile', use: { ...devices['Pixel 7'] } },
  ],
  webServer: process.env.PLAYTEST_EXTERNAL === '1' ? undefined : {
    command: process.env.CI
      ? `python3 tests-e2e/serve-build.py ${port}`
      : `NO_COLOR=true PUBLIC_URL=/play trunk serve --address 127.0.0.1 --public-url /play/ --port ${port}`,
    url: baseURL,
    reuseExistingServer: !process.env.CI,
    cwd: __dirname,
    timeout: 120_000,
  },
});
