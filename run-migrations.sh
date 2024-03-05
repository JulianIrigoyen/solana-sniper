#!/bin/bash

# Wait for PostgreSQL to become available.
while ! pg_isready -h localhost -p 5432 -U iriuser; do
    echo "Waiting for PostgreSQL to become available..."
    sleep 2
done

# Ensure the Rust environment is sourced for this session
source $HOME/.cargo/env

# Run Diesel migrations
diesel migration run --database-url postgres://iriuser:iripw12345@localhost/iridb
