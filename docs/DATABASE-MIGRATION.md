# Database Migration Guide

## Overview

This guide covers database migration procedures for Hermes Game Operator.

---

## Migration Principles

### 1. Always Backup

```bash
# Backup before migration
sqlite3 ~/.config/hermes-operator/data.db ".backup '/tmp/backup-before-migration.db'"
```

### 2. Use Transactions

```sql
BEGIN TRANSACTION;
-- Migration SQL
COMMIT;
```

### 3. Test in Staging

```bash
# Test migration in staging first
npm run db:migrate:staging
```

### 4. Monitor Performance

```sql
-- Check query performance after migration
EXPLAIN QUERY PLAN SELECT * FROM tasks WHERE status = 'running';
```

---

## Migration Workflow

### Step 1: Create Migration File

```bash
# Create new migration
npm run db:create-migration add_indexes
```

**File**: `migrations/20260708_add_indexes.sql`

```sql
-- Migration: Add indexes for performance
-- Date: 2026-07-08
-- Description: Add indexes to improve query performance

BEGIN TRANSACTION;

-- Add index on tasks.status
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);

-- Add index on tasks.created_at
CREATE INDEX IF NOT EXISTS idx_tasks_created_at ON tasks(created_at);

-- Add index on events.task_id
CREATE INDEX IF NOT EXISTS idx_events_task_id ON events(task_id);

COMMIT;
```

---

### Step 2: Test Migration

```bash
# Test migration
npm run db:migrate:test
```

---

### Step 3: Apply Migration

```bash
# Apply migration
npm run db:migrate
```

---

### Step 4: Verify Migration

```bash
# Verify migration
npm run db:verify
```

---

## Common Migrations

### Add Column

```sql
BEGIN TRANSACTION;

ALTER TABLE tasks ADD COLUMN priority INTEGER DEFAULT 0;

COMMIT;
```

### Rename Column

```sql
BEGIN TRANSACTION;

-- SQLite doesn't support RENAME COLUMN directly
-- Use this workaround:

-- 1. Create new table with new schema
CREATE TABLE tasks_new (
  task_id TEXT PRIMARY KEY,
  task_name TEXT,  -- renamed from 'name'
  status TEXT,
  created_at TEXT
);

-- 2. Copy data
INSERT INTO tasks_new (task_id, task_name, status, created_at)
SELECT task_id, name, status, created_at FROM tasks;

-- 3. Drop old table
DROP TABLE tasks;

-- 4. Rename new table
ALTER TABLE tasks_new RENAME TO tasks;

COMMIT;
```

### Add Index

```sql
BEGIN TRANSACTION;

CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);

COMMIT;
```

### Create Table

```sql
BEGIN TRANSACTION;

CREATE TABLE IF NOT EXISTS approvals (
  approval_id TEXT PRIMARY KEY,
  task_id TEXT NOT NULL,
  level TEXT NOT NULL,
  decision TEXT,
  created_at TEXT NOT NULL,
  FOREIGN KEY (task_id) REFERENCES tasks(task_id)
);

COMMIT;
```

### Drop Table

```sql
BEGIN TRANSACTION;

DROP TABLE IF EXISTS temp_data;

COMMIT;
```

---

## Migration Scripts

### Automatic Migration

```typescript
// scripts/migrate.ts
import fs from 'fs'
import path from 'path'
import sqlite3 from 'sqlite3'

const db = new sqlite3.Database('data.db')

async function runMigrations() {
  // Get list of migrations
  const migrationsDir = path.join(__dirname, '../migrations')
  const migrations = fs.readdirSync(migrationsDir)
    .filter(f => f.endsWith('.sql'))
    .sort()
  
  // Get applied migrations
  const applied = await getAppliedMigrations()
  
  // Run pending migrations
  for (const migration of migrations) {
    if (!applied.includes(migration)) {
      console.log(`Running migration: ${migration}`)
      await runMigration(migration)
      await recordMigration(migration)
    }
  }
}

async function runMigration(filename: string) {
  const sql = fs.readFileSync(
    path.join(__dirname, '../migrations', filename),
    'utf-8'
  )
  
  return new Promise((resolve, reject) => {
    db.exec(sql, (err) => {
      if (err) reject(err)
      else resolve(null)
    })
  })
}

async function getAppliedMigrations(): Promise<string[]> {
  return new Promise((resolve, reject) => {
    db.all(
      'SELECT filename FROM migrations ORDER BY applied_at',
      (err, rows) => {
        if (err) reject(err)
        else resolve(rows.map((r: any) => r.filename))
      }
    )
  })
}

async function recordMigration(filename: string) {
  return new Promise((resolve, reject) => {
    db.run(
      'INSERT INTO migrations (filename, applied_at) VALUES (?, ?)',
      [filename, new Date().toISOString()],
      (err) => {
        if (err) reject(err)
        else resolve(null)
      }
    )
  })
}

runMigrations().catch(console.error)
```

