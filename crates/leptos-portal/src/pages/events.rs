use crate::api::{self, DeliveryAttempt, Event};
use crate::components::{Navbar, ToastContext};
use crate::utils::{format_date, truncate_hash};
use leptos::*;

#[component]
pub fn EventsPage() -> impl IntoView {
    let toast = use_context::<ToastContext>().expect("ToastContext must be provided");

    // State for events list
    let (events, set_events) = create_signal(Vec::<Event>::new());
    let (loading, set_loading) = create_signal(true);
    let (error, set_error) = create_signal(Option::<String>::None);

    // State for filters
    let (search_query, set_search_query) = create_signal(String::new());
    let (status_filter, set_status_filter) = create_signal(String::from("all"));
    let (selected_endpoint_id, set_selected_endpoint_id) = create_signal(Option::<String>::None);

    // State for endpoints (for the dropdown filter)
    let (endpoints, set_endpoints) = create_signal(Vec::<(String, String, String)>::new()); // (endpoint_id, app_name, endpoint_description)

    // State for event details modal
    let (show_details_modal, set_show_details_modal) = create_signal(false);
    let (selected_event, set_selected_event) = create_signal(Option::<Event>::None);
    let (delivery_attempts, set_delivery_attempts) = create_signal(Vec::<DeliveryAttempt>::new());
    let (loading_attempts, set_loading_attempts) = create_signal(false);

    // State for response details modal
    let (show_response_modal, set_show_response_modal) = create_signal(false);
    let (response_modal_title, set_response_modal_title) = create_signal(String::new());
    let (response_modal_content, set_response_modal_content) = create_signal(String::new());

    // Define reload_events function first
    let reload_events = move || {
        set_loading.set(true);
        set_error.set(None);

        let endpoint_id = selected_endpoint_id.get();

        spawn_local(async move {
            match api::list_events(endpoint_id.as_deref()).await {
                Ok(response) => {
                    set_events.set(response.events);
                    set_loading.set(false);
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_loading.set(false);
                }
            }
        });
    };

    // Load all endpoints from all applications for the filter dropdown
    create_effect(move |_| {
        spawn_local(async move {
            match api::list_applications().await {
                Ok(response) => {
                    let mut all_endpoints = Vec::new();

                    // Fetch endpoints for each application
                    for app in response.applications {
                        match api::list_endpoints(&app.id).await {
                            Ok(endpoint_response) => {
                                for endpoint in endpoint_response.endpoints {
                                    // Get first contract address and event signature
                                    let contract = endpoint
                                        .contract_addresses
                                        .first()
                                        .map(|addr| {
                                            if addr.len() > 10 {
                                                format!(
                                                    "{}...{}",
                                                    &addr[..6],
                                                    &addr[addr.len() - 4..]
                                                )
                                            } else {
                                                addr.clone()
                                            }
                                        })
                                        .unwrap_or_else(|| "N/A".to_string());

                                    let event = endpoint
                                        .event_signatures
                                        .first()
                                        .map(|sig| sig.split('(').next().unwrap_or("Event"))
                                        .unwrap_or("Event");

                                    // Create a descriptive label: "App Name - Contract Address (Event)"
                                    let description =
                                        format!("{} - {} ({})", app.name, contract, event);
                                    all_endpoints.push((
                                        endpoint.id,
                                        app.name.clone(),
                                        description,
                                    ));
                                }
                            }
                            Err(_) => {
                                // Skip applications with no endpoints or errors
                            }
                        }
                    }

                    set_endpoints.set(all_endpoints);
                }
                Err(e) => {
                    toast.error(format!("Failed to load endpoints: {e}"));
                }
            }
        });
    });

    // Load events on mount
    create_effect(move |_| {
        reload_events();
    });

    // Filter events based on search query and status
    let filtered_events = move || {
        let query = search_query.get().to_lowercase();
        let status = status_filter.get();

        events
            .get()
            .into_iter()
            .filter(|event| {
                // Filter by search query (tx hash, contract address, etc.)
                let matches_search = query.is_empty()
                    || event.transaction_hash.to_lowercase().contains(&query)
                    || event.contract_address.to_lowercase().contains(&query);

                // Filter by status (must match the display logic)
                let matches_status = match status.as_str() {
                    "delivered" => {
                        let successful = event.successful_deliveries.unwrap_or(0);
                        let total = event.delivery_count.unwrap_or(0);
                        total > 0 && successful == total
                    }
                    "failed" => {
                        let successful = event.successful_deliveries.unwrap_or(0);
                        let total = event.delivery_count.unwrap_or(0);
                        total > 0 && successful == 0
                    }
                    "partial" => {
                        let successful = event.successful_deliveries.unwrap_or(0);
                        let total = event.delivery_count.unwrap_or(0);
                        total > 0 && successful > 0 && successful < total
                    }
                    "pending" => event.delivery_count.unwrap_or(0) == 0,
                    _ => true, // "all" or any other value
                };

                matches_search && matches_status
            })
            .collect::<Vec<_>>()
    };

    // Handle opening event details
    let view_event_details = move |event: Event| {
        set_selected_event.set(Some(event.clone()));
        set_show_details_modal.set(true);
        set_loading_attempts.set(true);

        // Load delivery attempts for this event
        let event_id = event.id.clone();
        spawn_local(async move {
            match api::list_delivery_attempts(Some(&event_id), None).await {
                Ok(response) => {
                    set_delivery_attempts.set(response.delivery_attempts);
                    set_loading_attempts.set(false);
                }
                Err(e) => {
                    toast.error(format!("Failed to load delivery attempts: {e}"));
                    set_loading_attempts.set(false);
                }
            }
        });
    };

    // Copy to clipboard utility
    let copy_to_clipboard = move |text: String| {
        let window = web_sys::window().expect("no global window exists");
        let clipboard = window.navigator().clipboard();
        let _ = clipboard.write_text(&text);
        toast.success("Copied to clipboard");
    };

    // Format the event data for display
    let format_data = move |data: &str| -> String {
        if data.len() <= 20 {
            data.to_string()
        } else {
            format!("{}...", &data[0..20])
        }
    };

    view! {
        <div>
            <Navbar/>
            <div class="container main-content">
                <h1 style="margin-bottom: 2rem;">"Event History"</h1>

                <div class="card" style="margin-bottom: 1.5rem;">
                    <div style="display: flex; gap: 1rem; flex-wrap: wrap;">
                        // Search input
                        <input
                            type="text"
                            class="input"
                            placeholder="Search by transaction hash or contract..."
                            style="flex: 1; min-width: 250px;"
                            on:input=move |ev| {
                                set_search_query.set(event_target_value(&ev));
                            }
                            prop:value=move || search_query.get()
                        />

                        // Endpoint filter
                        <select
                            class="input"
                            style="min-width: 250px;"
                            on:change=move |ev| {
                                let value = event_target_value(&ev);
                                set_selected_endpoint_id.set(if value.is_empty() { None } else { Some(value) });
                                reload_events();
                            }
                        >
                            <option value="">"All Endpoints"</option>
                            <For
                                each=move || endpoints.get()
                                key=|(id, _, _)| id.clone()
                                children=move |(endpoint_id, _app_name, description)| {
                                    view! { <option value={endpoint_id}>{description}</option> }
                                }
                            />
                        </select>

                        // Status filter
                        <select
                            class="input"
                            style="min-width: 150px;"
                            on:change=move |ev| {
                                set_status_filter.set(event_target_value(&ev));
                            }
                        >
                            <option value="all">"All Status"</option>
                            <option value="pending">"Pending"</option>
                            <option value="delivered">"Delivered"</option>
                            <option value="partial">"Partial"</option>
                            <option value="failed">"Failed"</option>
                        </select>

                        // Refresh button
                        <button
                            class="btn"
                            style="min-width: 100px;"
                            on:click=move |_| reload_events()
                        >
                            "Refresh"
                        </button>
                    </div>
                </div>

                <Show when=move || error.get().is_some()>
                    <div class="card" style="background-color: #fef2f2; border-left: 4px solid var(--danger); margin-bottom: 1rem;">
                        <p style="color: var(--danger); margin: 0;">{move || error.get().unwrap_or_default()}</p>
                    </div>
                </Show>

                <Show
                    when=move || loading.get()
                    fallback=move || view! {
                        <Show
                            when=move || !events.get().is_empty()
                            fallback=move || view! {
                                <div class="card" style="text-align: center; padding: 3rem;">
                                    <p style="color: var(--text-secondary); margin-bottom: 1.5rem;">
                                        "No events found. Events will appear here once your endpoints receive blockchain events."
                                    </p>
                                </div>
                            }
                        >
                            <Show
                                when=move || !filtered_events().is_empty()
                                fallback=move || view! {
                                    <div class="card" style="text-align: center; padding: 2rem;">
                                        <p style="color: var(--text-secondary);">
                                            "No events match your filters."
                                        </p>
                                    </div>
                                }
                            >
                                <div class="card">
                                    <div style="overflow-x: auto;">
                                        <table class="table">
                                            <thead>
                                                <tr>
                                                    <th>"Block #"</th>
                                                    <th>"Transaction"</th>
                                                    <th>"Contract"</th>
                                                    <th>"Time"</th>
                                                    <th>"Status"</th>
                                                    <th>"Action"</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                <For
                                                    each=move || filtered_events()
                                                    key=|event| event.id.clone()
                                                    children=move |event: Event| {
                                                        // Calculate status display
                                                        let (status_text, status_class) = {
                                                            let total = event.delivery_count.unwrap_or(0);
                                                            let success = event.successful_deliveries.unwrap_or(0);

                                                            if total == 0 {
                                                                ("Pending", "pending")
                                                            } else if success == 0 {
                                                                ("Failed", "failed")
                                                            } else if success < total {
                                                                ("Partial", "partial")
                                                            } else {
                                                                ("Delivered", "success")
                                                            }
                                                        };

                                                        // Clone the event ID for the view_event_details function
                                                        let event_id = event.id.clone();

                                                        // Store transaction hash and contract address for copy operations
                                                        let tx_hash = event.transaction_hash.clone();
                                                        let contract_addr = event.contract_address.clone();

                                                        view! {
                                                            <tr>
                                                                <td>{event.block_number}</td>
                                                                <td>
                                                                    <span
                                                                        class="hash-value"
                                                                        title={event.transaction_hash.clone()}
                                                                        on:click=move |_| {
                                                                            // Clone inside closure to avoid borrowing moved value
                                                                            let tx = tx_hash.clone();
                                                                            copy_to_clipboard(tx);
                                                                        }
                                                                    >
                                                                        {truncate_hash(&event.transaction_hash, 10)}
                                                                    </span>
                                                                </td>
                                                                <td>
                                                                    <span
                                                                        class="hash-value"
                                                                        title={event.contract_address.clone()}
                                                                        on:click=move |_| {
                                                                            // Clone inside closure to avoid borrowing moved value
                                                                            let addr = contract_addr.clone();
                                                                            copy_to_clipboard(addr);
                                                                        }
                                                                    >
                                                                        {truncate_hash(&event.contract_address, 10)}
                                                                    </span>
                                                                </td>
                                                                <td>{format_date(&event.ingested_at)}</td>
                                                                <td>
                                                                    <span class=format!("status-badge {}", status_class)>
                                                                        {status_text}
                                                                    </span>
                                                                </td>
                                                                <td>
                                                                    <button
                                                                        class="btn btn-sm"
                                                                        on:click=move |_| {
                                                                            // Get the event from the events list by ID
                                                                            let all_events = events.get_untracked();
                                                                            let event_to_view = all_events.iter()
                                                                                .find(|e| e.id == event_id)
                                                                                .unwrap()
                                                                                .clone();
                                                                            view_event_details(event_to_view);
                                                                        }
                                                                    >
                                                                        "View"
                                                                    </button>
                                                                </td>
                                                            </tr>
                                                        }
                                                    }
                                                />
                                            </tbody>
                                        </table>
                                    </div>
                                </div>
                            </Show>
                        </Show>
                    }
                >
                    <div class="card" style="text-align: center; padding: 2rem;">
                        <p style="color: var(--text-secondary);">
                            "Loading events..."
                        </p>
                    </div>
                </Show>

                // Event details modal
                <Show when=move || show_details_modal.get() && selected_event.get().is_some()>
                    <div class="modal-overlay" on:click=move |_| set_show_details_modal.set(false)>
                        <div class="modal modal-lg" on:click=move |e| e.stop_propagation()>
                            <h2>"Event Details"</h2>
                            {move || {
                                let event = selected_event.get().unwrap();
                                view! {
                                    <div class="event-details">
                                        <div class="details-grid">
                                            <div class="details-row">
                                                <div class="details-label">"Transaction Hash"</div>
                                                <div class="details-value">
                                                    <span
                                                        class="hash-value"
                                                        on:click=move |_| {
                                                            // Clone inside the closure
                                                            let tx_hash = selected_event.with(|e| e.as_ref().unwrap().transaction_hash.clone());
                                                            copy_to_clipboard(tx_hash);
                                                        }
                                                    >
                                                        {event.transaction_hash.clone()}
                                                    </span>
                                                </div>
                                            </div>
                                            <div class="details-row">
                                                <div class="details-label">"Contract Address"</div>
                                                <div class="details-value">
                                                    <span
                                                        class="hash-value"
                                                        on:click=move |_| {
                                                            // Clone inside the closure
                                                            let addr = selected_event.with(|e| e.as_ref().unwrap().contract_address.clone());
                                                            copy_to_clipboard(addr);
                                                        }
                                                    >
                                                        {event.contract_address.clone()}
                                                    </span>
                                                </div>
                                            </div>
                                            <div class="details-row">
                                                <div class="details-label">"Block"</div>
                                                <div class="details-value">{event.block_number}</div>
                                            </div>
                                            <div class="details-row">
                                                <div class="details-label">"Log Index"</div>
                                                <div class="details-value">{event.log_index}</div>
                                            </div>
                                            <div class="details-row">
                                                <div class="details-label">"Time"</div>
                                                <div class="details-value">{format_date(&event.ingested_at)}</div>
                                            </div>
                                        </div>

                                        <h3 style="margin-top: 1.5rem;">"Topics"</h3>
                                        <div class="card" style="margin-bottom: 1rem;">
                                            <For
                                                each=move || event.topics.clone()
                                                key=|topic| topic.clone()
                                                children=move |topic| {
                                                    let topic_for_click = topic.clone();
                                                    let topic_for_display = topic.clone();
                                                    view! {
                                                        <div class="topic-item">
                                                            <span
                                                                class="hash-value"
                                                                on:click=move |_| {
                                                                    let t = topic_for_click.clone();
                                                                    copy_to_clipboard(t);
                                                                }
                                                            >
                                                                {topic_for_display}
                                                            </span>
                                                        </div>
                                                    }
                                                }
                                            />
                                        </div>

                                        <h3 style="margin-top: 1.5rem;">"Data"</h3>
                                        <div class="card" style="margin-bottom: 1.5rem;">
                                            <div style="font-family: monospace; word-break: break-all;">
                                                <span
                                                    class="hash-value"
                                                    on:click=move |_| {
                                                        let data = selected_event.with(|e| e.as_ref().unwrap().data.clone());
                                                        copy_to_clipboard(data);
                                                    }
                                                >
                                                    {event.data.clone()}
                                                </span>
                                            </div>
                                        </div>

                                        <h3>"Delivery Attempts"</h3>
                                        <Show
                                            when=move || !loading_attempts.get()
                                            fallback=move || view! {
                                                <p style="color: var(--text-secondary); text-align: center;">
                                                    "Loading delivery attempts..."
                                                </p>
                                            }
                                        >
                                            <Show
                                                when=move || !delivery_attempts.get().is_empty()
                                                fallback=move || view! {
                                                    <p style="color: var(--text-secondary); text-align: center;">
                                                        "No delivery attempts found for this event."
                                                    </p>
                                                }
                                            >
                                                <div style="overflow-x: auto;">
                                                    <table class="table">
                                                        <thead>
                                                            <tr>
                                                                <th>"Endpoint"</th>
                                                                <th>"Attempt #"</th>
                                                                <th>"Time"</th>
                                                                <th>"Status"</th>
                                                                <th>"Duration"</th>
                                                                <th>"Response"</th>
                                                            </tr>
                                                        </thead>
                                                        <tbody>
                                                            <For
                                                                each=move || delivery_attempts.get()
                                                                key=|attempt| attempt.id.clone()
                                                                children=move |attempt: DeliveryAttempt| {
                                                                    let status_class = if attempt.success.unwrap_or(false) {
                                                                        "success"
                                                                    } else {
                                                                        "failed"
                                                                    };

                                                                    let response_body = attempt.response_body.clone().unwrap_or_default();
                                                                    let error_message = attempt.error_message.clone().unwrap_or_default();
                                                                    let display_message = if !error_message.is_empty() {
                                                                        error_message.clone()
                                                                    } else if !response_body.is_empty() {
                                                                        response_body.clone()
                                                                    } else {
                                                                        "—".to_string()
                                                                    };

                                                                    // Show View Full button if message is long, has newlines, or looks like JSON/structured data
                                                                    let is_expandable = display_message.len() > 50
                                                                        || display_message.contains('\n')
                                                                        || display_message.starts_with('{')
                                                                        || display_message.starts_with('[')
                                                                        || display_message != "—";

                                                                    // Store values for use in closure
                                                                    let full_response = store_value(response_body);
                                                                    let full_error = store_value(error_message);

                                                                    view! {
                                                                        <tr>
                                                                            <td>{attempt.endpoint_name.clone()}</td>
                                                                            <td>{attempt.attempt_number}</td>
                                                                            <td>{format_date(&attempt.attempted_at)}</td>
                                                                            <td>
                                                                                <span class=format!("status-badge {}", status_class)>
                                                                                    {if attempt.success.unwrap_or(false) { "Success" } else { "Failed" }}
                                                                                    {move || attempt.http_status_code.map(|code| format!(" ({code})")).unwrap_or_default()}
                                                                                </span>
                                                                            </td>
                                                                            <td>
                                                                                {move || attempt.duration_ms.map(|d| format!("{d}ms")).unwrap_or_else(|| "—".to_string())}
                                                                            </td>
                                                                            <td style="max-width: 500px; word-wrap: break-word; white-space: pre-wrap; font-family: monospace; font-size: 0.85rem;">
                                                                                <span style="display: block; overflow-x: auto; word-break: break-word; white-space: pre-wrap;">{display_message.clone()}</span>
                                                                                {move || if is_expandable {
                                                                                    view! {
                                                                                        <button
                                                                                            class="btn-link"
                                                                                            style="margin-left: 8px; font-size: 0.875rem; padding: 2px 8px;"
                                                                                            on:click=move |_| {
                                                                                                let err = full_error.get_value();
                                                                                                let resp = full_response.get_value();
                                                                                                let title = if !err.is_empty() {
                                                                                                    "Error Message"
                                                                                                } else {
                                                                                                    "Response Body"
                                                                                                };
                                                                                                let content = if !err.is_empty() {
                                                                                                    err
                                                                                                } else {
                                                                                                    resp
                                                                                                };
                                                                                                set_response_modal_title.set(title.to_string());
                                                                                                set_response_modal_content.set(content);
                                                                                                set_show_response_modal.set(true);
                                                                                            }
                                                                                        >
                                                                                            "View Full"
                                                                                        </button>
                                                                                    }.into_view()
                                                                                } else {
                                                                                    view! { <span></span> }.into_view()
                                                                                }}
                                                                            </td>
                                                                        </tr>
                                                                    }
                                                                }
                                                            />
                                                        </tbody>
                                                    </table>
                                                </div>
                                            </Show>
                                        </Show>

                                        <div style="text-align: right; margin-top: 1.5rem;">
                                            <button
                                                class="btn"
                                                on:click=move |_| set_show_details_modal.set(false)
                                            >
                                                "Close"
                                            </button>
                                        </div>
                                    </div>
                                }
                            }}
                        </div>
                    </div>
                </Show>

                // Response Details Modal
                <Show when=move || show_response_modal.get()>
                    <div class="modal-overlay" on:click=move |_| set_show_response_modal.set(false)>
                        <div class="modal modal-lg" on:click=move |e| e.stop_propagation()>
                            {move || {
                                view! {
                                    <div>
                                        <h2>{response_modal_title.get()}</h2>
                                        <div style="background-color: var(--bg-secondary); padding: 1rem; border-radius: 8px; margin-top: 1rem; max-height: 500px; overflow-y: auto;">
                                            <pre style="margin: 0; white-space: pre-wrap; word-wrap: break-word; font-family: 'Courier New', monospace; font-size: 0.875rem;">
                                                {response_modal_content.get()}
                                            </pre>
                                        </div>
                                        <div style="text-align: right; margin-top: 1.5rem;">
                                            <button
                                                class="btn"
                                                on:click=move |_| set_show_response_modal.set(false)
                                            >
                                                "Close"
                                            </button>
                                        </div>
                                    </div>
                                }
                            }}
                        </div>
                    </div>
                </Show>
            </div>
        </div>
    }
}
