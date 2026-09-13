import {chromium as installed} from '/Users/vanna/Source/Dystrail/dystrail-web/node_modules/playwright/index.mjs';
export const chromium={launch:options=>installed.launch({...options,executablePath:'/Users/vanna/Source/Dystrail/dystrail-web/tests-e2e/chrome-launcher.py'})};
