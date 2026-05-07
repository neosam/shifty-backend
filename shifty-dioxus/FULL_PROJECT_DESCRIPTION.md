# Shifty-Dioxus Project Description

## Overview
**Shifty-Dioxus** is a modern web application for employee shift management and scheduling, built with Rust and the Dioxus framework. It's designed as a sophisticated workforce management system that handles employee scheduling, time tracking, vacation management, and reporting.

## Tech Stack

### Frontend Framework
- **Dioxus 0.6.1**: Modern Rust-based web framework for building reactive user interfaces
- **WebAssembly (WASM)**: Compiles to WASM for browser execution
- **TailwindCSS**: Utility-first CSS framework for responsive and modern UI design

### Core Dependencies
- **Reqwest**: HTTP client for REST API communication
- **Serde**: Serialization/deserialization for JSON data handling
- **Time crate**: Advanced date and time manipulation
- **UUID**: Unique identifier generation
- **Web-sys & js-sys**: WebAssembly bindings for browser APIs
- **Thiserror**: Ergonomic error handling
- **Tracing**: Structured logging and diagnostics

### Development Tools
- **Cargo**: Rust package manager and build system
- **Dioxus CLI**: Development server with hot-reload
- **TailwindCSS CLI**: CSS compilation and optimization

## Architecture

### Project Structure
```
src/
├── component/           # Reusable UI components
├── page/               # Page-level components and routing
├── service/            # Business logic and API communication
├── state/              # Application state management
├── i18n/               # Internationalization support
├── api.rs              # REST API client functions
├── router.rs           # Application routing configuration
├── error.rs            # Centralized error handling
├── base_types.rs       # Custom data types (ImStr, etc.)
└── main.rs             # Application entry point

rest-types/             # Shared data types between frontend and backend
assets/                 # Static assets (CSS, images)
```

### Design Patterns

#### State Management
- **Centralized Stores**: Each domain has its own state store (auth, employee, shiftplan, etc.)
- **Reactive Updates**: UI automatically updates when state changes
- **Thread-safe Access**: Uses Rust's ownership system for safe concurrent access

#### Service Layer Architecture
- **Modular Services**: Separated business logic into focused services
- **Coroutine-based**: Async services using Dioxus coroutines
- **Error Handling**: Consistent error propagation with `ShiftyError` enum

#### Component-based UI
- **Reusable Components**: Modular components for common UI patterns
- **Props-based Communication**: Type-safe component communication
- **Conditional Rendering**: Dynamic UI based on user permissions and state

## Implemented Features

### Core Functionality
1. **Shift Planning & Scheduling**
   - Weekly shift view with drag-and-drop functionality
   - Slot-based scheduling system (time slots with minimum staffing requirements)
   - Conflict detection and resolution
   - Copy previous week functionality
   - Structure editing mode for shift planners

2. **Employee Management**
   - Employee profiles with work details
   - Working hours configuration per employee
   - Availability management (mark unavailable days)
   - Role-based access control

3. **Time Tracking & Reporting**
   - Working hours calculation and tracking
   - Balance tracking (expected vs. actual hours)
   - Weekly and monthly summaries
   - Vacation day tracking and entitlements
   - Sick leave and holiday management
   - Custom extra hours categories

4. **Authentication & Authorization**
   - User authentication system
   - Role-based permissions (HR, Shift Planner, Employee)
   - Session management with automatic logout on 401 errors

### Advanced Features

#### Internationalization (i18n)
- Multi-language support (English, German)
- Locale-aware date/time formatting
- Text templating with variable substitution
- Fallback mechanism for missing translations

#### Working Hours Management
- Flexible work schedule configuration
- Holiday and special day handling
- Vacation entitlement calculations
- Overtime and extra work tracking
- Custom extra hours categories for specific needs

#### Conflict Detection
- Automatic booking conflict identification
- Insufficient staffing warnings
- Visual indicators for scheduling issues

#### Data Export
- Personal calendar export functionality
- Insufficiently booked slots export
- Report generation for various time periods

### User Interface Features

#### Responsive Design
- Mobile-friendly responsive layout
- Print-optimized styles for reports
- Modern UI with TailwindCSS styling

#### Interactive Components
- Dropdown menus with search functionality
- Modal dialogs for forms and confirmations
- Overlay components for additional information
- Week navigation with calendar integration

#### Accessibility
- Keyboard navigation support
- Screen reader compatible
- Color-coded visual indicators
- Clear visual hierarchy

