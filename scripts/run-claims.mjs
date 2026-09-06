import { readFileSync, readdirSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { extname, join } from 'node:path';

const claims = JSON.parse(readFileSync(new URL('../.factory/claims.json', import.meta.url), 'utf8'));
const sourceRoots = [
  new URL('../src', import.meta.url),
  new URL('../tests', import.meta.url),
  new URL('../frontend/src', import.meta.url),
];
const sourceExtensions = new Set(['.rs', '.ts', '.svelte']);

function sourceFiles(directory) {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory.pathname, entry.name);
    if (entry.isDirectory()) return sourceFiles(new URL(`file://${path}/`));
    return sourceExtensions.has(extname(entry.name)) ? [path] : [];
  });
}

const tags = new Map();
for (const file of sourceRoots.flatMap(sourceFiles)) {
  const source = readFileSync(file, 'utf8');
  for (const match of source.matchAll(/@claim:([a-z0-9-]+)/g)) {
    const locations = tags.get(match[1]) || [];
    locations.push(file);
    tags.set(match[1], locations);
  }
}

const ids = claims.map((claim) => claim.id);
if (new Set(ids).size !== ids.length) {
  throw new Error('claims.json contains duplicate claim IDs');
}
for (const claim of claims) {
  const locations = tags.get(claim.id) || [];
  if (locations.length !== 1) {
    throw new Error(`@claim:${claim.id} must occur exactly once in test source; found ${locations.length}`);
  }
}
for (const [tag] of tags) {
  if (!ids.includes(tag)) throw new Error(`@claim:${tag} has no claims.json entry`);
}

for (const claim of claims) {
  process.stdout.write(`\n[claim:${claim.id}] ${claim.test}\n`);
  const result = spawnSync(claim.test, { shell: true, stdio: 'inherit' });
  if (result.status !== 0) process.exit(result.status || 1);
}
