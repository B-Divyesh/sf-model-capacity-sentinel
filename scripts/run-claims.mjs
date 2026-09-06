import { readFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';

const claims = JSON.parse(readFileSync(new URL('../.factory/claims.json', import.meta.url), 'utf8'));
for (const claim of claims) {
  process.stdout.write(`\n[claim:${claim.id}] ${claim.test}\n`);
  const result = spawnSync(claim.test, { shell: true, stdio: 'inherit' });
  if (result.status !== 0) process.exit(result.status || 1);
}
