use crate::components::Navbar;
use leptos::*;
use leptos_router::*;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <div>
            <Navbar/>
            <div class="container main-content">
                <div class="card" style="text-align: center; padding: 4rem 2rem;">
                    <h1 style="font-size: 2.5rem; margin-bottom: 1rem; color: var(--primary);">
                        "Welcome to EthHook"
                    </h1>
                    <p style="font-size: 1.25rem; color: var(--text-secondary); margin-bottom: 2rem;">
                        "Real-time Ethereum event monitoring and webhook delivery platform"
                    </p>
                    <div style="display: flex; gap: 1rem; justify-content: center;">
                        <div style="font-size: 1rem;">
                            <A href="/register" class="btn btn-primary">
                                "Get Started"
                            </A>
                        </div>
                        <div style="font-size: 1rem;">
                            <A href="/login" class="btn btn-secondary">
                                "Login"
                            </A>
                        </div>
                    </div>

                    <div style="margin-top: 4rem; text-align: left;">
                        <h2 style="font-size: 1.75rem; margin-bottom: 1.5rem;">"Features"</h2>
                        <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 2rem;">
                            <div class="card">
                                <h3 style="color: var(--primary); margin-bottom: 0.5rem;">"Real-time Monitoring"</h3>
                                <p style="color: var(--text-secondary);">
                                    "Monitor Ethereum blockchain events in real-time with low latency"
                                </p>
                            </div>
                            <div class="card">
                                <h3 style="color: var(--primary); margin-bottom: 0.5rem;">"Reliable Webhooks"</h3>
                                <p style="color: var(--text-secondary);">
                                    "Guaranteed delivery with exponential backoff and retry logic"
                                </p>
                            </div>
                            <div class="card">
                                <h3 style="color: var(--primary); margin-bottom: 0.5rem;">"Easy Integration"</h3>
                                <p style="color: var(--text-secondary);">
                                    "Simple REST API and webhook configuration for seamless integration"
                                </p>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
