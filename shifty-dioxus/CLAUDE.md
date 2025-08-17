# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Shifty is a shift planning and employee management web application built with Dioxus (Rust-based web framework). It manages employee schedules, working hours, billing periods, and provides multi-language support (English, German, Czech).

## Essential Commands

### Development
```bash
# Start Tailwind CSS compiler (required for styling)
npx tailwindcss -i ./input.css -o ./assets/tailwind.css --watch

# Run development server with hot reload
dx serve --hot-reload

# Build the application
dx build

# Clean build artifacts
dx clean
```

### Code Quality
```bash
# Format code
cargo fmt

# Run clippy linter
cargo clippy

# Type check
cargo check

# Run tests
cargo test
```

## Architecture Overview

### Frontend Stack
- **Dioxus**: React-like framework for Rust
- **Tailwind CSS**: Utility-first CSS with custom zoom classes (scale-down-50, scale-down-75, scale-down-100)
- **Multi-language i18n**: Located in `src/i18n/` with keys in `mod.rs` and translations in `en.rs`, `de.rs`, `cs.rs`

### Key Architectural Patterns

1. **Component-Service-State Pattern**:
   - **Components** (`src/component/`): UI components using Dioxus RSX syntax
   - **Services** (`src/service/`): Business logic and API communication with coroutines
   - **State** (`src/state/`): Data structures and domain models
   - **Pages** (`src/page/`): Full page components that compose smaller components

2. **API Communication**:
   - `src/api.rs`: REST API client functions
   - `src/loader.rs`: Data loading utilities
   - `rest-types/`: Shared types between frontend and backend
   - Backend proxy configuration in `Dioxus.toml`

3. **Routing**: Defined in `src/router.rs` using Dioxus Router

### Critical Components

**WeekView** (`src/component/week_view.rs`):
- Core shift planning view with sticky time column
- Uses CSS `zoom` property for zoom functionality (not `transform: scale`)
- Implements horizontal scrolling for weekdays while keeping time column fixed

**i18n System**:
- All translations must be added to all three locales (En, De, Cs)
- German translations previously had a bug where they used `Locale::En` instead of `Locale::De` (now fixed)
- Translation keys are defined in `src/i18n/mod.rs` enum `Key`

**Billing Period** (`src/page/billing_period_details.rs`):
- Displays sales person values with translations for BALANCE, EXPECTED_HOURS, OVERALL
- Formats dates using `i18n.format_date()` 
- Rounds monetary values to 2 decimal places

### Common Issues & Solutions

1. **Zoom gaps in WeekView**: Use CSS `zoom` property, not `transform: scale`
2. **German translations not working**: Ensure using `Locale::De` not `Locale::En`
3. **WASM validation errors**: Run `dx clean` before rebuilding
4. **Time column width issues**: Adjust `min-w-*` classes in TimeView component

### Backend Configuration

The application expects a backend server running on `http://localhost:3000` with endpoints defined in `Dioxus.toml` proxy configuration.

### Development Notes

- The application serves on `http://localhost:8080` by default
- Tailwind CSS must be running in watch mode during development
- Custom Tailwind colors are defined in `tailwind.config.js` (missingColor, blockedColor)
- Print-specific styles are configured for shift plan printing