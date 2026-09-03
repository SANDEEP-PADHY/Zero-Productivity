-- Initial Schema for Productivity Tracker
-- Matches DATA_MODEL.md with strict RLS policies.

-- Revoke the broad default privileges that the Supabase Postgres image grants to anon and authenticated
ALTER DEFAULT PRIVILEGES IN SCHEMA public REVOKE ALL ON TABLES FROM anon, authenticated;
ALTER DEFAULT PRIVILEGES IN SCHEMA public REVOKE ALL ON ROUTINES FROM anon, authenticated;
ALTER DEFAULT PRIVILEGES IN SCHEMA public REVOKE ALL ON SEQUENCES FROM anon, authenticated;
DO $$
BEGIN
  CREATE ROLE anon NOLOGIN;
  EXCEPTION WHEN OTHERS THEN RAISE NOTICE 'anon role exists';
END
$$;
DO $$
BEGIN
  CREATE ROLE authenticated NOLOGIN;
  EXCEPTION WHEN OTHERS THEN RAISE NOTICE 'authenticated role exists';
END
$$;
GRANT USAGE ON SCHEMA public TO authenticated;
-- We will grant specific CRUD permissions on tables at the bottom of the file

-- Create an updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- ==========================================
-- DEVICES
-- ==========================================
CREATE TABLE devices (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL DEFAULT auth.uid() REFERENCES auth.users(id) ON DELETE CASCADE,
    device_name TEXT NOT NULL,
    platform TEXT NOT NULL,
    os_version TEXT,
    app_version TEXT,
    first_registered_at TIMESTAMPTZ NOT NULL,
    last_sync_at TIMESTAMPTZ,
    last_activity_sync_at TIMESTAMPTZ,
    settings_mode TEXT NOT NULL,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TRIGGER update_devices_updated_at BEFORE UPDATE ON devices FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

ALTER TABLE devices ENABLE ROW LEVEL SECURITY;
CREATE POLICY "Users can manage their own devices" ON devices FOR ALL USING (auth.uid() = user_id);

-- ==========================================
-- ACTIVITY SESSIONS
-- ==========================================
CREATE TABLE activity_sessions (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL DEFAULT auth.uid() REFERENCES auth.users(id) ON DELETE CASCADE,
    device_id UUID NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    source TEXT NOT NULL,
    activity_type TEXT,
    application_id TEXT,
    application_name TEXT,
    browser_name TEXT,
    domain TEXT,
    url TEXT,
    title TEXT,
    started_at TIMESTAMPTZ NOT NULL,
    ended_at TIMESTAMPTZ NOT NULL,
    duration_ms BIGINT NOT NULL,
    foreground_ms BIGINT,
    interaction_ms BIGINT,
    media_ms BIGINT,
    idle_ms BIGINT,
    classification TEXT,
    confidence FLOAT,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Activity sessions are immutable, so no updated_at trigger needed

ALTER TABLE activity_sessions ENABLE ROW LEVEL SECURITY;
CREATE POLICY "Users can manage their own activity sessions" ON activity_sessions FOR ALL USING (auth.uid() = user_id);

-- ==========================================
-- SETTINGS
-- ==========================================
CREATE TABLE settings (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL DEFAULT auth.uid() REFERENCES auth.users(id) ON DELETE CASCADE,
    device_id UUID REFERENCES devices(id) ON DELETE CASCADE,
    version BIGINT NOT NULL,
    mode TEXT NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TRIGGER update_settings_updated_at BEFORE UPDATE ON settings FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

ALTER TABLE settings ENABLE ROW LEVEL SECURITY;
CREATE POLICY "Users can manage their own settings" ON settings FOR ALL USING (auth.uid() = user_id);

-- ==========================================
-- TOMBSTONES
-- ==========================================
CREATE TABLE tombstones (
    id UUID PRIMARY KEY,
    target_record_id UUID NOT NULL,
    user_id UUID NOT NULL DEFAULT auth.uid() REFERENCES auth.users(id) ON DELETE CASCADE,
    device_id UUID REFERENCES devices(id) ON DELETE CASCADE,
    deletion_timestamp TIMESTAMPTZ NOT NULL,
    schema_version INT NOT NULL,
    sync_state TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TRIGGER update_tombstones_updated_at BEFORE UPDATE ON tombstones FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

ALTER TABLE tombstones ENABLE ROW LEVEL SECURITY;
CREATE POLICY "Users can manage their own tombstones" ON tombstones FOR ALL USING (auth.uid() = user_id);

-- Explicitly grant minimal permissions required for PostgREST
GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE devices, activity_sessions, settings, tombstones TO authenticated;
