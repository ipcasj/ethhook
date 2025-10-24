# Leptos Portal

The web-based admin portal for EthHook platform built with Leptos and WebAssembly.

## Prerequisites

Install Trunk (the build tool for Leptos):

```bash
cargo install trunk
```

Install the WebAssembly target:

```bash
rustup target add wasm32-unknown-unknown
```

## Development

Start the development server:

```bash
cd crates/leptos-portal
trunk serve
```

The portal will be available at `http://localhost:3000`

## Building for Production

Build an optimized release:

```bash
trunk build --release
```

The built files will be in `crates/leptos-portal/dist/`

## Project Structure

```text
leptos-portal/
├── src/
│   ├── api/           # API client functions
│   ├── auth.rs        # Authentication & token management
│   ├── components/    # Reusable UI components
│   ├── pages/         # Page components
│   │   ├── home.rs
│   │   ├── login.rs
│   │   ├── register.rs
│   │   ├── dashboard.rs
│   │   ├── applications.rs
│   │   ├── endpoints.rs
│   │   ├── events.rs
│   │   └── settings.rs
│   ├── utils.rs       # Utility functions
│   └── lib.rs         # App entry point
├── index.html         # HTML template
├── style.css          # Global styles
├── Cargo.toml
└── Trunk.toml         # Trunk configuration
```

## Features

### Implemented

- ✅ Authentication (Login/Register)
- ✅ JWT token management with local storage
- ✅ Protected routes
- ✅ Dashboard overview
- ✅ Navigation with Leptos Router
- ✅ Responsive design
- ✅ Loading states and error handling

### To Implement (Next Steps)

- [ ] Application CRUD operations
- [ ] Endpoint CRUD operations
- [ ] Event history viewer with pagination
- [ ] Real-time stats on dashboard
- [ ] API key management
- [ ] Form validation
- [ ] Confirmation modals
- [ ] Toast notifications

## API Integration

The frontend connects to the Admin API at `http://localhost:8080`. Make sure the Admin API is running and CORS is configured to allow requests from `http://localhost:3000`.

## Technologies

- **Leptos 0.6**: Reactive UI framework
- **Leptos Router**: Client-side routing
- **gloo-net**: HTTP requests
- **gloo-storage**: Local storage for auth tokens
- **WebAssembly**: Compiled to WASM for browser execution
