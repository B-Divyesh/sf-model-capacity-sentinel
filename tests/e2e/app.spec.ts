import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

const accessToken='e2e-access-token-that-is-at-least-24-chars';
async function seedAccess(page:import('@playwright/test').Page){
  await page.addInitScript(token=>sessionStorage.setItem('capacity-sentinel-access',token),accessToken);
}

test('first visit waits for an access code without a failed bootstrap request',async({page})=>{
  const consoleErrors:string[]=[];
  const summaryRequests:string[]=[];
  page.on('console',message=>{if(message.type()==='error')consoleErrors.push(message.text())});
  page.on('request',request=>{if(new URL(request.url()).pathname==='/api/summary')summaryRequests.push(request.url())});
  await page.goto('/');
  await expect(page.getByRole('heading',{name:"Open this project’s field sheet"})).toBeVisible();
  expect(summaryRequests).toEqual([]);
  expect(consoleErrors).toEqual([]);
  await page.getByLabel('Project access code').fill(accessToken);
  await page.getByLabel('Project access code').press('Enter');
  await expect(page.getByRole('heading',{name:"Open this project’s field sheet"})).not.toBeVisible();
  await expect(page.getByText('Preparing the field sheet…')).not.toBeVisible();
  expect(summaryRequests).toHaveLength(1);
  expect(consoleErrors).toEqual([]);
});

test('empty state, legal routes, and keyboard dialog work',async({page})=>{
  await seedAccess(page);
  const consoleErrors:string[]=[];
  const failedResponses:string[]=[];
  page.on('console',message=>{if(message.type()==='error')consoleErrors.push(message.text())});
  page.on('response',response=>{if(response.status()>=400)failedResponses.push(`${response.status()} ${response.url()}`)});
  await page.goto('/');
  await expect(page).toHaveTitle(/Capacity Sentinel/);
  await expect(page.locator('main')).toHaveCount(1);
  await expect(page.getByRole('heading',{level:1})).toHaveCount(1);
  await expect(page.getByText('No specimens yet')).toBeVisible();
  const scan=await new AxeBuilder({page}).analyze();
  expect(scan.violations.filter(v=>v.impact==='serious'||v.impact==='critical')).toEqual([]);
  await page.getByRole('button',{name:'Add your first probe'}).click();
  await expect(page.getByRole('dialog')).toBeVisible();
  await expect(page.getByLabel('Canary name')).toBeFocused();
  await page.keyboard.press('Escape');
  await expect(page.getByRole('dialog')).not.toBeVisible();
  await page.goto('/privacy');
  await expect(page.getByRole('heading',{level:1})).toHaveText('Privacy, by habitat.');
  await page.goto('/terms');
  await expect(page.getByRole('heading',{level:1})).toHaveText('Terms of use.');
  expect([...consoleErrors,...failedResponses]).toEqual([]);
});

test('populated dashboard has no serious or critical accessibility violations',async({page},testInfo)=>{
  await seedAccess(page);
  const name=`Public endpoint specimen ${testInfo.project.name}`;
  const created=await page.request.post('/api/probes',{headers:{Authorization:`Bearer ${accessToken}`},data:{name,provider:'Example',endpoint_url:'https://example.com/chat',model:'test-model',api_key:'test-key',prompt:'Synthetic QA probe only',required_fields:['status'],interval_minutes:5,timeout_ms:2000,latency_slo_ms:1000,availability_slo_percent:99,max_output_tokens:32,daily_token_cap:1000,enabled:true}});
  expect(created.status()).toBe(201);
  await page.goto('/');
  await expect(page.getByText(name,{exact:true})).toBeVisible();
  await expect(page.locator('.ticks[role="img"]').first()).toBeVisible();
  const scan=await new AxeBuilder({page}).analyze();
  expect(scan.violations.filter(v=>v.impact==='serious'||v.impact==='critical')).toEqual([]);
});
