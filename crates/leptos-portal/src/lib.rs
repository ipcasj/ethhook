use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use wasm_bindgen::prelude::*;

mod api;
mod auth;
mod components;
mod pages;
mod utils;

use components::{ToastContainer, ToastContext};
use pages::*;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    // Provide toast context globally
    let toast_ctx = ToastContext::new();
    provide_context(toast_ctx);

    view! {
        <Router>
            <Routes>
                <Route path="/" view=HomePage/>
                <Route path="/login" view=LoginPage/>
                <Route path="/register" view=RegisterPage/>
                <Route path="/dashboard" view=DashboardPage/>
                <Route path="/applications" view=ApplicationsPage/>
                <Route path="/applications/:id" view=ApplicationDetailPage/>
                <Route path="/applications/:id/endpoints" view=EndpointsPage/>
                <Route path="/endpoints" view=EndpointsPage/>
                <Route path="/events" view=EventsPage/>
                <Route path="/settings" view=SettingsPage/>
            </Routes>
        </Router>
        <ToastContainer/>
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).expect("error initializing logger");
    leptos::mount_to_body(App)
}
