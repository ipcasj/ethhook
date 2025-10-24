use crate::api::{self, Endpoint};
use crate::components::{Navbar, SkeletonEndpointCard};
use leptos::*;
use leptos_router::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{window, Window};

#[component]
pub fn EndpointsPage() -> impl IntoView {
    let params = use_params_map();
    let app_id = move || params.with(|p| p.get("id").cloned().unwrap_or_default());

    let (endpoints, set_endpoints) = create_signal(Vec::<Endpoint>::new());
    let (loading, set_loading) = create_signal(true);
    let (error, set_error) = create_signal(Option::<String>::None);
    let (show_create_modal, set_show_create_modal) = create_signal(false);
    let (show_edit_modal, set_show_edit_modal) = create_signal(false);
    let (show_delete_modal, set_show_delete_modal) = create_signal(false);
    let (endpoint_to_delete, set_endpoint_to_delete) = create_signal(Option::<Endpoint>::None);
    let (endpoint_to_edit, set_endpoint_to_edit) = create_signal(Option::<Endpoint>::None);
    let (app_name, set_app_name) = create_signal(String::from("Application"));
    let (search_query, set_search_query) = create_signal(String::new());

    // Load application info and endpoints
    let load_data = move || {
        let app_id_val = app_id();

        spawn_local(async move {
            set_loading.set(true);
            set_error.set(None);

            if app_id_val.is_empty() {
                // No app_id: Load ALL user endpoints across all applications
                set_app_name.set("All Endpoints".to_string());

                match api::list_all_user_endpoints().await {
                    Ok(response) => {
                        set_endpoints.set(response.endpoints);
                        set_loading.set(false);
                    }
                    Err(e) => {
                        set_error.set(Some(format!("Failed to load endpoints: {e}")));
                        set_loading.set(false);
                    }
                }
            } else {
                // Load application name
                match api::get_application(&app_id_val).await {
                    Ok(app) => set_app_name.set(app.name),
                    Err(e) => {
                        set_error.set(Some(format!("Failed to load application: {e}")));
                        set_loading.set(false);
                        return;
                    }
                }

                // Load endpoints for specific application
                match api::list_endpoints(&app_id_val).await {
                    Ok(response) => {
                        set_endpoints.set(response.endpoints);
                        set_loading.set(false);
                    }
                    Err(e) => {
                        set_error.set(Some(format!("Failed to load endpoints: {e}")));
                        set_loading.set(false);
                    }
                }
            }
        });
    };

    create_effect(move |_| {
        load_data();
    });

    let reload = move || load_data();

    let handle_edit = move |endpoint: Endpoint| {
        set_endpoint_to_edit.set(Some(endpoint));
        set_show_edit_modal.set(true);
    };

    let confirm_delete = move |endpoint: Endpoint| {
        set_endpoint_to_delete.set(Some(endpoint));
        set_show_delete_modal.set(true);
    };

    let do_delete = move || {
        if let Some(endpoint) = endpoint_to_delete.get() {
            let endpoint_id = endpoint.id.clone();
            spawn_local(async move {
                match api::delete_endpoint(&endpoint_id).await {
                    Ok(_) => {
                        set_show_delete_modal.set(false);
                        set_endpoint_to_delete.set(None);
                        reload();
                    }
                    Err(e) => {
                        set_error.set(Some(format!("Failed to delete endpoint: {e}")));
                    }
                }
            });
        }
    };

    let regenerate_secret = move |endpoint_id: String| {
        spawn_local(async move {
            match api::regenerate_hmac_secret(&endpoint_id).await {
                Ok(_) => reload(),
                Err(e) => set_error.set(Some(format!("Failed to regenerate secret: {e}"))),
            }
        });
    };

    let copy_to_clipboard = move |text: String| {
        if let Some(window) = window() {
            let clipboard = window.navigator().clipboard();
            let _ = clipboard.write_text(&text);
        }
    };

    // Filter endpoints based on search query
    let filtered_endpoints = move || {
        let query = search_query.get().to_lowercase();
        if query.is_empty() {
            return endpoints.get();
        }

        endpoints
            .get()
            .into_iter()
            .filter(|endpoint| {
                endpoint.name.to_lowercase().contains(&query)
                    || endpoint.webhook_url.to_lowercase().contains(&query)
                    || endpoint
                        .description
                        .as_ref()
                        .map(|d: &String| d.to_lowercase().contains(&query))
                        .unwrap_or(false)
                    || format!("{:?}", endpoint.chain_ids)
                        .to_lowercase()
                        .contains(&query)
                    || format!("{:?}", endpoint.contract_addresses)
                        .to_lowercase()
                        .contains(&query)
                    || (endpoint.is_active && query.contains("active"))
                    || (!endpoint.is_active && query.contains("inactive"))
            })
            .collect()
    };

    view! {
        <div>
            <Navbar/>
            <div class="container main-content">
                // Breadcrumb - only show when viewing a specific application
                <Show when=move || !app_id().is_empty()>
                    <div style="margin-bottom: 1rem;">
                        <a href="/applications" style="color: var(--primary); text-decoration: none;">"← Back to Applications"</a>
                    </div>
                </Show>

                // Header
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 2rem;">
                    <div>
                        <h1>{move || app_name.get()}</h1>
                        <p style="color: var(--text-secondary); margin-top: 0.5rem;">"Webhook Endpoints"</p>
                    </div>
                    // Always show "New Endpoint" button
                    <button
                        class="btn btn-primary"
                        on:click=move |_| set_show_create_modal.set(true)
                    >
                        "+ New Endpoint"
                    </button>
                </div>

                // Error display
                <Show when=move || error.get().is_some()>
                    <div class="card" style="background-color: var(--error-light); border-left: 4px solid var(--error); margin-bottom: 1rem;">
                        <p style="color: var(--error); margin: 0;">{move || error.get().unwrap_or_default()}</p>
                    </div>
                </Show>

                // Search bar
                <Show when=move || !loading.get() && !endpoints.get().is_empty()>
                    <div class="search-box" style="margin-bottom: 1.5rem;">
                        <input
                            type="text"
                            class="search-input"
                            placeholder="Search endpoints by URL, description, chain ID, or address..."
                            on:input=move |ev| {
                                set_search_query.set(event_target_value(&ev));
                            }
                            prop:value=move || search_query.get()
                        />
                        <Show when=move || !search_query.get().is_empty()>
                            <button
                                class="search-clear"
                                on:click=move |_| set_search_query.set(String::new())
                                title="Clear search"
                            >
                                "✕"
                            </button>
                        </Show>
                    </div>
                    <div class="search-results-count" style="margin-bottom: 1rem; color: var(--text-secondary); font-size: 0.875rem;">
                        {move || {
                            let filtered = filtered_endpoints();
                            let total = endpoints.get().len();
                            if search_query.get().is_empty() {
                                format!("Showing {total} endpoints")
                            } else {
                                format!("Showing {} of {} endpoints", filtered.len(), total)
                            }
                        }}
                    </div>
                </Show>

                // Loading
                <Show when=move || loading.get()>
                    <div style="display: grid; gap: 1.5rem;">
                        <SkeletonEndpointCard/>
                        <SkeletonEndpointCard/>
                        <SkeletonEndpointCard/>
                    </div>
                </Show>

                // Endpoints list
                <Show when=move || !loading.get()>
                    <Show
                        when=move || !endpoints.get().is_empty()
                        fallback=move || view! {
                            <div class="card">
                                <div style="text-align: center; padding: 3rem;">
                                    <p style="font-size: 1.25rem; color: var(--text-secondary); margin-bottom: 1rem;">
                                        "No endpoints yet"
                                    </p>
                                    <p style="color: var(--text-secondary); margin-bottom: 2rem;">
                                        "Create your first endpoint to start receiving webhook events"
                                    </p>
                                    <button
                                        class="btn btn-primary"
                                        on:click=move |_| set_show_create_modal.set(true)
                                    >
                                        "+ Create First Endpoint"
                                    </button>
                                </div>
                            </div>
                        }
                    >
                        <Show
                            when=move || !filtered_endpoints().is_empty()
                            fallback=move || view! {
                                <div class="card" style="text-align: center; padding: 3rem;">
                                    <p style="color: var(--text-secondary); margin-bottom: 1rem;">
                                        "No endpoints match your search."
                                    </p>
                                    <button
                                        class="btn btn-secondary"
                                        on:click=move |_| set_search_query.set(String::new())
                                    >
                                        "Clear search"
                                    </button>
                                </div>
                            }
                        >
                            <div style="display: grid; gap: 1rem;">
                                <For
                                    each=move || filtered_endpoints()
                                key=|endpoint| endpoint.id.clone()
                                children=move |endpoint: Endpoint| {
                                    let ep = endpoint.clone();
                                    let ep_edit = endpoint.clone();
                                    let on_edit = move || handle_edit(ep_edit.clone());
                                    let on_delete = move || confirm_delete(ep.clone());
                                    let on_regenerate = {
                                        let id = endpoint.id.clone();
                                        move || regenerate_secret(id.clone())
                                    };
                                    let on_copy = {
                                        let secret = endpoint.hmac_secret.clone();
                                        move || copy_to_clipboard(secret.clone())
                                    };

                                    view! {
                                        <EndpointCard
                                            endpoint=endpoint
                                            on_edit=on_edit
                                            on_delete=on_delete
                                            on_regenerate=on_regenerate
                                            on_copy=on_copy
                                        />
                                    }
                                }
                            />
                            </div>
                        </Show>
                    </Show>
                </Show>

                // Create modal
                <Show when=move || show_create_modal.get()>
                    <CreateEndpointModal
                        app_id=app_id()
                        on_close=move || set_show_create_modal.set(false)
                        on_created=move || {
                            set_show_create_modal.set(false);
                            reload();
                        }
                    />
                </Show>

                // Edit modal
                <Show when=move || show_edit_modal.get()>
                    <Show when=move || endpoint_to_edit.get().is_some()>
                        <EditEndpointModal
                            endpoint=endpoint_to_edit.get().unwrap()
                            on_close=move || {
                                set_show_edit_modal.set(false);
                                set_endpoint_to_edit.set(None);
                            }
                            on_updated=move || {
                                set_show_edit_modal.set(false);
                                set_endpoint_to_edit.set(None);
                                reload();
                            }
                        />
                    </Show>
                </Show>

                // Delete confirmation modal
                <Show when=move || show_delete_modal.get()>
                    <div class="modal-overlay" on:click=move |_| set_show_delete_modal.set(false)>
                        <div class="modal" on:click=move |e| e.stop_propagation()>
                            <h2>"Delete Endpoint"</h2>
                            <p style="margin: 1rem 0;">
                                "Are you sure you want to delete endpoint "
                                <strong>{move || endpoint_to_delete.get().map(|e| e.webhook_url).unwrap_or_default()}</strong>
                                "? This action cannot be undone."
                            </p>
                            <div style="display: flex; gap: 1rem; justify-content: flex-end; margin-top: 1.5rem;">
                                <button
                                    class="btn"
                                    on:click=move |_| set_show_delete_modal.set(false)
                                >
                                    "Cancel"
                                </button>
                                <button
                                    class="btn"
                                    style="background-color: var(--error); color: white;"
                                    on:click=move |_| do_delete()
                                >
                                    "Delete"
                                </button>
                            </div>
                        </div>
                    </div>
                </Show>
            </div>
        </div>
    }
}

