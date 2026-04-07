# FastClient CRM (Rust)

A lightweight, fast Customer Relationship Management system built with Rust, HTMX, and Tailwind CSS.

## Features

- **Customer Management** - Add, edit, view, and delete customer records
- **Status Tracking** - Track customers through stages: New, Contacted, Callback, Follow Up
- **Notes System** - Add timestamped notes to customer records
- **CSV Export** - Export data for reporting
- **Search & Filter** - Quick search across customer fields, filter by status
- **Responsive UI** - Clean, modern interface that works on desktop and mobile
- **User Authentication** - Secure login with session-based authentication

## Tech Stack

- **Rust** - Backend with axum web framework
- **PostgreSQL** - Relational database
- **HTMX** - Dynamic interactions without complex JavaScript
- **Tailwind CSS v4** - Utility-first styling
- **Askama** - Server-side HTML templates

## Requirements

- Rust 1.75+
- PostgreSQL 12+
- Bun runtime (for asset building)

## Quick Start

1. Copy environment file:
   ```bash
   cp .env.example .env
   ```

2. Configure database credentials in `.env`

3. Create the database:
   ```bash
   createdb fastclient
   ```

4. Build and run:
   ```bash
   cargo run
   ```

5. Visit `http://localhost:8000`

## Development

```bash
# Run with hot reload (requires cargo-watch)
cargo watch -x run

# Build assets
bun install
bun run build
```

## Project Structure

```
fastclient-exp/
├── src/
│   ├── auth/              # Authentication middleware
│   ├── config/            # Application settings
│   ├── database/          # Connection pool + migrations
│   ├── handlers/          # Route handlers
│   ├── models/            # Data models (User, Customer, CustomerNote)
│   ├── static/            # CSS/JS assets
│   └── main.rs            # Application entry point
├── templates/             # Askama HTML templates
├── migrations/            # SQL migration files
└── Cargo.toml
```

## Customer Fields

| Field | Description |
|-------|-------------|
| Name | Customer/company name |
| Email | Contact email |
| Phone | Contact phone number |
| Website | Company website (optional) |
| City | City location |
| State | State/region |
| Industry | Business industry (optional) |
| Status | Current pipeline status |

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| APP_NAME | Application name | FastClient |
| APP_ENV | Environment (development/production) | development |
| APP_DEBUG | Enable debug mode | true |
| APP_URL | Application URL | http://localhost:8000 |
| APP_ADDR | Listen address | 0.0.0.0:8000 |
| DATABASE_URL | PostgreSQL connection string | - |
| RUST_LOG | Logging level | debug |

## License

MIT
