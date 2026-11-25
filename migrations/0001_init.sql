CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('admin','ingest','viewer')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS devices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    fleet_id TEXT NOT NULL,
    external_id TEXT NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(fleet_id, external_id)
);
CREATE INDEX IF NOT EXISTS idx_devices_fleet_id ON devices(fleet_id);

CREATE TABLE IF NOT EXISTS telemetry (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    device_id UUID NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    observed_at TIMESTAMPTZ NOT NULL,
    metric TEXT NOT NULL,
    value DOUBLE PRECISION NOT NULL,
    unit TEXT,
    attributes JSONB,
    received_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_telemetry_device_observed ON telemetry(device_id, observed_at DESC);
CREATE INDEX IF NOT EXISTS idx_telemetry_metric_observed ON telemetry(metric, observed_at DESC);

CREATE TABLE IF NOT EXISTS alert_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    device_id UUID REFERENCES devices(id) ON DELETE CASCADE,
    metric TEXT NOT NULL,
    operator TEXT NOT NULL CHECK (operator IN ('gt','gte','lt','lte','eq')),
    threshold DOUBLE PRECISION NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