#[component]
fn EndpointCard<F1, F2, F3, F4>(
    endpoint: Endpoint,
    on_edit: F1,
    on_delete: F2,
    on_regenerate: F3,
    on_copy: F4,
) -> impl IntoView
where
    F1: Fn() + 'static + Clone,
    F2: Fn() + 'static + Clone,
    F3: Fn() + 'static + Clone,
    F4: Fn() + 'static + Clone,
{
    let (show_secret, set_show_secret) = create_signal(false);
    let (copied, set_copied) = create_signal(false);

    let id = endpoint.id.clone();
    let url = endpoint.webhook_url.clone();
    let description = create_signal(endpoint.description.clone());
    let hmac_secret = endpoint.hmac_secret.clone();
    let is_active = endpoint.is_active;
    let chain_ids = endpoint.chain_ids.clone();
    let contract_addresses = endpoint.contract_addresses.clone();
    let event_signatures = endpoint.event_signatures.clone();

    let toggle_secret = move || set_show_secret.update(|v| *v = !*v);

    let on_edit_clone = on_edit.clone();
    let on_copy_clone1 = on_copy.clone();
    let on_copy_clone2 = on_copy.clone();
    let on_delete_clone = on_delete.clone();
    let on_regenerate_clone = on_regenerate.clone();

    let copy_secret1 = move || {
        on_copy_clone1.clone()();
        set_copied.set(true);
        set_timeout(
            move || set_copied.set(false),
            std::time::Duration::from_secs(2),
        );
    };

    let copy_secret2 = move || {
        on_copy_clone2.clone()();
        set_copied.set(true);
        set_timeout(
            move || set_copied.set(false),
            std::time::Duration::from_secs(2),
        );
    };

    view! {
        <div class="card">
            // Header
            <div style="display: flex; justify-content: space-between; align-items: start; margin-bottom: 1rem;">
                <div style="flex: 1;">
                    <div style="display: flex; align-items: center; gap: 0.5rem; margin-bottom: 0.5rem;">
                        <h3 style="margin: 0; font-size: 1.125rem;">{url.clone()}</h3>
                        <span
                            class="badge"
                            style=move || if is_active {
                                "background-color: var(--success-light); color: var(--success);"
                            } else {
                                "background-color: var(--text-secondary); color: white;"
                            }
                        >
                            {if is_active { "Active" } else { "Inactive" }}
                        </span>
                    </div>
                    <Show when=move || description.0.get().is_some()>
                        <p style="color: var(--text-secondary); margin: 0; font-size: 0.875rem;">
                            {description.0.get().unwrap_or_default()}
                        </p>
                    </Show>
                </div>
                <div style="display: flex; gap: 0.5rem;">
                    <button
                        class="btn"
                        style="padding: 0.5rem 1rem;"
                        on:click=move |_| {
                            let window: Window = window().unwrap();
                            let _ = window.location().set_href(&format!("/endpoints/{id}"));
                        }
                        title="View Details"
                    >
                        "View"
                    </button>
                    <button
                        class="btn btn-secondary"
                        style="padding: 0.5rem 1rem;"
                        on:click=move |_| on_edit_clone.clone()()
                        title="Edit Endpoint"
                    >
                        "✏️ Edit"
                    </button>
                    <button
                        class="btn"
                        style="background-color: var(--error); color: white; padding: 0.5rem 1rem;"
                        on:click=move |_| on_delete_clone.clone()()
                        title="Delete Endpoint"
                    >
                        "🗑️"
                    </button>
                </div>
            </div>

            // Filters
            <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 1rem; margin-bottom: 1rem; padding-top: 1rem; border-top: 1px solid var(--border);">
                <div>
                    <label class="label" style="font-size: 0.75rem;">"Chain IDs"</label>
                    <p style="margin: 0; font-size: 0.875rem;">
                        {if chain_ids.is_empty() { "All chains".to_string() } else { chain_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(", ") }}
                    </p>
                </div>
                <div>
                    <label class="label" style="font-size: 0.75rem;">"Contract Addresses"</label>
                    <p style="margin: 0; font-size: 0.875rem;">
                        {if contract_addresses.is_empty() { "All contracts".to_string() } else { format!("{} addresses", contract_addresses.len()) }}
                    </p>
                </div>
                <div>
                    <label class="label" style="font-size: 0.75rem;">"Event Signatures"</label>
                    <p style="margin: 0; font-size: 0.875rem;">
                        {if event_signatures.is_empty() { "All events".to_string() } else { format!("{} signatures", event_signatures.len()) }}
                    </p>
                </div>
            </div>

            // HMAC Secret
            <div style="border-top: 1px solid var(--border); padding-top: 1rem;">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.5rem;">
                    <label class="label">"HMAC Secret"</label>
                    <div style="display: flex; gap: 0.5rem;">
                        <button
                            class="btn"
                            style="font-size: 0.875rem; padding: 0.25rem 0.75rem;"
                            on:click=move |_| toggle_secret()
                        >
                            {move || if show_secret.get() { "🙈 Hide" } else { "👁️ Show" }}
                        </button>
                        <button
                            class="btn"
                            style="font-size: 0.875rem; padding: 0.25rem 0.75rem;"
                            on:click=move |_| copy_secret1()
                        >
                            {move || if copied.get() { "✅ Copied!" } else { "📋 Copy" }}
                        </button>
                        <button
                            class="btn"
                            style="font-size: 0.875rem; padding: 0.25rem 0.75rem; background-color: var(--warning); color: white;"
                            on:click=move |_| on_regenerate_clone.clone()()
                            title="Generate a new HMAC secret (old secret will stop working)"
                        >
                            "🔄 Regenerate"
                        </button>
                    </div>
                </div>
                <div
                    class="input"
                    style="font-family: monospace; font-size: 0.875rem; background-color: var(--bg-secondary); cursor: pointer;"
                    on:click=move |_| copy_secret2()
                >
                    {move || if show_secret.get() {
                        hmac_secret.clone()
                    } else {
                        "••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••".to_string()
                    }}
                </div>
            </div>
        </div>
    }
}

