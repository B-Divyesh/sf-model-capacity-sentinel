import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';

describe('container release identity', () => {
  it('bakes one build argument into the binary and image metadata', () => {
    const dockerfile = readFileSync(resolve(process.cwd(), 'Dockerfile'), 'utf8');
    const runtimeStage = dockerfile.split('FROM gcr.io/distroless/cc-debian12:nonroot')[1];

    expect(dockerfile).toMatch(/^ARG BUILD_SHA=dev$/m);
    expect(dockerfile).toMatch(/^LABEL org\.opencontainers\.image\.revision=\$BUILD_SHA$/m);
    expect(dockerfile).not.toContain('BUILD_SHA=unknown');
    expect(runtimeStage).not.toMatch(/^ENV .*BUILD_SHA=.*$/m);
  });
});
