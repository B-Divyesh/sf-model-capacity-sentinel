import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

test('empty state, legal routes, and keyboard dialog work',async({page})=>{
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