#[component]
fn EditEndpointModal<F1, F2>(endpoint: Endpoint, on_close: F1, on_updated: F2) -> impl IntoView
where
    F1: Fn() + 'static + Copy,
    F2: Fn() + 'static + Copy,
{
    let (webhook_url, set_webhook_url) = create_signal(endpoint.webhook_url.clone());
    let (description, set_description) =
        create_signal(endpoint.description.clone().unwrap_or_default());
    let (chain_ids_input, set_chain_ids_input) = create_signal(
        endpoint
            .chain_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(", "),
    );
    let (contract_addresses_input, set_contract_addresses_input) =
        create_signal(endpoint.contract_addresses.join(", "));
    let (event_signatures_input, set_event_signatures_input) =
        create_signal(endpoint.event_signatures.join(", "));
    let (is_active, set_is_active) = create_signal(endpoint.is_active);
    let (loading, set_loading) = create_signal(false);
    let (error, set_error) = create_signal(Option::<String>::None);

    let endpoint_id = endpoint.id.clone();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        let url = webhook_url.get();
        if url.trim().is_empty() {
            set_error.set(Some("Webhook URL is required".to_string()));
            return;
        }

        if !url.starts_with("http://") && !url.starts_with("https://") {
            set_error.set(Some(
                "Webhook URL must start with http:// or https://".to_string(),
            ));
            return;
        }

        let desc = description.get();
        let desc_opt = if desc.trim().is_empty() {
            None
        } else {
            Some(desc)
        };

        // Parse chain IDs (comma-separated numbers)
        let chain_ids: Vec<i32> = chain_ids_input
            .get()
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();

        if chain_ids.is_empty() {
            set_error.set(Some("At least one valid chain ID is required".to_string()));
            return;
        }

        // Parse contract addresses (comma-separated, optional)
        let contract_addresses: Vec<String> = contract_addresses_input
            .get()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        // Parse event signatures (comma-separated, optional)
        let event_signatures: Vec<String> = event_signatures_input
            .get()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        set_loading.set(true);
        set_error.set(None);

        let endpoint_id_clone = endpoint_id.clone();
        let active = is_active.get();

        spawn_local(async move {
            match api::update_endpoint(
                &endpoint_id_clone,
                Some(url),
                desc_opt,
                Some(chain_ids),
                Some(contract_addresses),
                Some(event_signatures),
                Some(active),
            )
            .await
            {
                Ok(_) => {
                    on_updated();
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_loading.set(false);
                }
            }
        });
    };

    view! {
        <div class="modal-overlay" on:click=move |_| on_close()>
            <div class="modal" on:click=move |e| e.stop_propagation()>
                <h2>"Edit Webhook Endpoint"</h2>

                <form on:submit=on_submit>
                    // Webhook URL
                    <div style="margin-bottom: 1rem;">
                        <label class="label">"Webhook URL" <span style="color: var(--error);">"*"</span></label>
                        <input
                            type="url"
                            class="input"
                            placeholder="https://example.com/webhook"
                            prop:value=move || webhook_url.get()
                            on:input=move |ev| set_webhook_url.set(event_target_value(&ev))
                            prop:disabled=move || loading.get()
                            required
                        />
                    </div>

                    // Description
                    <div style="margin-bottom: 1rem;">
                        <label class="label">"Description"</label>
                        <textarea
                            class="input"
                            placeholder="Optional description"
                            prop:value=move || description.get()
                            on:input=move |ev| set_description.set(event_target_value(&ev))
                            prop:disabled=move || loading.get()
                            maxlength="200"
                        />
                    </div>

                    // Active status
                    <div style="margin-bottom: 1rem;">
                        <label class="label" style="display: flex; align-items: center; gap: 0.5rem; cursor: pointer;">
                            <input
                                type="checkbox"
                                prop:checked=move || is_active.get()
                                on:change=move |ev| set_is_active.set(event_target_checked(&ev))
                                prop:disabled=move || loading.get()
                            />
                            "Active"
                        </label>
                        <p style="font-size: 0.75rem; color: var(--text-secondary); margin-top: 0.25rem;">
                            "Inactive endpoints will not receive webhook events"
                        </p>
                    </div>

                    // Chain IDs
                    <div style="margin-bottom: 1rem;">
                        <label class="label">"Chain IDs" <span style="color: var(--error);">"*"</span></label>
                        <input
                            type="text"
                            class="input"
                            placeholder="1, 137, 42161 (comma-separated)"
                            prop:value=move || chain_ids_input.get()
                            on:input=move |ev| set_chain_ids_input.set(event_target_value(&ev))
                            prop:disabled=move || loading.get()
                            required
                        />
                        <p style="font-size: 0.75rem; color: var(--text-secondary); margin-top: 0.25rem;">
                            "Ethereum=1, Polygon=137, Arbitrum=42161, etc."
                        </p>
                    </div>

                    // Contract Addresses (optional)
                    <div style="margin-bottom: 1rem;">
                        <label class="label">"Contract Addresses " <span style="color: var(--text-secondary);">"(optional)"</span></label>
                        <textarea
                            class="input"
                            placeholder="0x123..., 0x456... (comma-separated, leave empty for all)"
                            prop:value=move || contract_addresses_input.get()
                            on:input=move |ev| set_contract_addresses_input.set(event_target_value(&ev))
                            prop:disabled=move || loading.get()
                        />
                    </div>

                    // Event Signatures (optional)
                    <div style="margin-bottom: 1rem;">
                        <label class="label">"Event Signatures " <span style="color: var(--text-secondary);">"(optional)"</span></label>
                        <textarea
                            class="input"
                            placeholder="Transfer(address,address,uint256) (comma-separated, leave empty for all)"
                            prop:value=move || event_signatures_input.get()
                            on:input=move |ev| set_event_signatures_input.set(event_target_value(&ev))
                            prop:disabled=move || loading.get()
                        />
                    </div>

                    // Error display
                    <Show when=move || error.get().is_some()>
                        <div style="background-color: var(--error-light); border-left: 4px solid var(--error); padding: 0.75rem; margin-bottom: 1rem;">
                            <p style="color: var(--error); margin: 0; font-size: 0.875rem;">
                                {move || error.get().unwrap_or_default()}
                            </p>
                        </div>
                    </Show>

                    // Actions
                    <div style="display: flex; gap: 1rem; justify-content: flex-end;">
                        <button
                            type="button"
                            class="btn"
                            on:click=move |_| on_close()
                            prop:disabled=move || loading.get()
                        >
                            "Cancel"
                        </button>
                        <button
                            type="submit"
                            class="btn btn-primary"
                            prop:disabled=move || loading.get()
                        >
                            {move || if loading.get() { "Saving..." } else { "Save Changes" }}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    }
}

