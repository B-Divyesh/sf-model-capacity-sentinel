import { defineConfig, devices } from '@playwright/test';
export default defineConfig({
  testDir:'./tests/e2e', timeout:30_000, retries:0,
  use:{baseURL:'http://127.0.0.1:8080',trace:'retain-on-failure'},
  webServer:{command:'npm run build && DATA_DIR=/tmp/capacity-sentinel-e2e cargo run',url:'http://127.0.0.1:8080/health',reuseExistingServer:false,timeout:120_000},
  projects:[{name:'desktop',use:{...devices['Desktop Chrome']}},{name:'mobile',use:{browserName:'chromium',viewport:{width:390,height:844},isMobile:true,hasTouch:true}}]
});
