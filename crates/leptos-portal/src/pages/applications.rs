use crate::api::{self, Application};
use crate::components::{Navbar, SkeletonApplicationCard, ToastContext};
use crate::utils::{is_valid_length, length_error_message};
use leptos::ev::SubmitEvent;
use leptos::*;

#[component]
pub fn ApplicationsPage() -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastContext must be provided");

    let (applications, set_applications) = create_signal(Vec::<Application>::new());
    let (loading, set_loading) = create_signal(true);
    let (error, set_error) = create_signal(Option::<String>::None);
    let (show_create_modal, set_show_create_modal) = create_signal(false);
    let (show_edit_modal, set_show_edit_modal) = create_signal(false);
    let (show_delete_modal, set_show_delete_modal) = create_signal(false);
    let (app_to_delete, set_app_to_delete) = create_signal(Option::<Application>::None);
    let (app_to_edit, set_app_to_edit) = create_signal(Option::<Application>::None);
    let (search_query, set_search_query) = create_signal(String::new());

    // Load applications on mount
    create_effect(move |_| {
        spawn_local(async move {
            match api::list_applications().await {
                Ok(response) => {
                    set_applications.set(response.applications);
                    set_loading.set(false);
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_loading.set(false);
                }
            }
        });
    });

    let reload_applications = move || {
        set_loading.set(true);
        set_error.set(None);
        spawn_local(async move {
            match api::list_applications().await {
                Ok(response) => {
                    set_applications.set(response.applications);
                    set_loading.set(false);
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_loading.set(false);
                }
            }
        });
    };

    let handle_edit = move |app: Application| {
        set_app_to_edit.set(Some(app));
        set_show_edit_modal.set(true);
    };

    let handle_delete = move |app: Application| {
        set_app_to_delete.set(Some(app));
        set_show_delete_modal.set(true);
    };

    let confirm_delete = move || {
        if let Some(app) = app_to_delete.get() {
            let app_name = app.name.clone();
            let app_id = app.id.clone();
            spawn_local(async move {
                match api::delete_application(&app_id).await {
                    Ok(_) => {
                        set_show_delete_modal.set(false);
                        toast.success(format!("Application '{}' deleted successfully", app_name));
                        reload_applications();
                    }
                    Err(e) => {
                        toast.error(format!("Failed to delete application: {}", e));
                    }
                }
            });
        }
    };

    let copy_to_clipboard = move |text: String| {
        let window = web_sys::window().expect("no global window exists");
        let clipboard = window.navigator().clipboard();
        let _ = clipboard.write_text(&text);
        toast.success("Copied to clipboard");
    };

    // Filter applications based on search query
    let filtered_applications = move || {
        let query = search_query.get().to_lowercase();
        if query.is_empty() {
            return applications.get();
        }

        applications
            .get()
            .into_iter()
            .filter(|app| {
                app.name.to_lowercase().contains(&query)
                    || app
                        .description
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(&query))
                        .unwrap_or(false)
                    || (app.is_active && query.contains("active"))
                    || (!app.is_active && query.contains("inactive"))
            })
            .collect()
    };

    view! {
        <div>
            <Navbar/>
            <div class="container main-content">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 2rem;">
                    <h1>"Applications"</h1>
                    <button
                        class="btn btn-primary"
                        on:click=move |_| set_show_create_modal.set(true)
                    >
                        "+ Create Application"
                    </button>
                </div>

                <Show when=move || error.get().is_some()>
                    <div class="card" style="background-color: #fef2f2; border-left: 4px solid var(--danger); margin-bottom: 1rem;">
                        <p style="color: var(--danger); margin: 0;">{move || error.get().unwrap_or_default()}</p>
                    </div>
                </Show>

                // Search bar
                <Show when=move || !loading.get() && !applications.get().is_empty()>
                    <div class="search-box" style="margin-bottom: 1.5rem;">
                        <input
                            type="text"
                            class="search-input"
                            placeholder="Search applications by name, description, or status..."
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
                            let filtered = filtered_applications();
                            let total = applications.get().len();
                            if search_query.get().is_empty() {
                                format!("Showing {} applications", total)
                            } else {
                                format!("Showing {} of {} applications", filtered.len(), total)
                            }
                        }}
                    </div>
                </Show>

                <Show
                    when=move || loading.get()
                    fallback=move || view! {
                        <Show
                            when=move || !applications.get().is_empty()
                            fallback=move || view! {
                                <div class="card" style="text-align: center; padding: 3rem;">
                                    <p style="color: var(--text-secondary); margin-bottom: 1.5rem;">
                                        "No applications yet. Create your first application to get started!"
                                    </p>
                                    <button
                                        class="btn btn-primary"
                                        on:click=move |_| set_show_create_modal.set(true)
                                    >
                                        "+ Create Your First Application"
                                    </button>
                                </div>
                            }
                        >
                            <Show
                                when=move || !filtered_applications().is_empty()
                                fallback=move || view! {
                                    <div class="card" style="text-align: center; padding: 3rem;">
                                        <p style="color: var(--text-secondary); margin-bottom: 1rem;">
                                            "No applications match your search."
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
                                <div style="display: grid; gap: 1.5rem;">
                                    <For
                                        each=move || filtered_applications()
                                    key=|app| app.id.clone()
                                    children=move |app: Application| {
                                        let app_clone = app.clone();
                                        let app_clone2 = app.clone();
                                        let app_clone3 = app.clone();
                                        let api_key = app.api_key.clone();

                                        view! {
                                            <ApplicationCard
                                                app=app
                                                on_edit=move || handle_edit(app_clone3.clone())
                                                on_delete=move || handle_delete(app_clone.clone())
                                                on_regenerate=move || {
                                                    let app_id = app_clone2.id.clone();
                                                    spawn_local(async move {
                                                        match api::regenerate_api_key(&app_id).await {
                                                            Ok(_) => {
                                                                toast.success("API key regenerated successfully");
                                                                reload_applications();
                                                            }
                                                            Err(e) => toast.error(format!("Failed to regenerate key: {}", e)),
                                                        }
                                                    });
                                                }
                                                on_copy=move || copy_to_clipboard(api_key.clone())
                                            />
                                        }
                                    }
                                />
                            </div>
                            </Show>
                        </Show>
                    }
                >
                    <div style="display: grid; gap: 1.5rem;">
                        <SkeletonApplicationCard/>
                        <SkeletonApplicationCard/>
                        <SkeletonApplicationCard/>
                    </div>
                </Show>

                // Create Modal
                <Show when=move || show_create_modal.get()>
                    <CreateApplicationModal
                        on_close=move || set_show_create_modal.set(false)
                        on_created=move || {
                            toast.success("Application created successfully");
                            set_show_create_modal.set(false);
                            reload_applications();
                        }
                    />
                </Show>

                // Edit Modal
                <Show when=move || show_edit_modal.get()>
                    <Show when=move || app_to_edit.get().is_some()>
                        <EditApplicationModal
                            app=app_to_edit.get().unwrap()
                            on_close=move || set_show_edit_modal.set(false)
                            on_updated=move || {
                                toast.success("Application updated successfully");
                                set_show_edit_modal.set(false);
                                reload_applications();
                            }
                        />
                    </Show>
                </Show>

                // Delete Confirmation Modal
                <Show when=move || show_delete_modal.get()>
                    <div class="modal-overlay" on:click=move |_| set_show_delete_modal.set(false)>
                        <div class="modal" on:click=move |e| e.stop_propagation()>
                            <h2>"Confirm Deletion"</h2>
                            <p style="margin: 1.5rem 0;">
                                "Are you sure you want to delete "
                                <strong>{move || app_to_delete.get().map(|a| a.name).unwrap_or_default()}</strong>
                                "? This action cannot be undone."
                            </p>
                            <div style="display: flex; gap: 1rem; justify-content: flex-end;">
                                <button
                                    class="btn"
                                    on:click=move |_| set_show_delete_modal.set(false)
                                >
                                    "Cancel"
                                </button>
                                <button
                                    class="btn"
                                    style="background-color: var(--error); color: white;"
                                    on:click=move |_| confirm_delete()
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
fn ApplicationCard<F1, F2, F3, F4>(
    app: Application,
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
    let (show_key, set_show_key) = create_signal(false);
    let (copied, set_copied) = create_signal(false);

    // Clone fields upfront to avoid move issues
    let name = app.name.clone();
    let description = create_signal(app.description.clone());
    let is_active = app.is_active;
    let created_at = app.created_at.clone();
    let app_id = app.id.clone();
    let api_key = app.api_key.clone();

    let app_id_for_endpoints = app_id.clone();
    let app_id_for_details = app_id.clone();

    let on_edit_clone = on_edit.clone();
    let on_copy_clone1 = on_copy.clone();
    let on_copy_clone2 = on_copy.clone();
    let on_delete_clone = on_delete.clone();
    let on_regenerate_clone = on_regenerate.clone();

    let toggle_key = move || {
        set_show_key.update(|v| *v = !*v);
    };

    let copy_key1 = move || {
        on_copy_clone1.clone()();
        set_copied.set(true);
        set_timeout(
            move || set_copied.set(false),
            std::time::Duration::from_secs(2),
        );
    };

    let copy_key2 = move || {
        on_copy_clone2.clone()();
        set_copied.set(true);
        set_timeout(
            move || set_copied.set(false),
            std::time::Duration::from_secs(2),
        );
    };

    view! {
        <div class="card">
            <div style="display: flex; justify-content: space-between; align-items: start; margin-bottom: 1rem;">
                <div>
                    <h2 style="margin-bottom: 0.5rem;">{name.clone()}</h2>
                    <Show when=move || description.0.get().is_some()>
                        <p style="color: var(--text-secondary); margin-bottom: 0.5rem;">
                            {description.0.get().unwrap_or_default()}
                        </p>
                    </Show>
                    <div style="display: flex; gap: 1rem; margin-top: 0.5rem;">
                        <span class="badge" style:background-color=move || if is_active { "var(--success)" } else { "var(--text-secondary)" }>
                            {move || if is_active { "Active" } else { "Inactive" }}
                        </span>
                        <span style="color: var(--text-secondary); font-size: 0.875rem;">
                            "Created: " {created_at.clone()}
                        </span>
                    </div>
                </div>
                <div style="display: flex; gap: 0.5rem;">
                    <button
                        class="btn"
                        on:click=move |_| {
                            let window = web_sys::window().expect("no global window exists");
                            let _ = window.location().set_href(&format!("/applications/{}/endpoints", app_id_for_endpoints));
                        }
                        title="Manage Endpoints"
                    >
                        "🔗 Endpoints"
                    </button>
                    <button
                        class="btn"
                        on:click=move |_| {
                            let window = web_sys::window().expect("no global window exists");
                            let _ = window.location().set_href(&format!("/applications/{}", app_id_for_details));
                        }
                        title="View Details"
                    >
                        "📄 View"
                    </button>
                    <button
                        class="btn"
                        on:click=move |_| on_edit_clone.clone()()
                        title="Edit Application"
                    >
                        "✏️ Edit"
                    </button>
                    <button
                        class="btn"
                        style="background-color: var(--error); color: white;"
                        on:click=move |_| on_delete_clone.clone()()
                        title="Delete Application"
                    >
                        "🗑️"
                    </button>
                </div>
            </div>

            <div style="border-top: 1px solid var(--border); padding-top: 1rem;">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.5rem;">
                    <label class="label">"API Key"</label>
                    <div style="display: flex; gap: 0.5rem;">
                        <button
                            class="btn"
                            style="font-size: 0.875rem; padding: 0.25rem 0.75rem;"
                            on:click=move |_| toggle_key()
                        >
                            {move || if show_key.get() { "🙈 Hide" } else { "👁️ Show" }}
                        </button>
                        <button
                            class="btn"
                            style="font-size: 0.875rem; padding: 0.25rem 0.75rem;"
                            on:click=move |_| copy_key1()
                        >
                            {move || if copied.get() { "✅ Copied!" } else { "📋 Copy" }}
                        </button>
                        <button
                            class="btn"
                            style="font-size: 0.875rem; padding: 0.25rem 0.75rem; background-color: var(--warning); color: white;"
                            on:click=move |_| on_regenerate_clone.clone()()
                            title="Generate a new API key (old key will stop working)"
                        >
                            "🔄 Regenerate"
                        </button>
                    </div>
                </div>
                <div
                    class="input"
                    style="font-family: monospace; font-size: 0.875rem; background-color: var(--bg-secondary); cursor: pointer;"
                    on:click=move |_| copy_key2()
                >
                    {move || if show_key.get() {
                        api_key.clone()
                    } else {
                        "••••••••••••••••••••••••••••••••".to_string()
                    }}
                </div>
            </div>
        </div>
    }
}

#[component]
fn CreateApplicationModal<F1, F2>(on_close: F1, on_created: F2) -> impl IntoView
where
    F1: Fn() + 'static + Copy,
    F2: Fn() + 'static + Copy,
{
    let (name, set_name) = create_signal(String::new());
    let (description, set_description) = create_signal(String::new());
    let (loading, set_loading) = create_signal(false);
    let (error, set_error) = create_signal(Option::<String>::None);

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        set_loading.set(true);
        set_error.set(None);

        let name_val = name.get();
        let desc_val = description.get();
        let desc_opt = if desc_val.is_empty() {
            None
        } else {
            Some(desc_val)
        };

        let on_created_clone = on_created;
        spawn_local(async move {
            match api::create_application(name_val, desc_opt).await {
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
                <h2 style="margin-bottom: 1.5rem;">"Create New Application"</h2>

                <Show when=move || error.get().is_some()>
                    <div style="background-color: #fef2f2; border-left: 4px solid var(--danger); padding: 0.75rem; margin-bottom: 1rem;">
                        <p style="color: var(--danger); margin: 0; font-size: 0.875rem;">
                            {move || error.get().unwrap_or_default()}
                        </p>
                    </div>
                </Show>

                <form on:submit=on_submit>
                    <div style="margin-bottom: 1rem;">
                        <label class="label" for="app-name">
                            "Application Name"
                            <span style="color: var(--error);">"*"</span>
                            <span style="color: var(--text-secondary); font-weight: normal; font-size: 0.875rem;">
                                {move || format!(" ({}/50)", name.get().len())}
                            </span>
                        </label>
                        <input
                            type="text"
                            id="app-name"
                            class="input"
                            placeholder="My Awesome App"
                            prop:value=name
                            on:input=move |ev| set_name.set(event_target_value(&ev))
                            required
                            maxlength="50"
                            style=move || {
                                let valid = is_valid_length(&name.get(), 1, 50);
                                if !name.get().is_empty() && !valid {
                                    "border-color: var(--danger);"
                                } else {
                                    ""
                                }
                            }
                        />
                        <Show when=move || {
                            let n = name.get();
                            !n.is_empty() && length_error_message("Name", &n, 1, 50).is_some()
                        }>
                            <p style="color: var(--danger); font-size: 0.875rem; margin-top: 0.25rem; margin-bottom: 0;">
                                {move || length_error_message("Name", &name.get(), 1, 50).unwrap_or_default()}
                            </p>
                        </Show>
                    </div>

                    <div style="margin-bottom: 1.5rem;">
                        <label class="label" for="app-description">
                            "Description"
                            <span style="color: var(--text-secondary); font-weight: normal;">"(optional)"</span>
                            <span style="color: var(--text-secondary); font-weight: normal; font-size: 0.875rem;">
                                {move || format!(" ({}/500)", description.get().len())}
                            </span>
                        </label>
                        <textarea
                            id="app-description"
                            class="input"
                            placeholder="A brief description of your application..."
                            rows="3"
                            prop:value=description
                            on:input=move |ev| set_description.set(event_target_value(&ev))
                            maxlength="500"
                            style=move || {
                                let desc = description.get();
                                let valid = is_valid_length(&desc, 0, 500);
                                if !desc.is_empty() && !valid {
                                    "border-color: var(--danger);"
                                } else {
                                    ""
                                }
                            }
                        />
                        <Show when=move || {
                            let d = description.get();
                            !d.is_empty() && length_error_message("Description", &d, 0, 500).is_some()
                        }>
                            <p style="color: var(--danger); font-size: 0.875rem; margin-top: 0.25rem; margin-bottom: 0;">
                                {move || length_error_message("Description", &description.get(), 0, 500).unwrap_or_default()}
                            </p>
                        </Show>
                    </div>

                    <div style="display: flex; gap: 1rem; justify-content: flex-end;">
                        <button
                            type="button"
                            class="btn"
                            on:click=move |_| on_close()
                            disabled=loading
                        >
                            "Cancel"
                        </button>
                        <button
                            type="submit"
                            class="btn btn-primary"
                            disabled=move || {
                                loading.get() ||
                                name.get().is_empty() ||
                                !is_valid_length(&name.get(), 1, 50) ||
                                !is_valid_length(&description.get(), 0, 500)
                            }
                        >
                            {move || if loading.get() { "Creating..." } else { "Create Application" }}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    }
}

#[component]
fn EditApplicationModal<F1, F2>(app: Application, on_close: F1, on_updated: F2) -> impl IntoView
where
    F1: Fn() + 'static + Copy,
    F2: Fn() + 'static + Copy,
{
    let (name, set_name) = create_signal(app.name.clone());
    let (description, set_description) = create_signal(app.description.clone().unwrap_or_default());
    let (is_active, set_is_active) = create_signal(app.is_active);
    let (loading, set_loading) = create_signal(false);
    let (error, set_error) = create_signal(Option::<String>::None);

    let app_id = app.id.clone();

    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        set_loading.set(true);
        set_error.set(None);

        let name_val = name.get();
        let desc_val = description.get();
        let desc_opt = if desc_val.is_empty() {
            None
        } else {
            Some(desc_val)
        };
        let active = is_active.get();

        let app_id_clone = app_id.clone();
        let on_updated_clone = on_updated;
        spawn_local(async move {
            match api::update_application(&app_id_clone, Some(name_val), desc_opt, Some(active))
                .await
            {
                Ok(_) => {
                    on_updated_clone();
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
                <h2 style="margin-bottom: 1.5rem;">"Edit Application"</h2>

                <Show when=move || error.get().is_some()>
                    <div style="background-color: #fef2f2; border-left: 4px solid var(--danger); padding: 0.75rem; margin-bottom: 1rem;">
                        <p style="color: var(--danger); margin: 0; font-size: 0.875rem;">
                            {move || error.get().unwrap_or_default()}
                        </p>
                    </div>
                </Show>

                <form on:submit=on_submit>
                    <div style="margin-bottom: 1rem;">
                        <label class="label" for="edit-app-name">
                            "Application Name"
                            <span style="color: var(--error);">"*"</span>
                            <span style="color: var(--text-secondary); font-weight: normal; font-size: 0.875rem;">
                                {move || format!(" ({}/50)", name.get().len())}
                            </span>
                        </label>
                        <input
                            type="text"
                            id="edit-app-name"
                            class="input"
                            placeholder="My Awesome App"
                            prop:value=name
                            on:input=move |ev| set_name.set(event_target_value(&ev))
                            required
                            maxlength="50"
                            style=move || {
                                let valid = is_valid_length(&name.get(), 1, 50);
                                if !name.get().is_empty() && !valid {
                                    "border-color: var(--danger);"
                                } else {
                                    ""
                                }
                            }
                        />
                        <Show when=move || {
                            let n = name.get();
                            !n.is_empty() && length_error_message("Name", &n, 1, 50).is_some()
                        }>
                            <p style="color: var(--danger); font-size: 0.875rem; margin-top: 0.25rem; margin-bottom: 0;">
                                {move || length_error_message("Name", &name.get(), 1, 50).unwrap_or_default()}
                            </p>
                        </Show>
                    </div>

                    <div style="margin-bottom: 1rem;">
                        <label class="label" for="edit-app-description">
                            "Description"
                            <span style="color: var(--text-secondary); font-weight: normal;">"(optional)"</span>
                            <span style="color: var(--text-secondary); font-weight: normal; font-size: 0.875rem;">
                                {move || format!(" ({}/500)", description.get().len())}
                            </span>
                        </label>
                        <textarea
                            id="edit-app-description"
                            class="input"
                            placeholder="A brief description of your application..."
                            rows="3"
                            prop:value=description
                            on:input=move |ev| set_description.set(event_target_value(&ev))
                            maxlength="500"
                            style=move || {
                                let desc = description.get();
                                let valid = is_valid_length(&desc, 0, 500);
                                if !desc.is_empty() && !valid {
                                    "border-color: var(--danger);"
                                } else {
                                    ""
                                }
                            }
                        />
                        <Show when=move || {
                            let d = description.get();
                            !d.is_empty() && length_error_message("Description", &d, 0, 500).is_some()
                        }>
                            <p style="color: var(--danger); font-size: 0.875rem; margin-top: 0.25rem; margin-bottom: 0;">
                                {move || length_error_message("Description", &description.get(), 0, 500).unwrap_or_default()}
                            </p>
                        </Show>
                    </div>

                    <div style="margin-bottom: 1.5rem;">
                        <label class="label" style="display: flex; align-items: center; gap: 0.5rem; cursor: pointer;">
                            <input
                                type="checkbox"
                                checked=is_active
                                on:change=move |ev| set_is_active.set(event_target_checked(&ev))
                            />
                            "Active"
                        </label>
                        <p style="font-size: 0.875rem; color: var(--text-secondary); margin-top: 0.25rem;">
                            "Inactive applications won't receive webhook events"
                        </p>
                    </div>

                    <div style="display: flex; gap: 1rem; justify-content: flex-end;">
                        <button
                            type="button"
                            class="btn"
                            on:click=move |_| on_close()
                            disabled=loading
                        >
                            "Cancel"
                        </button>
                        <button
                            type="submit"
                            class="btn btn-primary"
                            disabled=move || {
                                loading.get() ||
                                name.get().is_empty() ||
                                !is_valid_length(&name.get(), 1, 50) ||
                                !is_valid_length(&description.get(), 0, 500)
                            }
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
pub fn ApplicationDetailPage() -> impl IntoView {
    view! {
        <div>
            <Navbar/>
            <div class="container main-content">
                <h1 style="margin-bottom: 2rem;">"Application Details"</h1>
                <div class="card">
                    <p>"Application details will be displayed here."</p>
                </div>
            </div>
        </div>
    }
}
