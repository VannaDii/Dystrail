import { defineConfig, devices } from '@playwright/test';

const PORT = 8080;
const PUBLIC_URL = 'http://localhost:8080/play/';
const isCi = Boolean(process.env.CI);

export default defineConfig({
  testDir: './tests-e2e',
  fullyParallel: true,
  reporter: 'list',
  use: {
    baseURL: PUBLIC_URL,
    trace: isCi ? 'on' : 'on-first-retry',
    video: isCi ? 'on' : 'retain-on-failure',
    headless: true,
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
  webServer: {
    command: 'PUBLIC_URL=/play trunk serve --release --public-url /play/ --port 8080 --open=false',
    url: `http://localhost:${PORT}/play/`,
    reuseExistingServer: !process.env.CI,
    cwd: __dirname,
    timeout: 120_000,
  },
});
