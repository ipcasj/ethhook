use crate::api;
use crate::auth;
use crate::components::{ErrorMessage, Navbar};
use leptos::*;
use leptos_router::*;

#[component]
pub fn LoginPage() -> impl IntoView {
    let (email, set_email) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (error, set_error) = create_signal(None::<String>);
    let (loading, set_loading) = create_signal(false);

    let on_submit = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        set_loading.set(true);
        set_error.set(None);

        let email_val = email.get();
        let password_val = password.get();

        spawn_local(async move {
            match api::login(email_val, password_val).await {
                Ok(response) => {
                    auth::save_token(&response.token);
                    auth::save_user(&auth::AuthUser {
                        id: response.user.id,
                        email: response.user.email,
                        name: response.user.name,
                    });
                    // Navigate to dashboard
                    let window = web_sys::window().expect("no global window exists");
                    let _ = window.location().set_href("/dashboard");
                }
                Err(e) => {
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
                    <h1 style="margin-bottom: 2rem; text-align: center;">"Login"</h1>

                    <Show when=move || error.get().is_some()>
                        <ErrorMessage message=error.get().unwrap_or_default()/>
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

                        <div style="margin-bottom: 1.5rem;">
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

                        <button
                            type="submit"
                            class="btn btn-primary"
                            style="width: 100%;"
                            disabled=move || loading.get()
                        >
                            {move || if loading.get() { "Logging in..." } else { "Login" }}
                        </button>
                    </form>

                    <p style="text-align: center; margin-top: 1.5rem; color: var(--text-secondary);">
                        "Don't have an account? "
                        <A href="/register" class="nav-link">"Register"</A>
                    </p>
                </div>
            </div>
        </div>
    }
}
