use crate::auth;
use crate::components::Navbar;
use leptos::*;

#[component]
pub fn SettingsPage() -> impl IntoView {
    let user = auth::get_user();

    view! {
        <div>
            <Navbar/>
            <div class="container main-content">
                <h1 style="margin-bottom: 2rem;">"Settings"</h1>

                <div class="card" style="margin-bottom: 1.5rem;">
                    <h2 style="margin-bottom: 1rem;">"Profile Information"</h2>
                    <div style="margin-bottom: 1rem;">
                        <label class="label">"Email"</label>
                        <input
                            type="email"
                            class="input"
                            value=user.clone().map(|u| u.email).unwrap_or_default()
                            readonly
                        />
                    </div>
                    <div style="margin-bottom: 1rem;">
                        <label class="label">"Name"</label>
                        <input
                            type="text"
                            class="input"
                            value=user.map(|u| u.name).unwrap_or_default()
                            readonly
                        />
                    </div>
                </div>

                <div class="card" style="margin-bottom: 1.5rem;">
                    <h2 style="margin-bottom: 1rem;">"API Keys"</h2>
                    <p style="color: var(--text-secondary); margin-bottom: 1rem;">
                        "Manage your API keys for programmatic access."
                    </p>
                    <button class="btn btn-primary">
                        "Generate New API Key"
                    </button>
                </div>

                <div class="card">
                    <h2 style="margin-bottom: 1rem;">"Danger Zone"</h2>
                    <p style="color: var(--text-secondary); margin-bottom: 1rem;">
                        "Permanently delete your account and all associated data."
                    </p>
                    <button class="btn btn-danger">
                        "Delete Account"
                    </button>
                </div>
            </div>
        </div>
    }
}