---

### Rollback Migration

```typescript
// scripts/rollback.ts
import fs from 'fs'
import path from 'path'
import sqlite3 from 'sqlite3'

const db = new sqlite3.Database('data.db')

async function rollbackMigration() {
  // Get last applied migration
  const last = await getLastMigration()
  
  if (!last) {
    console.log('No migrations to rollback')
    return
  }
  
  console.log(`Rolling back: ${last}`)
  
  // Run rollback SQL
  const rollbackFile = last.replace('.sql', '.rollback.sql')
  const rollbackPath = path.join(__dirname, '../migrations', rollbackFile)
  
  if (fs.existsSync(rollbackPath)) {
    const sql = fs.readFileSync(rollbackPath, 'utf-8')
    await runRollback(sql)
    await removeMigrationRecord(last)
    console.log('Rollback complete')
  } else {
    console.log('No rollback script found')
  }
}

async function getLastMigration(): Promise<string | null> {
  return new Promise((resolve, reject) => {
    db.get(
      'SELECT filename FROM migrations ORDER BY applied_at DESC LIMIT 1',
      (err, row: any) => {
        if (err) reject(err)
        else resolve(row?.filename || null)
      }
    )
  })
}

async function runRollback(sql: string) {
  return new Promise((resolve, reject) => {
    db.exec(sql, (err) => {
      if (err) reject(err)
      else resolve(null)
    })
  })
}

async function removeMigrationRecord(filename: string) {
  return new Promise((resolve, reject) => {
    db.run(
      'DELETE FROM migrations WHERE filename = ?',
      [filename],
      (err) => {
        if (err) reject(err)
        else resolve(null)
      }
    )
  })
}

rollbackMigration().catch(console.error)
```

---

## Migration Best Practices

### 1. Use Idempotent Operations

```sql
-- Good: IF NOT EXISTS
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);

-- Bad: May fail if index exists
CREATE INDEX idx_tasks_status ON tasks(status);
```

### 2. Handle Large Tables

```sql
-- For large tables, migrate in batches
BEGIN TRANSACTION;

-- Process in batches of 1000
INSERT INTO tasks_new
SELECT * FROM tasks WHERE rowid < 1000;

COMMIT;

-- Repeat for next batch
```

### 3. Test Data Integrity

```sql
-- Verify data after migration
SELECT COUNT(*) FROM tasks;
SELECT COUNT(*) FROM events;

-- Check foreign keys
PRAGMA foreign_key_check;
```

### 4. Document Migrations

```sql
-- Add comments
-- Migration: Add priority column
-- Date: 2026-07-08
-- Description: Add priority field to tasks for better task management
-- Author: John Doe
```

---

## Monitoring

### Track Migration Status

```sql
-- View applied migrations
SELECT * FROM migrations ORDER BY applied_at DESC;

-- Check migration duration
SELECT 
  filename,
  applied_at,
  strftime('%s', applied_at) - strftime('%s', started_at) as duration_seconds
FROM migrations;
```

### Monitor Performance

```sql
-- Check query performance
EXPLAIN QUERY PLAN SELECT * FROM tasks WHERE status = 'running';

-- Analyze database
ANALYZE;

-- Check index usage
SELECT * FROM sqlite_master WHERE type = 'index';
```

---

## Troubleshooting

### Migration Fails

```bash
# Check error logs
tail -100 ~/.config/hermes-operator/logs/migration.log

# Verify database integrity
sqlite3 data.db "PRAGMA integrity_check;"

# Restore from backup
cp /tmp/backup-before-migration.db ~/.config/hermes-operator/data.db
```

### Performance Degradation

```sql
-- Rebuild indexes
REINDEX;

-- Update statistics
ANALYZE;

-- Vacuum database
VACUUM;
```

### Data Loss

```bash
# Restore from backup
cp /tmp/backup-before-migration.db ~/.config/hermes-operator/data.db

# Verify data
sqlite3 data.db "SELECT COUNT(*) FROM tasks;"
```

---

## Version Control

### Track Migrations in Git

```bash
# Add migration files
git add migrations/*.sql

# Commit migration
git commit -m "feat: add indexes for performance"

# Push to repository
git push origin main
```

### Migration Naming Convention

```
YYYYMMDD_description.sql
20260708_add_indexes.sql
20260709_create_approvals_table.sql
```

---

## Resources

- [SQLite Migration Guide](https://www.sqlite.org/lang_altertable.html)
- [Database Migration Best Practices](https://flywaydb.org/documentation/concepts/migrations)
- [Migration Testing](https://www.martinfowler.com/bliki/Flyway.html)

---

**Guide Version**: 1.0.0  
**Last Updated**: 2026-07-08  
**Status**: Ready to use
