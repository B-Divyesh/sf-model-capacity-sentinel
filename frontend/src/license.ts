const slug = 'model-capacity-sentinel';
const tokenKey = `sb_license:${slug}`;
const verdictKey = `${tokenKey}:verdict`;
const day = 86_400_000;

type CachedVerdict = { valid: boolean; at: number; token: string };
export type LicenseState = { unlocked: boolean; checking: boolean; notice: string; token: string };
export const checkoutUrl = `https://api.sociobot.in/api/v1/products/${slug}/checkout`;

function cachedVerdict(token: string): CachedVerdict | null {
  try {
    const cached = JSON.parse(localStorage.getItem(verdictKey) || 'null') as CachedVerdict | null;
    if (!cached || cached.token !== token || Date.now() - cached.at >= day) return null;
    return cached;
  } catch {
    return null;
  }
}

function state(token: string, valid: boolean): LicenseState {
  return {
    unlocked: valid,
    checking: false,
    notice: valid ? 'Atlas license active' : 'License no longer active',
    token,
  };
}

export function consumeLicenseFromUrl() {
  const url = new URL(location.href);
  const token = url.searchParams.get('license');
  if (token) {
    if (localStorage.getItem(tokenKey) !== token) localStorage.removeItem(verdictKey);
    localStorage.setItem(tokenKey, token);
    url.searchParams.delete('license');
    history.replaceState({}, '', url.pathname + url.search + url.hash);
  }
  return token;
}

export function storedToken() {
  return localStorage.getItem(tokenKey) || '';
}

export function storeToken(token: string) {
  localStorage.setItem(tokenKey, token.trim());
  localStorage.removeItem(verdictKey);
}

export function cachedLicense(token: string): LicenseState | null {
  const cached = cachedVerdict(token);
  return cached ? state(token, cached.valid) : null;
}

export async function verifyLicense(token: string, force = false): Promise<LicenseState> {
  if (!token) return { unlocked: false, checking: false, notice: '', token: '' };
  const cached = cachedVerdict(token);
  if (!force && cached) return state(token, cached.valid);
  try {
    const res = await fetch(
      `https://api.sociobot.in/api/v1/products/${slug}/verify?license=${encodeURIComponent(token)}`,
    );
    const body = await res.json() as { valid?: unknown };
    const valid = Boolean(body.valid);
    localStorage.setItem(verdictKey, JSON.stringify({ valid, at: Date.now(), token }));
    return state(token, valid);
  } catch {
    return {
      unlocked: cached?.valid ?? false,
      checking: false,
      notice: 'License check will retry when online',
      token,
    };
  }
}
