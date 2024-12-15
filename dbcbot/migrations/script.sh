#!/bin/bash

# Set your PostgreSQL username and database name
USERNAME=""
DATABASE="database_name"
SQL_DIR="path/to/your/sql/files"

# Loop through all .sql files in the specified directory
for sql_file in "$SQL_DIR"/*.sql; do
  echo "Executing $sql_file..."
  psql -U "$USERNAME" -d "$DATABASE" -f "$sql_file"
done
