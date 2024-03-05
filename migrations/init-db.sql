--EXECUTE ON PSQL SERVER
-- Create superuser
CREATE ROLE rust_iri WITH LOGIN SUPERUSER PASSWORD 'iri12345';

-- Create database
CREATE DATABASE rust_iri_db OWNER rust_iri;

-- Connect to the database \c rust_iri_db


-- Create schema
CREATE SCHEMA IF NOT EXISTS polygon;

-- Install TimescaleDB extension
CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;


