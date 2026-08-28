export type Observation={id:string;probe_id:string;started_at:string;latency_ms:number;http_status:number|null;outcome:'healthy'|'error'|'blocked';error_class:string|null;detail:string|null;input_tokens:number|null;output_tokens:number|null;invariant_valid:number};
export type Stats={sample_count:number;availability_percent:number;p95_latency_ms:number;consecutive_failures:number;tokens_today:number};
export type Probe={id:string;name:string;provider:string;endpoint_url:string;model:string;has_api_key:boolean;prompt:string;required_fields:string[];interval_minutes:number;timeout_ms:number;latency_slo_ms:number;availability_slo_percent:number;max_output_tokens:number;daily_token_cap:number;enabled:boolean;created_at:string;updated_at:string;last_observation:Observation|null;stats:Stats};
export type Alert={id:string;probe_id:string;kind:string;status:'open'|'resolved';message:string;opened_at:string;resolved_at:string|null};
export type Summary={probes:Probe[];alerts:Alert[];observations:Observation[]};
export type ProbeInput=Omit<Probe,'id'|'has_api_key'|'created_at'|'updated_at'|'last_observation'|'stats'> & {api_key:string};
