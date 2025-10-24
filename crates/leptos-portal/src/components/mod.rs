use crate::auth;
use leptos::*;
use leptos_router::*;

mod event_recommendations;
mod skeleton;
mod toast;

pub use skeleton::{SkeletonApplicationCard, SkeletonEndpointCard, SkeletonStatCard};
pub use toast::{ToastContainer, ToastContext};

#[component]
pub fn Navbar() -> impl IntoView {
    let is_auth = create_memo(move |_| auth::is_authenticated());
    let user = create_memo(move |_| auth::get_user());

    view! {
        <nav class="nav">
            <div class="container nav-content">
                <A href="/" class="nav-brand">"EthHook Portal"</A>
                <ul class="nav-links">
                    <Show
                        when=move || is_auth.get()
                        fallback=|| view! {
                            <li><A href="/login" class="nav-link">"Login"</A></li>
                            <li><A href="/register" class="nav-link">"Register"</A></li>
                        }
                    >
                        <li><A href="/dashboard" class="nav-link">"Dashboard"</A></li>
                        <li><A href="/applications" class="nav-link">"Applications"</A></li>
                        <li><A href="/endpoints" class="nav-link">"Endpoints"</A></li>
                        <li><A href="/events" class="nav-link">"Events"</A></li>
                        <li><A href="/settings" class="nav-link">"Settings"</A></li>
                        <li>
                            <span class="nav-link" style="cursor: pointer;" on:click=move |_| {
                                auth::logout();
                                let window = web_sys::window().expect("no global window exists");
                                let _ = window.location().set_href("/login");
                            }>
                                {move || user.get().map(|u| u.name).unwrap_or_default()}
                                " | Logout"
                            </span>
                        </li>
                    </Show>
                </ul>
            </div>
        </nav>
    }
}

#[component]
pub fn LoadingSpinner() -> impl IntoView {
    view! {
        <div style="display: flex; justify-content: center; align-items: center; padding: 2rem;">
            <div class="spinner"></div>
        </div>
    }
}

#[component]
pub fn ErrorMessage(message: String) -> impl IntoView {
    view! {
        <div class="error" style="padding: 1rem; background-color: #fee2e2; border-radius: 0.375rem;">
            {message}
        </div>
    }
}
