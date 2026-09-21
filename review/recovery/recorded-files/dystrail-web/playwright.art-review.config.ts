import base from './playwright.config';
export default {
 ...base,
 testDir:'./tests-e2e',
 outputDir:process.env.ART_REVIEW_OUTPUT || '../review/art-satire/road-c03-final-results',
 workers:2,
 webServer:undefined,
 use:{...base.use,baseURL:'http://127.0.0.1:62531/play/',launchOptions:{executablePath:'/Users/vanna/Library/Caches/ms-playwright/chromium_headless_shell-1208/chrome-headless-shell-mac-arm64/chrome-headless-shell'}},
};
