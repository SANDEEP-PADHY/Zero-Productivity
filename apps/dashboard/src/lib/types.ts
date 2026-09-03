export type Device = {
  id: string;
  user_id: string;
  device_name: string;
  platform: string;
  os_version: string | null;
  app_version: string | null;
  first_registered_at: string;
  last_sync_at: string | null;
  last_activity_sync_at: string | null;
  settings_mode: string;
  revoked_at: string | null;
  created_at: string;
  updated_at: string;
};

export type ActivitySession = {
  id: string;
  user_id: string;
  device_id: string;
  source: string;
  activity_type: string | null;
  application_id: string | null;
  application_name: string | null;
  browser_name: string | null;
  domain: string | null;
  url: string | null;
  title: string | null;
  started_at: string;
  ended_at: string;
  duration_ms: number;
  foreground_ms: number | null;
  interaction_ms: number | null;
  media_ms: number | null;
  idle_ms: number | null;
  classification: string | null;
  confidence: number | null;
  metadata: Record<string, unknown> | null;
  created_at: string;
};

export type Setting = {
  id: string;
  user_id: string;
  device_id: string | null;
  version: number;
  mode: string;
  payload: Record<string, unknown>;
  created_at: string;
  updated_at: string;
};

export type Tombstone = {
  id: string;
  target_record_id: string;
  user_id: string;
  device_id: string | null;
  deletion_timestamp: string;
  schema_version: number;
  sync_state: string;
  created_at: string;
  updated_at: string;
};
