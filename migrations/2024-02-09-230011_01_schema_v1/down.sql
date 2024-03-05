-- This file should undo anything in `up.sql`
DROP SCHEMA IF EXISTS iri CASCADE;
REVOKE ALL PRIVILEGES ON DATABASE iridb FROM iriuser;
DROP DATABASE IF EXISTS iridb;
DROP USER IF EXISTS iriuser;