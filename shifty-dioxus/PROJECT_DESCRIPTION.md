# Shifty Project Architecture

## Overview
Shifty is a web application built with Rust and Dioxus for managing employee shifts and schedules. It uses a client-side architecture with state management and REST API integration.

## Core Components

### Error Handling
- Custom `ShiftyError` enum using thiserror for ergonomic error definitions
- Centralized error handling with automatic HTTP 401 detection and page reload
- Two-tier error handling approach:
  - `error_handler`: Direct error processing with specific handling per error type
  - `result_handler`: Generic Result wrapper that processes errors and returns Option
- Special handling for authentication errors (HTTP 401) with automatic page reload
- Integration with reqwest errors for API communication
- Support for time-related errors from the time crate

### State Management
- Uses centralized stores for different domains (auth, config, employee, shiftplan, etc.)
- State is managed through thread-safe read/write access patterns
- Follows a reactive pattern where UI updates based on state changes

### Data Types
- `ImStr`: Custom string type wrapping `Rc<str>` for efficient string sharing
- Strong type system with domain-specific types like `Week`, `Slot`, `WorkingHoursCategory`
- Extensive use of Rust's type system for compile-time guarantees

### Internationalization (i18n)
- Flexible i18n system supporting multiple locales (currently En, De)
- Locale-aware date/time formatting
- Text templating with variable substitution
- Fallback mechanism for missing translations

### Services Layer
- Modular service architecture for business logic
- Key services include:
  - Authentication/Authorization
  - Employee management
  - Booking conflict detection
  - Slot editing
  - Weekly summaries

### API Integration
- REST API communication via reqwest
- Error handling with custom `ShiftyError` type
- Response parsing with serde
- Authentication state management

### UI Components
- Component-based architecture using Dioxus
- Modular page structure
- Reusable overlay and dropdown components
- Slot editing interface

### Time Management
- Sophisticated date/time handling
- Week-based scheduling system
- Working hours calculations
- Holiday and vacation tracking

## Design Patterns

### Type Safety
- Heavy use of Rust's type system
- Custom types for domain concepts
- Error handling through Result types

### Resource Management
- Reference counting (Rc) for shared ownership
- Efficient string handling with ImStr
- Thread-safe state access

### Service Pattern
- Separation of concerns between services
- Async/await for asynchronous operations
- Clear responsibility boundaries

### State Management
- Centralized stores for application state
- Reactive updates
- Thread-safe access patterns

## Project Structure

### Main Directories
- `src/component/`: UI components
- `src/page/`: Page definitions
- `src/service/`: Business logic services
- `src/state/`: State management
- `src/i18n/`: Internationalization
- `src/error.rs`: Error handling

### Key Features
- Employee shift management
- Schedule planning
- Conflict detection
- Multi-language support
- Authentication/Authorization
- Working hours calculation

## Dependencies
- dioxus: Web UI framework
- reqwest: HTTP client
- serde: Serialization
- time: Date/time handling
- uuid: Unique identifiers
- web-sys: Web API bindings

## Development
The project uses Rust 2021 edition and targets WebAssembly for browser deployment.