#[component]
fn CreateEndpointModal<F1, F2>(app_id: String, on_close: F1, on_created: F2) -> impl IntoView
where
    F1: Fn() + 'static + Copy,
    F2: Fn() + 'static + Copy,
{
    let (webhook_url, set_webhook_url) = create_signal(String::new());
    let (description, set_description) = create_signal(String::new());
    let (chain_ids_input, set_chain_ids_input) = create_signal(String::from("1"));
    let (contract_addresses_input, set_contract_addresses_input) = create_signal(String::new());
    let (event_signatures_input, set_event_signatures_input) = create_signal(String::new());
    let (loading, set_loading) = create_signal(false);
    let (error, set_error) = create_signal(Option::<String>::None);

    // Application selector state (when app_id is empty)
    let (selected_app_id, set_selected_app_id) = create_signal(app_id.clone());
    let (applications, set_applications) = create_signal(Vec::<(String, String)>::new()); // (id, name)
    let (loading_apps, set_loading_apps) = create_signal(false);

    // Load applications if app_id is empty
    let needs_app_selector = app_id.trim().is_empty();
    if needs_app_selector {
        set_loading_apps.set(true);
        spawn_local(async move {
            match api::list_applications().await {
                Ok(response) => {
                    let apps: Vec<(String, String)> = response
                        .applications
                        .into_iter()
                        .map(|app| (app.id.to_string(), app.name))
                        .collect();
                    if let Some(first_app) = apps.first() {
                        set_selected_app_id.set(first_app.0.clone());
                    }
                    set_applications.set(apps);
                    set_loading_apps.set(false);
                }
                Err(e) => {
                    set_error.set(Some(format!("Failed to load applications: {e}")));
                    set_loading_apps.set(false);
                }
            }
        });
    }

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        let final_app_id = selected_app_id.get();
        if final_app_id.trim().is_empty() {
            set_error.set(Some("Please select an application".to_string()));
            return;
        }

        let url = webhook_url.get();
        if url.trim().is_empty() {
            set_error.set(Some("Webhook URL is required".to_string()));
            return;
        }

        if !url.starts_with("http://") && !url.starts_with("https://") {
            set_error.set(Some(
                "Webhook URL must start with http:// or https://".to_string(),
            ));
            return;
        }

        let desc = description.get();
        let desc = if desc.trim().is_empty() {
            None
        } else {
            Some(desc)
        };

        // Parse chain IDs (comma-separated numbers)
        let chain_ids: Vec<i32> = chain_ids_input
            .get()
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();

        if chain_ids.is_empty() {
            set_error.set(Some("At least one valid chain ID is required".to_string()));
            return;
        }

        // Parse contract addresses (comma-separated, optional)
        let contract_addresses: Vec<String> = contract_addresses_input
            .get()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        // Parse event signatures (comma-separated, optional)
        let event_signatures: Vec<String> = event_signatures_input
            .get()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        set_loading.set(true);
        set_error.set(None);

        let final_app_id_clone = final_app_id.clone();
        let on_created_clone = on_created;

        spawn_local(async move {
            match api::create_endpoint(
                final_app_id_clone,
                url,
                desc,
                chain_ids,
                contract_addresses,
                event_signatures,
            )
            .await
            {
                Ok(_) => {
                    on_created_clone();
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_loading.set(false);
                }
            }
        });
    };

    view! {
        <div class="modal-overlay" on:click=move |_| on_close()>
            <div class="modal" on:click=move |e| e.stop_propagation()>
                <h2>"Create Webhook Endpoint"</h2>

                <form on:submit=on_submit>
                    // Application selector (only shown when no app_id in context)
                    {move || if needs_app_selector {
                        view! {
                            <div style="margin-bottom: 1rem;">
                                <label class="label">"Application" <span style="color: var(--error);">"*"</span></label>
                                <Show when=move || loading_apps.get()>
                                    <div style="padding: 0.75rem; text-align: center; color: var(--text-secondary);">
                                        "Loading applications..."
                                    </div>
                                </Show>
                                <Show when=move || !loading_apps.get()>
                                    {move || if applications.get().is_empty() {
                                        view! {
                                            <div style="padding: 0.75rem; background-color: var(--warning-light); border-radius: 8px; color: var(--warning);">
                                                "No applications found. Please create an application first."
                                            </div>
                                        }.into_view()
                                    } else {
                                        view! {
                                            <select
                                                class="input"
                                                prop:value=move || selected_app_id.get()
                                                on:change=move |ev| set_selected_app_id.set(event_target_value(&ev))
                                                prop:disabled=move || loading.get()
                                            >
                                                <For
                                                    each=move || applications.get()
                                                    key=|(id, _)| id.clone()
                                                    children=|(id, name)| {
                                                        view! {
                                                            <option value=id.clone()>{name}</option>
                                                        }
                                                    }
                                                />
                                            </select>
                                        }.into_view()
                                    }}
                                </Show>
                            </div>
                        }.into_view()
                    } else {
                        view! { <div></div> }.into_view()
                    }}

                    // Webhook URL
                    <div style="margin-bottom: 1rem;">
                        <label class="label">"Webhook URL" <span style="color: var(--error);">"*"</span></label>
                        <input
                            type="url"
                            class="input"
                            placeholder="https://example.com/webhook"
                            prop:value=move || webhook_url.get()
                            on:input=move |ev| set_webhook_url.set(event_target_value(&ev))
                            prop:disabled=move || loading.get()
                            required
                        />
                    </div>

                    // Description
                    <div style="margin-bottom: 1rem;">
                        <label class="label">"Description"</label>
                        <textarea
                            class="input"
                            placeholder="Optional description"
                            prop:value=move || description.get()
                            on:input=move |ev| set_description.set(event_target_value(&ev))
                            prop:disabled=move || loading.get()
                            maxlength="200"
                        />
                    </div>

                    // Chain IDs
                    <div style="margin-bottom: 1rem;">
                        <label class="label">"Chain IDs" <span style="color: var(--error);">"*"</span></label>
                        <input
                            type="text"
                            class="input"
                            placeholder="1, 137, 42161 (comma-separated)"
                            prop:value=move || chain_ids_input.get()
                            on:input=move |ev| set_chain_ids_input.set(event_target_value(&ev))
                            prop:disabled=move || loading.get()
                            required
                        />
                        <p style="font-size: 0.75rem; color: var(--text-secondary); margin-top: 0.25rem;">
                            "Ethereum=1, Polygon=137, Arbitrum=42161, etc."
                        </p>
                    </div>

                    // Contract Addresses (optional)
                    <div style="margin-bottom: 1rem;">
                        <label class="label">"Contract Addresses " <span style="color: var(--text-secondary);">"(optional)"</span></label>
                        <textarea
                            class="input"
                            placeholder="0x123..., 0x456... (comma-separated, leave empty for all)"
                            prop:value=move || contract_addresses_input.get()
                            on:input=move |ev| set_contract_addresses_input.set(event_target_value(&ev))
                            prop:disabled=move || loading.get()
                        />
                    </div>

                    // Event Signatures (optional)
                    <div style="margin-bottom: 1rem;">
                        <label class="label">"Event Signatures " <span style="color: var(--text-secondary);">"(optional)"</span></label>
                        <textarea
                            class="input"
                            placeholder="Transfer(address,address,uint256) (comma-separated, leave empty for all)"
                            prop:value=move || event_signatures_input.get()
                            on:input=move |ev| set_event_signatures_input.set(event_target_value(&ev))
                            prop:disabled=move || loading.get()
                        />
                    </div>

                    // Error display
                    <Show when=move || error.get().is_some()>
                        <div style="background-color: var(--error-light); border-left: 4px solid var(--error); padding: 0.75rem; margin-bottom: 1rem;">
                            <p style="color: var(--error); margin: 0; font-size: 0.875rem;">
                                {move || error.get().unwrap_or_default()}
                            </p>
                        </div>
                    </Show>

                    // Actions
                    <div style="display: flex; gap: 1rem; justify-content: flex-end;">
                        <button
                            type="button"
                            class="btn"
                            on:click=move |_| on_close()
                            prop:disabled=move || loading.get()
                        >
                            "Cancel"
                        </button>
                        <button
                            type="submit"
                            class="btn btn-primary"
                            prop:disabled=move || loading.get()
                        >
                            {move || if loading.get() { "Creating..." } else { "Create Endpoint" }}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    }
}
