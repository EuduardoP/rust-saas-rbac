set dotenv-load

# List all commands
default:
    @just --list

# Start docker database
db-start:
    docker-compose up -d

# Stop database and remove volumes
db-stop:
    docker-compose down -v

db-studio:
    rainfrog -u {{env_var('DATABASE_URL')}}

# Run app Dioxus (hot reload)
dev:
    dx serve -p web

# Run migrations
migrate:
    sea-orm-cli migrate up --migration-dir packages/migration

# Undo migrations
migrate-down:
    sea-orm-cli migrate down --migration-dir packages/migration

# Generate entities
generate:
    sea-orm-cli generate entity -u {{env_var('DATABASE_URL')}} -o packages/api/src/db/entities --with-serde both

# Run both migration and generate
db-push: migrate generate

