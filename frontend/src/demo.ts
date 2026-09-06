import type { Alert, Observation, Probe, ProbeInput, Summary } from "./types";

const demoKey = "demo:capacity-sentinel:summary";
const stamp = "2026-09-06T09:30:00.000Z";

function observation(
  partial: Partial<Observation> & Pick<Observation, "id" | "probe_id">,
): Observation {
  return {
    started_at: stamp,
    latency_ms: 0,
    http_status: 200,
    outcome: "healthy",
    error_class: null,
    detail: "All required fields present",
    input_tokens: 42,
    output_tokens: 18,
    invariant_valid: 1,
    ...partial,
  };
}

function probe(
  partial: Partial<Probe> & Pick<Probe, "id" | "name" | "provider" | "model">,
): Probe {
  const { id, name, provider, model, ...rest } = partial;
  const last = rest.last_observation ?? null;
  return {
    id,
    name,
    provider,
    endpoint_url: "https://api.example-model.test/v1/chat/completions",
    model,
    has_api_key: true,
    required_fields: ["status", "region"],
    interval_minutes: 5,
    timeout_ms: 15000,
    latency_slo_ms: 1800,
    availability_slo_percent: 99,
    max_output_tokens: 96,
    daily_token_cap: 5000,
    enabled: true,
    created_at: stamp,
    updated_at: stamp,
    last_observation: last,
    stats: {
      sample_count: 20,
      availability_percent: 100,
      p95_latency_ms: 712,
      consecutive_failures: 0,
      tokens_today: 1240,
    },
    ...rest,
  };
}

export function initialDemo(): Summary {
  const primaryId = "demo-primary-us";
  const europeId = "demo-eu-structured";
  const primary = observation({
    id: "demo-observation-primary",
    probe_id: primaryId,
    latency_ms: 712,
    detail: "All required fields present",
  });
  const capacity = observation({
    id: "demo-observation-capacity",
    probe_id: europeId,
    latency_ms: 1240,
    http_status: 429,
    outcome: "error",
    error_class: "capacity",
    detail: "Provider returned HTTP 429",
    invariant_valid: 0,
  });
  const recovered = observation({
    id: "demo-observation-recovered",
    probe_id: primaryId,
    started_at: "2026-09-06T07:30:00.000Z",
    latency_ms: 1610,
    detail: "All required fields present",
  });
  const alerts: Alert[] = [
    {
      id: "demo-alert-capacity",
      probe_id: europeId,
      kind: "availability",
      status: "open",
      message:
        "2 consecutive capacity failures on Example AI/eu-json-2026 (capacity)",
      opened_at: "2026-09-06T09:25:00.000Z",
      resolved_at: null,
    },
    {
      id: "demo-alert-recovered",
      probe_id: primaryId,
      kind: "latency",
      status: "resolved",
      message: "p95 latency recovered below the 1,800 ms objective",
      opened_at: "2026-09-05T15:00:00.000Z",
      resolved_at: "2026-09-06T07:30:00.000Z",
    },
  ];
  return {
    probes: [
      probe({
        id: primaryId,
        name: "North America chat availability",
        provider: "Example AI",
        model: "chat-pro-2026",
        last_observation: primary,
        stats: {
          sample_count: 20,
          availability_percent: 100,
          p95_latency_ms: 712,
          consecutive_failures: 0,
          tokens_today: 1240,
        },
      }),
      probe({
        id: europeId,
        name: "Europe structured output",
        provider: "Example AI",
        model: "eu-json-2026",
        endpoint_url: "https://eu.example-model.test/v1/chat/completions",
        required_fields: ["status", "result.label"],
        latency_slo_ms: 1400,
        last_observation: capacity,
        stats: {
          sample_count: 20,
          availability_percent: 90,
          p95_latency_ms: 1240,
          consecutive_failures: 2,
          tokens_today: 980,
        },
      }),
    ],
    alerts,
    observations: [capacity, primary, recovered],
  };
}

export function loadDemo(): Summary {
  try {
    const stored = localStorage.getItem(demoKey);
    if (stored) return JSON.parse(stored) as Summary;
    const sample = initialDemo();
    saveDemo(sample);
    return sample;
  } catch {
    return initialDemo();
  }
}

export function saveDemo(summary: Summary): void {
  localStorage.setItem(demoKey, JSON.stringify(summary));
}

export function resetDemo(): Summary {
  const sample = initialDemo();
  saveDemo(sample);
  return sample;
}

export function updateDemoProbe(
  summary: Summary,
  id: string,
  input: ProbeInput,
): Summary {
  return {
    ...summary,
    probes: summary.probes.map((item) =>
      item.id === id
        ? {
            ...item,
            name: input.name,
            provider: input.provider,
            endpoint_url: input.endpoint_url,
            model: input.model,
            required_fields: input.required_fields,
            interval_minutes: input.interval_minutes,
            timeout_ms: input.timeout_ms,
            latency_slo_ms: input.latency_slo_ms,
            availability_slo_percent: input.availability_slo_percent,
            max_output_tokens: input.max_output_tokens,
            daily_token_cap: input.daily_token_cap,
            enabled: input.enabled,
            updated_at: new Date().toISOString(),
          }
        : item,
    ),
  };
}

export function runDemoProbe(summary: Summary, id: string): Summary {
  const current = summary.probes.find((item) => item.id === id);
  if (!current) return summary;
  const next = observation({
    id: `demo-run-${Date.now()}`,
    probe_id: id,
    started_at: new Date().toISOString(),
    latency_ms: current.last_observation?.outcome === "error" ? 834 : 694,
    detail: "All required fields present",
  });
  return {
    probes: summary.probes.map((item) =>
      item.id === id
        ? {
            ...item,
            last_observation: next,
            stats: {
              ...item.stats,
              sample_count: item.stats.sample_count + 1,
              availability_percent: Math.min(
                100,
                item.stats.availability_percent + 1,
              ),
              consecutive_failures: 0,
              p95_latency_ms: Math.max(
                item.stats.p95_latency_ms,
                next.latency_ms,
              ),
            },
          }
        : item,
    ),
    alerts: summary.alerts.map((alert) =>
      alert.probe_id === id && alert.status === "open"
        ? {
            ...alert,
            status: "resolved" as const,
            resolved_at: next.started_at,
            message: `${alert.message} Recovered in the sample.`,
          }
        : alert,
    ),
    observations: [next, ...summary.observations],
  };
}

export function removeDemoProbe(summary: Summary, id: string): Summary {
  return {
    probes: summary.probes.filter((item) => item.id !== id),
    alerts: summary.alerts.filter((item) => item.probe_id !== id),
    observations: summary.observations.filter((item) => item.probe_id !== id),
  };
}
