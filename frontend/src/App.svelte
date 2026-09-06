<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { api, ApiError, setAccessToken } from './api';
  import type { Probe, ProbeInput, Summary } from './types';
  import { cachedLicense, consumeLicenseFromUrl, storeToken, storedToken, verifyLicense, type LicenseState } from './license';
  import { loadDemo, removeDemoProbe, resetDemo, runDemoProbe, saveDemo, updateDemoProbe } from './demo';

  const empty: Summary = { probes: [], alerts: [], observations: [] };
  const buildId = import.meta.env.VITE_BUILD_SHA || 'local';
  const fresh = (): ProbeInput => ({ name: '', provider: '', endpoint_url: 'https://api.openai.com/v1/chat/completions', model: '', api_key: '', prompt: 'Return JSON with exactly {"status":"ok","region":"canary"}.', required_fields: ['status', 'region'], interval_minutes: 5, timeout_ms: 15000, latency_slo_ms: 3000, availability_slo_percent: 99, max_output_tokens: 128, daily_token_cap: 5000, enabled: true });
  let path = location.pathname;
  let data: Summary = empty;
  let loading = true;
  let loadError = '';
  let offline = !navigator.onLine;
  let busy = '';
  let toast = '';
  let formError = '';
  let fieldsText = 'status, region';
  let accessCode = '';
  let accessNeeded = false;
  let form: ProbeInput = fresh();
  let editing: Probe | null = null;
  let expanded = '';
  let confirming = '';
  let demoMode = false;
  let probeDialog: HTMLDialogElement;
  let planDialog: HTMLDialogElement;
  let firstField: HTMLInputElement;
  let license: LicenseState = { unlocked: false, checking: false, notice: '', token: '' };
  let licenseInput = '';

  $: demoMode = path === '/demo';
  $: openAlerts = data.alerts.filter((alert) => alert.status === 'open');
  $: resolvedAlerts = data.alerts.filter((alert) => alert.status === 'resolved');
  $: healthy = data.probes.filter((probe) => probe.last_observation?.outcome === 'healthy').length;
  $: availability = data.probes.length ? data.probes.reduce((total, probe) => total + probe.stats.availability_percent, 0) / data.probes.length : 0;
  $: avgLatency = data.probes.length ? Math.round(data.probes.reduce((total, probe) => total + probe.stats.p95_latency_ms, 0) / data.probes.length) : 0;

  onMount(() => {
    const saved = sessionStorage.getItem('capacity-sentinel-access');
    if (saved) { accessCode = saved; setAccessToken(saved); }
    const incoming = demoMode ? '' : consumeLicenseFromUrl();
    const token = incoming || storedToken();
    if (token) {
      const cached = cachedLicense(token);
      if (demoMode) license = cached || license;
      else { license = { ...license, checking: true, token }; verifyLicense(token).then((value) => license = value); }
    }
    enterRoute(path, false);
    const online = () => { offline = false; if (accessCode && !demoMode) load(); };
    const off = () => offline = true;
    const backForward = () => enterRoute(location.pathname, false);
    window.addEventListener('online', online);
    window.addEventListener('offline', off);
    window.addEventListener('popstate', backForward);
    const timer = setInterval(() => { if (accessCode && !offline && !document.hidden && !demoMode) load(false); }, 30000);
    return () => { window.removeEventListener('online', online); window.removeEventListener('offline', off); window.removeEventListener('popstate', backForward); clearInterval(timer); };
  });

  function routeTitle(next: string) { if (next === '/demo') return 'Demo — Capacity Sentinel'; if (next === '/privacy') return 'Privacy — Capacity Sentinel'; if (next === '/terms') return 'Terms — Capacity Sentinel'; return 'Capacity Sentinel — monitor model API capacity'; }
  function routeDescription(next: string) { if (next === '/demo') return 'Try a sample dashboard for model API capacity and output-shape monitoring.'; if (next === '/privacy') return 'Read how Capacity Sentinel stores and protects local operational data.'; if (next === '/terms') return 'Read the terms for Capacity Sentinel.'; return 'Monitor model API capacity, latency, and JSON response shape with synthetic canaries.'; }

  async function enterRoute(next: string, moveFocus = true) {
    path = next === '/index.html' ? '/' : next;
    document.title = routeTitle(path);
    document.querySelector('meta[name="description"]')?.setAttribute('content', routeDescription(path));
    document.querySelector('link[rel="canonical"]')?.setAttribute('href', `${location.origin}${path}`);
    loadError = '';
    if (path === '/demo') { data = loadDemo(); loading = false; accessNeeded = false; }
    else if (path === '/') { if (accessCode) load(); else { loading = false; accessNeeded = true; data = empty; } }
    else loading = false;
    if (moveFocus) { await tick(); (document.querySelector('main h1') as HTMLElement | null)?.focus(); }
  }
  function navigate(next: string) { if (location.pathname === next) return; history.pushState({}, '', next); enterRoute(next); }
  async function load(show = true) { if (demoMode) return; if (show) loading = true; try { data = await api.summary(); loadError = ''; accessNeeded = false; } catch (error) { accessNeeded = error instanceof ApiError && error.status === 401; loadError = accessNeeded ? '' : error instanceof Error ? error.message : 'Could not load observations'; } finally { loading = false; } }
  async function unlock() { if (!accessCode.trim()) return; setAccessToken(accessCode.trim()); sessionStorage.setItem('capacity-sentinel-access', accessCode.trim()); await load(); }
  function openCreate() { editing = null; form = fresh(); fieldsText = 'status, region'; formError = ''; probeDialog.showModal(); setTimeout(() => firstField?.focus()); }
  function openEdit(probe: Probe) { editing = probe; form = { name: probe.name, provider: probe.provider, endpoint_url: probe.endpoint_url, model: probe.model, api_key: '', prompt: '', required_fields: probe.required_fields, interval_minutes: probe.interval_minutes, timeout_ms: probe.timeout_ms, latency_slo_ms: probe.latency_slo_ms, availability_slo_percent: probe.availability_slo_percent, max_output_tokens: probe.max_output_tokens, daily_token_cap: probe.daily_token_cap, enabled: probe.enabled }; fieldsText = probe.required_fields.join(', '); formError = ''; probeDialog.showModal(); setTimeout(() => firstField?.focus()); }
  function addDemoProbe(input: ProbeInput) { const now = new Date().toISOString(); const id = `demo-${crypto.randomUUID()}`; return { ...data, probes: [...data.probes, { id, name: input.name, provider: input.provider, endpoint_url: input.endpoint_url, model: input.model, has_api_key: Boolean(input.api_key), required_fields: input.required_fields, interval_minutes: input.interval_minutes, timeout_ms: input.timeout_ms, latency_slo_ms: input.latency_slo_ms, availability_slo_percent: input.availability_slo_percent, max_output_tokens: input.max_output_tokens, daily_token_cap: input.daily_token_cap, enabled: input.enabled, created_at: now, updated_at: now, last_observation: null, stats: { sample_count: 0, availability_percent: 0, p95_latency_ms: 0, consecutive_failures: 0, tokens_today: 0 } }] }; }
  async function save() { busy = 'save'; formError = ''; form.required_fields = fieldsText.split(',').map((value) => value.trim()).filter(Boolean); try { if (demoMode) { data = editing ? updateDemoProbe(data, editing.id, form) : addDemoProbe(form); saveDemo(data); } else if (editing) await api.update(editing.id, form); else await api.create(form); probeDialog.close(); toast = editing ? 'Probe updated' : 'Probe saved. Its first observation is due now.'; if (!demoMode) await load(false); } catch (error) { formError = error instanceof Error ? error.message : 'Could not save the probe'; } finally { busy = ''; setTimeout(() => toast = '', 5000); } }
  async function run(probe: Probe) { busy = probe.id; try { if (demoMode) { data = runDemoProbe(data, probe.id); saveDemo(data); } else { await api.run(probe.id); await load(false); } toast = `Observation complete for ${probe.name}`; } catch (error) { toast = error instanceof Error ? error.message : 'Observation failed'; } finally { busy = ''; setTimeout(() => toast = '', 5000); } }
  function csvFromDemo() { const names = new Map(data.probes.map((probe) => [probe.id, probe])); const quote = (value: unknown) => `"${String(value ?? '').replaceAll('"', '""')}"`; const header = 'probe,provider,model,started_at,latency_ms,http_status,outcome,error_class,detail,input_tokens,output_tokens'; const rows = data.observations.map((item) => { const probe = names.get(item.probe_id); return [probe?.name, probe?.provider, probe?.model, item.started_at, item.latency_ms, item.http_status, item.outcome, item.error_class, item.detail, item.input_tokens, item.output_tokens].map(quote).join(','); }); return new Blob([[header, ...rows].join('\n')], { type: 'text/csv;charset=utf-8' }); }
  async function exportCsv() { try { const blob = demoMode ? csvFromDemo() : await api.exportCsv(); const url = URL.createObjectURL(blob); const link = document.createElement('a'); link.href = url; link.download = 'capacity-sentinel-observations.csv'; link.click(); URL.revokeObjectURL(url); } catch (error) { toast = error instanceof Error ? error.message : 'Could not export observations'; } }
  async function remove(probe: Probe) { if (confirming !== probe.id) { confirming = probe.id; setTimeout(() => confirming = '', 5000); return; } busy = probe.id; try { if (demoMode) { data = removeDemoProbe(data, probe.id); saveDemo(data); } else { await api.remove(probe.id); await load(false); } toast = `Removed ${probe.name} and its local history`; confirming = ''; } catch (error) { toast = error instanceof Error ? error.message : 'Could not remove the probe'; } finally { busy = ''; } }
  function resetSample() { data = resetDemo(); expanded = ''; confirming = ''; toast = 'Sample data reset'; setTimeout(() => toast = '', 5000); }
  async function restore() { if (!licenseInput.trim()) return; storeToken(licenseInput); license = { unlocked: false, checking: true, notice: 'Checking license…', token: licenseInput.trim() }; license = await verifyLicense(licenseInput.trim(), true); licenseInput = ''; }
  function fmt(date: string) { return new Intl.DateTimeFormat(undefined, { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' }).format(new Date(date)); }
  function status(probe: Probe) { if (!probe.enabled) return 'paused'; if (!probe.last_observation) return 'awaiting'; return probe.last_observation.outcome; }
  function probeName(id: string) { return data.probes.find((probe) => probe.id === id)?.name || 'Removed probe'; }
  function atlasStats(probe: Probe) { const rows = data.observations.filter((item) => item.probe_id === probe.id); const ok = rows.filter((item) => item.outcome === 'healthy').length; const latency = rows.filter((item) => item.http_status !== null).map((item) => item.latency_ms).sort((a, b) => a - b); return { count: rows.length, availability: rows.length ? ok / rows.length * 100 : 0, p95: latency[Math.max(0, Math.ceil(latency.length * .95) - 1)] || 0 }; }
</script>

<a class="skip" href="#main">Skip to main content</a>
<header class="site-head">
  <a class="brand" href="/" aria-label="Capacity Sentinel home" on:click|preventDefault={() => navigate('/')}><span class="brand-mark" aria-hidden="true">⌁</span><span>Capacity<br><b>Sentinel</b></span></a>
  <nav aria-label="Main navigation"><a href="/demo" on:click|preventDefault={() => navigate('/demo')}>Demo</a><a href="/#observations" on:click|preventDefault={() => { navigate('/'); setTimeout(() => document.querySelector('#observations')?.scrollIntoView()); }}>Dashboard</a><a href="/privacy" on:click|preventDefault={() => navigate('/privacy')}>Privacy</a></nav>
  {#if path === '/' || path === '/demo'}<div class="live-mark"><span class:warning={openAlerts.length > 0}></span>{demoMode ? 'Sample workspace' : openAlerts.length ? `${openAlerts.length} active alert${openAlerts.length === 1 ? '' : 's'}` : 'No active alerts'}</div>{/if}
</header>
<div class="route-status" aria-live="polite">{routeTitle(path)}</div>

{#if path === '/privacy'}
  <main id="main" class="legal"><h1 tabindex="-1">Read the privacy policy</h1><p class="lede">Capacity Sentinel keeps each project on its own runner. It does not use analytics.</p><h2>Data stored on your runner</h2><p>The runner stores probe settings, encrypted synthetic canaries and API keys, timings, status codes, token counts, validation results, and alerts in local SQLite storage. It does not store response content. Canary text is never returned by the API or shown after saving.</p><h2>Data sent outside your runner</h2><p>A probe sends its synthetic prompt and API credential only to the public endpoint you configure. Atlas license checks send a license token to Sociobot billing only after you choose to restore or verify a license.</p><h2>Your controls</h2><p>Delete a probe to remove its settings, observations, and alerts. Delete the mounted data directory to remove the installation. CSV export is available in the free product.</p><h2>Contact</h2><p>Email <a href="mailto:privacy@sociobot.in">privacy@sociobot.in</a> with privacy questions.</p></main>
{:else if path === '/terms'}
  <main id="main" class="legal"><h1 tabindex="-1">Read the terms of use</h1><p class="lede">Capacity Sentinel monitors synthetic requests. It does not guarantee provider availability or model quality.</p><h2>Use synthetic prompts</h2><p>Send only prompts you are allowed to send. Do not put customer content or personal data in a probe. You are responsible for provider terms, endpoint access, token charges, and probe intervals.</p><h2>What the monitor checks</h2><p>The runner checks status, elapsed request time, parseable JSON, and the JSON paths you name. It does not judge truthfulness, general model quality, or fitness for a particular decision.</p><h2>Atlas add-on</h2><p>Atlas is a $39 one-time paid add-on for the extended comparison view. Sociobot/Dodo is merchant of record. Refunds are handled there and revoke the license. CSV export and safety information stay free.</p><h2>Warranty and liability</h2><p>The software is provided as is under the MIT License. Check provider evidence before making high-impact decisions.</p></main>
{:else}
  <main id="main">
    {#if demoMode}<aside class="demo-banner" aria-label="Demo mode"><b>Demo — sample data, nothing is saved</b><span>Explore the sample workspace without opening your project data.</span><button class="secondary" on:click={resetSample}>Reset demo</button><button class="quiet-link" on:click={() => navigate('/')}>Start for real</button></aside>{/if}
    {#if offline}<div class="offline" role="status"><b>You are offline.</b> Showing the observations already on this screen. Reconnect to run or edit probes.</div>{/if}
    {#if toast}<div class="toast" role="status">{toast}</div>{/if}
    <section class="hero" aria-labelledby="page-title">
      <div class="hero-copy"><p class="eyebrow">Model API canary monitor · v1.1</p><h1 id="page-title" tabindex="-1">Monitor model API capacity early</h1><p class="lede">For teams running model APIs that need early evidence of failures, slower responses, and broken JSON output.</p><div class="hero-actions"><button class="primary" on:click={() => navigate('/demo')}>Try it with sample data</button><span class="action-note">See two providers, active alerts, and a recovered alert.</span></div><div class="trust-row"><span><b>Local data</b><small>SQLite on your runner</small></span><span><b>Encrypted keys</b><small>Stored on your runner</small></span><span><b>Free core</b><small>CSV export included</small></span></div></div>
      <figure class="hero-art"><img src="/assets/hero-field-guide.webp" width="1200" height="800" fetchpriority="high" decoding="async" alt="Botanical illustration representing connected model API endpoints, with one endpoint showing a warning"><figcaption>Original illustration for Capacity Sentinel</figcaption></figure>
    </section>

    <section class="dashboard" id="observations" aria-labelledby="overview-title">
      <div class="section-heading"><div><p class="eyebrow">Probe results</p><h2 id="overview-title">Current observations</h2></div><div class="actions"><button class="secondary" on:click={exportCsv}>Export CSV</button><button class="primary" on:click={openCreate}>{demoMode ? 'Add sample probe' : 'Add probe'}</button></div></div>
      {#if loading}<div class="loading" role="status"><span></span>Loading observations…</div>
      {:else if accessNeeded}<div class="state error-state"><span aria-hidden="true">⌁</span><div><h3>Open this project</h3><p>Enter the access code from the runner owner. It stays in this browser session.</p><label>Project access code<input type="password" bind:value={accessCode} autocomplete="off" on:keydown={(event) => event.key === 'Enter' && unlock()}></label><button class="secondary" on:click={unlock}>Open project</button></div></div>
      {:else if loadError}<div class="state error-state" role="alert"><span aria-hidden="true">×</span><div><h3>Could not read this project</h3><p>{loadError}. Check that the runner is online, then try again.</p><button class="secondary" on:click={() => load()}>Try again</button></div></div>
      {:else if !data.probes.length}<div class="state empty-state"><span class="empty-glyph" aria-hidden="true">⌇</span><div><h3>No probes yet</h3><p>Add a synthetic prompt and the JSON fields it must return. The runner builds a baseline after the first run.</p><button class="primary" on:click={openCreate}>Add your first probe</button></div></div>
      {:else}
        <div class="metrics" aria-label="Project summary"><article><span>Availability</span><strong>{availability.toFixed(1)}%</strong><small>Last 20 observations per probe</small></article><article><span>Healthy probes</span><strong>{healthy}<em> / {data.probes.length}</em></strong><small>{openAlerts.length ? `${openAlerts.length} alert${openAlerts.length === 1 ? '' : 's'} need attention` : 'No open alerts'}</small></article><article><span>Mean p95 latency</span><strong>{avgLatency}<em> ms</em></strong><small>Elapsed request time</small></article></div>
        {#if openAlerts.length}<section class="alerts" aria-labelledby="alerts-title"><div class="margin-label">OPEN<br>ALERTS</div><div><h3 id="alerts-title">Open alerts</h3>{#each openAlerts as alert}<article><span class="alert-pin" aria-hidden="true">!</span><div><b>{probeName(alert.probe_id)} · {alert.kind.replace('_', ' ')}</b><p>{alert.message}</p><small>Opened {fmt(alert.opened_at)}</small></div></article>{/each}</div></section>{/if}
        {#if resolvedAlerts.length}<section class="resolved-alerts" aria-labelledby="resolved-title"><h3 id="resolved-title">Recovered alerts</h3>{#each resolvedAlerts as alert}<p><b>{probeName(alert.probe_id)}</b> · {alert.message} <small>Recovered {alert.resolved_at ? fmt(alert.resolved_at) : 'recently'}</small></p>{/each}</section>{/if}
        <div class="specimens"><div class="table-head"><span>Probe / endpoint</span><span>Last outcome</span><span>Rolling evidence</span><span><span class="sr-only">Actions</span></span></div>
        {#each data.probes as probe, index}
          <article class="specimen" class:open={expanded === probe.id}>
            <div class="specimen-row"><button class="expand" aria-expanded={expanded === probe.id} aria-controls={'details-' + probe.id} on:click={() => expanded = expanded === probe.id ? '' : probe.id}><span class="number">{String(index + 1).padStart(2, '0')}</span><span><b>{probe.name}</b><small>{probe.provider} · {probe.model}</small></span><span class="chevron" aria-hidden="true">⌄</span></button><div><span class={'status ' + status(probe)}><i></i>{status(probe)}</span><small>{probe.last_observation ? fmt(probe.last_observation.started_at) : 'First run pending'}</small></div><div class="evidence"><b>{probe.stats.availability_percent.toFixed(1)}% <small>available</small></b><b>{probe.stats.p95_latency_ms} ms <small>p95</small></b><div class="ticks" role="img" aria-label={`${probe.stats.consecutive_failures} consecutive failures`}>{#each Array(Math.min(10, Math.max(1, probe.stats.sample_count))) as _, item}<i class:bad={item < probe.stats.consecutive_failures}></i>{/each}</div></div><div class="row-actions"><button class="icon-button" title={'Observe ' + probe.name} aria-label={'Observe ' + probe.name + ' now'} disabled={busy === probe.id || offline} on:click={() => run(probe)}>{busy === probe.id ? '…' : '↻'}</button><button class="icon-button" title={'Edit ' + probe.name} aria-label={'Edit ' + probe.name} on:click={() => openEdit(probe)}>✎</button></div></div>
            {#if expanded === probe.id}<div class="details" id={'details-' + probe.id}><div><p class="label">Synthetic prompt</p><p>Stored encrypted and never shown after saving.</p><p class="label">Required JSON fields</p><p>{probe.required_fields.length ? probe.required_fields.join(' · ') : 'Valid JSON only'}</p></div><div><p class="label">Settings</p><dl><div><dt>Interval</dt><dd>{probe.interval_minutes} min</dd></div><div><dt>Timeout</dt><dd>{probe.timeout_ms} ms</dd></div><div><dt>Daily token cap</dt><dd>{probe.stats.tokens_today} / {probe.daily_token_cap} tokens</dd></div><div><dt>Endpoint</dt><dd class="truncate" title={probe.endpoint_url}>{probe.endpoint_url}</dd></div></dl></div><div class="detail-actions"><button class="secondary" on:click={() => run(probe)} disabled={busy === probe.id || offline}>Run now</button><button class="quiet-danger" on:click={() => remove(probe)}>{confirming === probe.id ? 'Confirm remove' : 'Remove probe'}</button>{#if confirming === probe.id}<small>This also removes its local history.</small>{/if}</div></div>{/if}
          </article>
        {/each}</div>
      {/if}
    </section>

    {#if license.unlocked && data.probes.length}<section class="atlas-notebook" aria-labelledby="notebook-title"><div class="section-heading"><div><p class="eyebrow">Atlas · 365-day comparison</p><h2 id="notebook-title">Compare providers over time</h2></div><span class="license-ok">✓ Licensed</span></div><div class="atlas-table"><div class="atlas-row atlas-head"><span>Provider / model</span><span>Observations</span><span>Availability</span><span>p95 latency</span></div>{#each data.probes as probe}<div class="atlas-row"><b>{probe.provider}<small>{probe.model}</small></b><span>{atlasStats(probe).count}</span><span>{atlasStats(probe).availability.toFixed(1)}%</span><span>{atlasStats(probe).p95} ms</span></div>{/each}</div></section>{/if}
    <section class="method" id="how-it-works" aria-labelledby="method-title"><div class="method-intro"><p class="eyebrow">How it works</p><h2 id="method-title">Check the same synthetic request</h2><p>Capacity Sentinel stays out of your production request path. It records only the operational evidence needed to spot a change.</p></div><ol><li><span>01</span><div><h3>Add a synthetic probe</h3><p>Use a harmless monitoring prompt. Credentials are encrypted on your runner.</p></div></li><li><span>02</span><div><h3>Name required JSON fields</h3><p>List the fields that must exist. The monitor checks shape, not subjective quality.</p></div></li><li><span>03</span><div><h3>Review alerts and latency</h3><p>Two failures open an attributed alert. Rolling p95 shows slower responses.</p></div></li></ol></section>
    <section class="atlas" aria-labelledby="atlas-title"><div><p class="eyebrow">Optional Atlas add-on</p><h2 id="atlas-title">Compare providers for 365 days</h2><p>The free product runs probes, stores local evidence, opens alerts, and exports CSV. Atlas adds the extended comparison view.</p></div><div class="price"><span>One-time</span><strong>$39</strong><small>Checkout will be available after Sociobot registers this product. No subscription.</small>{#if license.unlocked}<p class="license-ok">✓ {license.notice}</p>{:else}<button class="secondary" on:click={() => planDialog.showModal()}>Restore an Atlas license</button>{/if}</div></section>
  </main>
{/if}

<footer><div class="brand mini"><span class="brand-mark" aria-hidden="true">⌁</span><span>Capacity <b>Sentinel</b></span></div><p>Synthetic monitoring for model API operators. Build {buildId}.</p><nav aria-label="Footer"><a href="/privacy" on:click|preventDefault={() => navigate('/privacy')}>Privacy</a><a href="/terms" on:click|preventDefault={() => navigate('/terms')}>Terms</a><a href="https://github.com/B-Divyesh/sf-model-capacity-sentinel">Source <span class="sr-only">(opens an external site)</span></a></nav></footer>

<dialog class="probe-dialog" bind:this={probeDialog} on:close={() => { formError = ''; confirming = ''; }}><form method="dialog" on:submit={(event) => { event.preventDefault(); save(); }}><header><div><p class="eyebrow">{editing ? 'Update probe' : 'New probe'}</p><h2>{editing ? 'Edit probe' : 'Add a probe'}</h2></div><button type="button" class="close" aria-label="Close dialog" on:click={() => probeDialog.close()}>×</button></header><p class="dialog-intro">Use a synthetic prompt only. This runner supports OpenAI-compatible chat completion endpoints.</p><div class="form-grid"><label>Probe name<input bind:this={firstField} bind:value={form.name} required maxlength="80" placeholder="EU capacity check"></label><label>Provider label<input bind:value={form.provider} required maxlength="50" placeholder="Provider or region"></label><label class="wide">Chat completion endpoint<input type="url" bind:value={form.endpoint_url} required aria-describedby="endpoint-help"><small id="endpoint-help">Use a public HTTP(S) endpoint. Private, loopback, and link-local addresses are blocked.</small></label><label>Model<input bind:value={form.model} required placeholder="model-name"></label><label>API key<input type="password" bind:value={form.api_key} required={!editing} autocomplete="off" placeholder={editing ? 'Leave blank to keep the stored key' : 'sk-…'}><small>Encrypted before SQLite storage.</small></label><label class="wide">Synthetic prompt<textarea bind:value={form.prompt} required={!editing} rows="3" placeholder={editing ? 'Leave blank to keep the encrypted prompt' : 'A harmless monitoring question'}></textarea><small>Encrypted before storage and never shown again.</small></label><label class="wide">Required JSON paths<input bind:value={fieldsText} placeholder="status, result.label"><small>Comma-separated dot paths. Leave empty to require valid JSON only.</small></label><fieldset class="wide"><legend>Schedule and objectives</legend><label>Every (minutes)<input type="number" min="1" max="1440" bind:value={form.interval_minutes}></label><label>Timeout (ms)<input type="number" min="1000" max="120000" step="500" bind:value={form.timeout_ms}></label><label>p95 target (ms)<input type="number" min="100" max="120000" step="100" bind:value={form.latency_slo_ms}></label><label>Availability target (%)<input type="number" min="1" max="100" step="0.1" bind:value={form.availability_slo_percent}></label><label>Max output tokens<input type="number" min="1" max="8192" bind:value={form.max_output_tokens}></label><label>Daily token cap<input type="number" min="1" max="10000000" bind:value={form.daily_token_cap}></label></fieldset><label class="check wide"><input type="checkbox" bind:checked={form.enabled}><span>Run this probe on schedule</span></label></div>{#if formError}<p class="form-error" role="alert">{formError}</p>{/if}<footer><button type="button" class="secondary" on:click={() => probeDialog.close()}>Cancel</button><button class="primary" type="submit" disabled={busy === 'save'}>{busy === 'save' ? 'Saving…' : editing ? 'Save changes' : 'Save probe'}</button></footer></form></dialog>
<dialog class="plan-dialog" bind:this={planDialog}><form method="dialog"><header><div><p class="eyebrow">Atlas license</p><h2>Restore a paid license</h2></div><button class="close" value="cancel" aria-label="Close dialog">×</button></header><p>Atlas is a <b>$39 one-time paid add-on</b> for the extended comparison view. Sociobot/Dodo handles checkout and refunds when product billing is available.</p>{#if license.unlocked}<p class="license-ok">✓ Atlas license active in this browser.</p>{:else}<div class="restore"><label for="license">Paste an Atlas license</label><input id="license" bind:value={licenseInput} autocomplete="off" placeholder="License token"><button type="button" class="secondary" on:click={restore} disabled={license.checking}>{license.checking ? 'Verifying…' : 'Verify license'}</button></div>{#if license.notice}<p class="license-notice" role="status">{license.notice}</p>{/if}{/if}<small>By purchasing, you agree to the <a href="/terms" on:click|preventDefault={() => { planDialog.close(); navigate('/terms'); }}>terms</a>. See the <a href="/privacy" on:click|preventDefault={() => { planDialog.close(); navigate('/privacy'); }}>privacy policy</a>.</small></form></dialog>