## API Integration

### REST API Communication
- Full REST API client implementation
- Error handling with automatic retry logic
- Session management and authentication tokens
- Proxy configuration for development environment

### Data Types
The `rest-types` crate defines comprehensive data transfer objects (DTOs) including:
- `SalesPersonTO`: Employee information and settings
- `BookingTO`: Shift assignments and scheduling
- `SlotTO`: Time slot definitions and requirements
- `WorkingHoursReportTO`: Comprehensive time tracking reports
- `ExtraHoursTO`: Additional work time entries
- And many more specialized types

## Development Features

### Hot Reload Development
- Dioxus development server with hot reload
- TailwindCSS watch mode for style changes
- Asset management with automatic rebuilding

### Error Handling
- Comprehensive error types using `thiserror`
- Automatic page reload on authentication errors
- User-friendly error messages and recovery

### State Persistence
- Local state management with persistence
- Configuration loading from backend
- Caching strategies for performance

## Security & Performance

### Security Features
- Authentication token management
- Role-based access control enforcement
- Secure HTTP communication
- Input validation and sanitization

### Performance Optimizations
- Efficient string handling with `ImStr` (Rc<str> wrapper)
- Lazy loading of resources
- Optimized rendering with minimal re-renders
- WebAssembly compilation for near-native performance

## Configuration

### Environment Support
- Development and production environment configurations
- Backend URL configuration
- Application title and branding customization
- Environment-specific feature toggles

### Deployment
- Static asset generation for web deployment
- WebAssembly optimization for production
- TailwindCSS purging for minimal CSS bundles

## Detailed Component Overview

### Pages (`src/page/`)
- **Home**: Landing page and navigation hub
- **ShiftPlan**: Main scheduling interface with weekly views
- **Employees**: Employee listing and management
- **EmployeeDetails**: Individual employee profile and settings
- **MyEmployeeDetails**: Personal employee profile for self-service
- **WeeklyOverview**: Summary view of weekly scheduling metrics
- **UserManagement**: Admin interface for user and role management
- **CustomExtraHoursManagement**: Configuration of custom work categories

### Components (`src/component/`)
- **WeekView**: Weekly calendar display with interactive scheduling
- **EmployeeView**: Employee information display and editing
- **TopBar**: Navigation header with user menu
- **SlotEdit**: Time slot creation and modification interface
- **WorkingHoursMiniOverview**: Compact working hours summary
- **DropdownBase**: Reusable dropdown menu component
- **Modal**: Modal dialog system for forms and confirmations

### Services (`src/service/`)
- **Auth**: Authentication and session management
- **Employee**: Employee data management and operations
- **BookingConflict**: Scheduling conflict detection and resolution
- **WeeklySummary**: Weekly metrics calculation and display
- **UserManagement**: User and permission management
- **I18n**: Internationalization service
- **Config**: Application configuration management

### State Management (`src/state/`)
- **Auth**: Authentication state and user information
- **Employee**: Employee data and selection state
- **ShiftPlan**: Scheduling data and weekly views
- **WeeklyOverview**: Summary metrics and calculations
- **Dropdown**: UI component state management

## Development Setup

### Prerequisites
1. **Rust**: Latest stable version with WebAssembly target
2. **Node.js & npm**: For TailwindCSS compilation
3. **Dioxus CLI**: For development server and hot reload

### Development Commands
```bash
# Install TailwindCSS
npm install -g tailwindcss

# Start TailwindCSS compiler (in separate terminal)
npx tailwindcss -i ./input.css -o ./assets/tailwind.css --watch

# Start Dioxus development server
dx serve --hot-reload
```

### Backend Integration
The application expects a backend API running on `localhost:3000` with the following endpoints:
- `/slot` - Time slot management
- `/booking` - Shift booking operations
- `/sales-person` - Employee management
- `/auth-info` - Authentication information
- `/working-hours` - Working hours tracking
- `/report` - Reporting functionality
- And many more as configured in `Dioxus.toml`

## License
This project is dual-licensed under:
- **MIT License**
- **Apache License 2.0**

This dual licensing approach follows the de-facto standard in the Rust ecosystem, allowing users to choose the license that best fits their needs.

---

This project represents a sophisticated, enterprise-grade shift management system built with modern Rust web technologies, offering a rich feature set for workforce scheduling and management while maintaining type safety, performance, and excellent user experience. 