use crate::api;
use crate::auth;
use crate::components::{ErrorMessage, Navbar};
use leptos::*;
use leptos_router::*;

#[component]
pub fn RegisterPage() -> impl IntoView {
    let (email, set_email) = create_signal(String::new());
    let (username, set_username) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (confirm_password, set_confirm_password) = create_signal(String::new());
    let (error, set_error) = create_signal(None::<String>);
    let (success, set_success) = create_signal(false);
    let (loading, set_loading) = create_signal(false);

    let on_submit = move |ev: ev::SubmitEvent| {
        ev.prevent_default();

        if password.get() != confirm_password.get() {
            set_error.set(Some("Passwords do not match".to_string()));
            return;
        }

        set_loading.set(true);
        set_error.set(None);

        let email_val = email.get();
        let username_val = username.get();
        let password_val = password.get();

        spawn_local(async move {
            match api::register(email_val, username_val, password_val).await {
                Ok(response) => {
                    // Save token and user info
                    auth::save_token(&response.token);
                    auth::save_user(&auth::AuthUser {
                        id: response.user.id,
                        email: response.user.email.clone(),
                        name: response.user.name.clone(),
                    });

                    set_success.set(true);
                    set_loading.set(false);

                    // Redirect to dashboard immediately
                    let window = web_sys::window().expect("no global window exists");
                    let _ = window.location().set_href("/dashboard");
                }
                Err(e) => {
                    // Display the error message from the API (already user-friendly)
                    set_error.set(Some(e));
                    set_loading.set(false);
                }
            }
        });
    };

    view! {
        <div>
            <Navbar/>
            <div class="container main-content">
                <div class="card" style="max-width: 500px; margin: 2rem auto;">
                    <h1 style="margin-bottom: 2rem; text-align: center;">"Register"</h1>

                    <Show when=move || error.get().is_some()>
                        <ErrorMessage message=error.get().unwrap_or_default()/>
                    </Show>

                    <Show when=move || success.get()>
                        <div class="success" style="padding: 1rem; background-color: #d1fae5; border-radius: 0.375rem;">
                            "Registration successful! Redirecting to login..."
                        </div>
                    </Show>

                    <form on:submit=on_submit style="margin-top: 1.5rem;">
                        <div style="margin-bottom: 1rem;">
                            <label class="label" for="email">"Email"</label>
                            <input
                                type="email"
                                id="email"
                                class="input"
                                placeholder="you@example.com"
                                prop:value=email
                                on:input=move |ev| set_email.set(event_target_value(&ev))
                                required
                            />
                        </div>

                        <div style="margin-bottom: 1rem;">
                            <label class="label" for="username">"Username"</label>
                            <input
                                type="text"
                                id="username"
                                class="input"
                                placeholder="johndoe"
                                prop:value=username
                                on:input=move |ev| set_username.set(event_target_value(&ev))
                                required
                            />
                        </div>

                        <div style="margin-bottom: 1rem;">
                            <label class="label" for="password">"Password"</label>
                            <input
                                type="password"
                                id="password"
                                class="input"
                                placeholder="••••••••"
                                prop:value=password
                                on:input=move |ev| set_password.set(event_target_value(&ev))
                                required
                            />
                        </div>

                        <div style="margin-bottom: 1.5rem;">
                            <label class="label" for="confirm_password">"Confirm Password"</label>
                            <input
                                type="password"
                                id="confirm_password"
                                class="input"
                                placeholder="••••••••"
                                prop:value=confirm_password
                                on:input=move |ev| set_confirm_password.set(event_target_value(&ev))
                                required
                            />
                        </div>

                        <button
                            type="submit"
                            class="btn btn-primary"
                            style="width: 100%;"
                            disabled=move || loading.get() || success.get()
                        >
                            {move || if loading.get() { "Registering..." } else { "Register" }}
                        </button>
                    </form>

                    <p style="text-align: center; margin-top: 1.5rem; color: var(--text-secondary);">
                        "Already have an account? "
                        <A href="/login" class="nav-link">"Login"</A>
                    </p>
                </div>
            </div>
        </div>
    }
}
