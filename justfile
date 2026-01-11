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
serve:
    dx serve -p web

tailwind:
    npx @tailwindcss/cli -i ./tailwind.css -o ./packages/web/assets/tailwind.css --watch

server:
  cargo watch --env-file .env -q -x 'run -p server --quiet'

dev:
  mprocs "just tailwind" "just serve" "just server" "cargo run --bin gen-openapi -p server"

# Run migrations
migrate:
    sea-orm-cli migrate up --migration-dir packages/migration

# Undo migrations
migrate-down:
    sea-orm-cli migrate down --migration-dir packages/migration

# Generate entities
generate:
    sea-orm-cli generate entity -u {{env_var('DATABASE_URL')}} -o packages/entities/src --with-serde both

generate-doc:
    cargo run --bin gen-openapi -p server

# Run both migration and generate
db-push: migrate generate

