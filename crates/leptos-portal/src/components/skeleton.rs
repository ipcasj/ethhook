use leptos::*;

/// Skeleton loader for application cards
#[component]
pub fn SkeletonApplicationCard() -> impl IntoView {
    view! {
        <div class="card skeleton-card">
            <div style="display: flex; justify-content: space-between; align-items: start; margin-bottom: 1rem;">
                <div style="flex: 1;">
                    <div class="skeleton skeleton-title" style="width: 60%; margin-bottom: 0.75rem;"></div>
                    <div class="skeleton skeleton-text" style="width: 40%; height: 1.25rem;"></div>
                </div>
                <div class="skeleton skeleton-badge" style="width: 60px; height: 24px;"></div>
            </div>

            <div class="skeleton skeleton-text" style="width: 100%; margin-bottom: 0.5rem;"></div>
            <div class="skeleton skeleton-text" style="width: 80%; margin-bottom: 1rem;"></div>

            <div style="display: grid; grid-template-columns: repeat(2, 1fr); gap: 1rem; margin-bottom: 1rem; padding-top: 1rem; border-top: 1px solid var(--border);">
                <div>
                    <div class="skeleton skeleton-label" style="width: 50%; margin-bottom: 0.5rem;"></div>
                    <div class="skeleton skeleton-text" style="width: 90%;"></div>
                </div>
                <div>
                    <div class="skeleton skeleton-label" style="width: 60%; margin-bottom: 0.5rem;"></div>
                    <div class="skeleton skeleton-text" style="width: 70%;"></div>
                </div>
            </div>

            <div style="display: flex; gap: 0.5rem; justify-content: flex-end;">
                <div class="skeleton skeleton-button" style="width: 80px;"></div>
                <div class="skeleton skeleton-button" style="width: 80px;"></div>
                <div class="skeleton skeleton-button" style="width: 100px;"></div>
                <div class="skeleton skeleton-button" style="width: 50px;"></div>
            </div>
        </div>
    }
}

/// Skeleton loader for endpoint cards
#[component]
pub fn SkeletonEndpointCard() -> impl IntoView {
    view! {
        <div class="card skeleton-card">
            <div style="display: flex; justify-content: space-between; align-items: start; margin-bottom: 1rem;">
                <div style="flex: 1;">
                    <div class="skeleton skeleton-title" style="width: 70%; margin-bottom: 0.75rem;"></div>
                    <div class="skeleton skeleton-text" style="width: 50%;"></div>
                </div>
                <div class="skeleton skeleton-badge" style="width: 60px; height: 24px;"></div>
            </div>

            <div class="skeleton skeleton-text" style="width: 90%; margin-bottom: 1rem;"></div>

            <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 1rem; margin-bottom: 1rem; padding-top: 1rem; border-top: 1px solid var(--border);">
                <div>
                    <div class="skeleton skeleton-label" style="width: 60%; margin-bottom: 0.5rem;"></div>
                    <div class="skeleton skeleton-text" style="width: 80%;"></div>
                </div>
                <div>
                    <div class="skeleton skeleton-label" style="width: 70%; margin-bottom: 0.5rem;"></div>
                    <div class="skeleton skeleton-text" style="width: 70%;"></div>
                </div>
                <div>
                    <div class="skeleton skeleton-label" style="width: 65%; margin-bottom: 0.5rem;"></div>
                    <div class="skeleton skeleton-text" style="width: 75%;"></div>
                </div>
            </div>

            <div style="border-top: 1px solid var(--border); padding-top: 1rem;">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.75rem;">
                    <div class="skeleton skeleton-label" style="width: 100px;"></div>
                    <div style="display: flex; gap: 0.5rem;">
                        <div class="skeleton skeleton-button-small" style="width: 60px;"></div>
                        <div class="skeleton skeleton-button-small" style="width: 60px;"></div>
                        <div class="skeleton skeleton-button-small" style="width: 90px;"></div>
                    </div>
                </div>
                <div class="skeleton skeleton-text" style="width: 100%; height: 2.5rem;"></div>
            </div>
        </div>
    }
}

/// Skeleton loader for dashboard statistics
#[component]
pub fn SkeletonStatCard() -> impl IntoView {
    view! {
        <div class="card skeleton-card" style="text-align: center;">
            <div class="skeleton skeleton-label" style="width: 60%; margin: 0 auto 1rem;"></div>
            <div class="skeleton skeleton-stat" style="width: 80px; height: 3rem; margin: 0 auto;"></div>
        </div>
    }
}

/// Generic skeleton loader
#[component]
pub fn SkeletonLoader(
    #[prop(default = 3)] count: usize,
    #[prop(default = "application")] card_type: &'static str,
) -> impl IntoView {
    view! {
        <div style="display: grid; gap: 1.5rem;">
            {(0..count).map(|_| {
                match card_type {
                    "endpoint" => view! { <SkeletonEndpointCard/> }.into_view(),
                    "stat" => view! { <SkeletonStatCard/> }.into_view(),
                    _ => view! { <SkeletonApplicationCard/> }.into_view(),
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}
