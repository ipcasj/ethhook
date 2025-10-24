use crate::components::{Navbar, SkeletonStatCard};
use crate::{api, auth};
use leptos::*;
use std::time::Duration;
use wasm_bindgen_futures::spawn_local;
use web_sys::console;

#[component]
pub fn DashboardPage() -> impl IntoView {
    let user = auth::get_user();
    let user_name = create_signal(user.map(|u| u.name).unwrap_or_default());
    let (total_apps, set_total_apps) = create_signal(0);
    let (total_endpoints, set_total_endpoints) = create_signal(0);
    let (events_today, set_events_today) = create_signal(0_i64);
    let (events_total, set_events_total) = create_signal(0_i64);
    let (success_rate, set_success_rate) = create_signal(0.0);
    let (total_deliveries, set_total_deliveries) = create_signal(0_i64);
    let (loading, set_loading) = create_signal(true);
    let (refresh_counter, set_refresh_counter) = create_signal(0); // Used to trigger refreshes

    // Function to load all dashboard data
    let load_dashboard_data = move || {
        set_loading.set(true);

        spawn_local(async move {
            // Load applications count
            if let Ok(response) = api::list_applications().await {
                set_total_apps.set(response.applications.len());

                // Load endpoints count for all applications
                let mut endpoint_count = 0;
                for app in response.applications {
                    if let Ok(endpoints) = api::list_endpoints(&app.id).await {
                        endpoint_count += endpoints.endpoints.len();
                    }
                }
                set_total_endpoints.set(endpoint_count);
            }

            // Load statistics from API
            if let Ok(stats) = api::get_dashboard_statistics().await {
                set_events_today.set(stats.events_today);
                set_events_total.set(stats.events_total);
                set_success_rate.set(stats.success_rate);
                set_total_deliveries.set(stats.total_deliveries);
                console::log_1(&"Dashboard data refreshed".into());
            }

            set_loading.set(false);
        });
    };

    // Initial load
    create_effect(move |_| {
        refresh_counter.get(); // Subscribe to refresh_counter
        load_dashboard_data();
    });

    // Set up auto-refresh every 30 seconds
    set_interval(
        move || {
            set_refresh_counter.update(|n| *n += 1);
        },
        Duration::from_secs(30),
    );

    // Manual refresh function for the refresh button
    let refresh_data = move |_| {
        set_refresh_counter.update(|n| *n += 1);
    };

    view! {
        <div>
            <Navbar/>
            <div class="container main-content">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 2rem;">
                    <h1>"Dashboard"</h1>
                    <button
                        class="btn"
                        on:click=refresh_data
                        style="display: flex; align-items: center; gap: 0.5rem;"
                    >
                        "🔄 Refresh"
                    </button>
                </div>

                <Show
                    when=move || !loading.get()
                    fallback=move || view! {
                        <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 1.5rem; margin-bottom: 2rem;">
                            <SkeletonStatCard/>
                            <SkeletonStatCard/>
                            <SkeletonStatCard/>
                            <SkeletonStatCard/>
                        </div>
                    }
                >
                    <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 1.5rem; margin-bottom: 2rem;">
                        <div class="stat-card stat-card-blue">
                            <div class="stat-card-icon">"📱"</div>
                            <div class="stat-card-label">"Total Applications"</div>
                            <div class="stat-card-value">
                                {move || total_apps.get().to_string()}
                            </div>
                        </div>
                        <div class="stat-card stat-card-green">
                            <div class="stat-card-icon">"🔗"</div>
                            <div class="stat-card-label">"Webhook Endpoints"</div>
                            <div class="stat-card-value">
                                {move || total_endpoints.get().to_string()}
                            </div>
                        </div>
                        <div class="stat-card stat-card-purple">
                            <div class="stat-card-icon">"📊"</div>
                            <div class="stat-card-label">"Events Today"</div>
                            <div class="stat-card-value">
                                {move || events_today.get().to_string()}
                            </div>
                            <div class="stat-card-trend">
                                {move || format!("{} total events", events_total.get())}
                            </div>
                        </div>
                        <div class="stat-card stat-card-orange">
                            <div class="stat-card-icon">"✅"</div>
                            <div class="stat-card-label">"Success Rate"</div>
                            <div class="stat-card-value">
                                {move || format!("{:.1}%", success_rate.get())}
                            </div>
                            <div class="stat-card-trend">
                                {move || format!("{} deliveries", total_deliveries.get())}
                            </div>
                        </div>
                    </div>

                    <Show
                        when=move || total_apps.get() == 0
                        fallback=move || view! {
                            <div class="card">
                                <h2 style="margin-bottom: 1rem;">"Quick Actions"</h2>
                                <div style="display: flex; gap: 1rem; flex-wrap: wrap;">
                                    <a href="/applications" class="btn btn-primary">
                                        "📱 View Applications"
                                    </a>
                                    <a href="/applications" class="btn">
                                        "+ Create New Application"
                                    </a>
                                </div>
                            </div>
                        }
                    >
                        <div class="card">
                            <h2 style="margin-bottom: 1rem;">"Welcome, " {user_name.0.get()} "!"</h2>
                            <p style="color: var(--text-secondary); margin-bottom: 1.5rem;">
                                "Get started by creating your first application and configuring webhook endpoints to receive blockchain events."
                            </p>
                            <a href="/applications" class="btn btn-primary">
                                "+ Create First Application"
                            </a>
                        </div>
                    </Show>
                </Show>

                <div class="card" style="margin-top: 2rem;">
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem;">
                        <h2>"Recent Events"</h2>
                        <a href="/events" class="btn">
                            "View All Events"
                        </a>
                    </div>
                    <p style="color: var(--text-secondary); text-align: center; padding: 2rem;">
                        "No events yet. Create an application and endpoint to start receiving events."
                    </p>
                </div>
            </div>
        </div>
    }
}
