#![allow(dead_code)] // Component under development

use leptos::*;

/// Popular Sepolia events with real-world data
#[derive(Clone, Debug)]
pub struct EventRecommendation {
    pub contract_name: String,
    pub contract_address: String,
    pub event_signature: String,
    pub description: String,
    pub volume: String,
    pub use_case: String,
    pub emoji: String,
}

pub fn get_popular_sepolia_events() -> Vec<EventRecommendation> {
    vec![
        EventRecommendation {
            contract_name: "Sepolia USDC".to_string(),
            contract_address: "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238".to_string(),
            event_signature: "Transfer(address indexed,address indexed,uint256)".to_string(),
            description: "USDC transfers on Sepolia testnet".to_string(),
            volume: "50-200 events/hour".to_string(),
            use_case: "Perfect for testing high-volume webhook delivery, payment tracking, and DeFi integrations".to_string(),
            emoji: "💵".to_string(),
        },
        EventRecommendation {
            contract_name: "Sepolia WETH".to_string(),
            contract_address: "0x7b79995e5f793A07Bc00c21412e50Ecae098E7f9".to_string(),
            event_signature: "Transfer(address indexed,address indexed,uint256)".to_string(),
            description: "WETH (Wrapped ETH) transfers".to_string(),
            volume: "20-50 events/hour".to_string(),
            use_case: "Most popular token in Sepolia DeFi. Great for testing token swaps and liquidity events".to_string(),
            emoji: "💎".to_string(),
        },
        EventRecommendation {
            contract_name: "Sepolia WETH Deposits".to_string(),
            contract_address: "0x7b79995e5f793A07Bc00c21412e50Ecae098E7f9".to_string(),
            event_signature: "Deposit(address indexed,uint256)".to_string(),
            description: "ETH deposited to become WETH".to_string(),
            volume: "10-30 events/hour".to_string(),
            use_case: "Track when users wrap ETH to WETH. Common in DeFi applications before swaps".to_string(),
            emoji: "⬇️".to_string(),
        },
        EventRecommendation {
            contract_name: "Sepolia WETH Withdrawals".to_string(),
            contract_address: "0x7b79995e5f793A07Bc00c21412e50Ecae098E7f9".to_string(),
            event_signature: "Withdrawal(address indexed,uint256)".to_string(),
            description: "WETH unwrapped back to ETH".to_string(),
            volume: "5-20 events/hour".to_string(),
            use_case: "Monitor when users unwrap WETH back to ETH. Important for DeFi exit tracking".to_string(),
            emoji: "⬆️".to_string(),
        },
        EventRecommendation {
            contract_name: "Sepolia DAI".to_string(),
            contract_address: "0x68194a729C2450ad26072b3D33ADaCbcef39D574".to_string(),
            event_signature: "Transfer(address indexed,address indexed,uint256)".to_string(),
            description: "DAI stablecoin transfers".to_string(),
            volume: "15-40 events/hour".to_string(),
            use_case: "Popular stablecoin for testing DeFi protocols and payment flows".to_string(),
            emoji: "🪙".to_string(),
        },
        EventRecommendation {
            contract_name: "Uniswap V2 Pair Created".to_string(),
            contract_address: "0x7E0987E5b3a30e3f2828572Bb659A548460a3003".to_string(),
            event_signature: "PairCreated(address indexed,address indexed,address,uint256)".to_string(),
            description: "New Uniswap trading pairs created".to_string(),
            volume: "1-5 events/hour".to_string(),
            use_case: "Track new DEX pairs being created. Useful for DeFi analytics and new token discovery".to_string(),
            emoji: "🦄".to_string(),
        },
        EventRecommendation {
            contract_name: "ERC721 (NFT) Transfers".to_string(),
            contract_address: "0x0000000000000000000000000000000000000000".to_string(), // Wildcard
            event_signature: "Transfer(address indexed,address indexed,uint256 indexed)".to_string(),
            description: "NFT transfers across all collections".to_string(),
            volume: "5-15 events/hour".to_string(),
            use_case: "Monitor NFT marketplace activity and ownership changes. Note: indexed uint256 distinguishes NFTs from ERC20".to_string(),
            emoji: "🎨".to_string(),
        },
        EventRecommendation {
            contract_name: "All ERC20 Transfers".to_string(),
            contract_address: "0x0000000000000000000000000000000000000000".to_string(), // Wildcard
            event_signature: "Transfer(address indexed,address indexed,uint256)".to_string(),
            description: "All ERC20 token transfers on Sepolia".to_string(),
            volume: "100-500 events/hour".to_string(),
            use_case: "🔥 VERY HIGH VOLUME - Track all token movements. Great for testing system performance under load".to_string(),
            emoji: "🌊".to_string(),
        },
        EventRecommendation {
            contract_name: "All Approvals".to_string(),
            contract_address: "0x0000000000000000000000000000000000000000".to_string(), // Wildcard
            event_signature: "Approval(address indexed,address indexed,uint256)".to_string(),
            description: "Token approval events (ERC20)".to_string(),
            volume: "30-80 events/hour".to_string(),
            use_case: "Monitor when users approve contracts to spend their tokens. Critical for DeFi security monitoring".to_string(),
            emoji: "✅".to_string(),
        },
    ]
}

#[component]
pub fn EventRecommendationTooltip() -> impl IntoView {
    let (show_tooltip, set_show_tooltip) = create_signal(false);
    let recommendations = create_signal(get_popular_sepolia_events());

    view! {
        <div class="event-recommendation-container">
            <button
                class="info-button"
                on:mouseenter=move |_| set_show_tooltip.set(true)
                on:mouseleave=move |_| set_show_tooltip.set(false)
                on:click=move |e| {
                    e.prevent_default();
                    set_show_tooltip.update(|v| *v = !*v);
                }
                title="Show popular event recommendations"
            >
                "💡 Popular Events"
            </button>

            <Show when=move || show_tooltip.get()>
                <div class="event-recommendations-tooltip">
                    <h4 style="margin-top: 0; margin-bottom: 1rem; color: var(--primary);">
                        "🔥 Most Active Sepolia Events"
                    </h4>
                    <p style="font-size: 0.875rem; color: var(--text-secondary); margin-bottom: 1rem;">
                        "Click any recommendation to auto-fill the form:"
                    </p>

                    <div class="recommendations-list">
                        {move || recommendations.0.get().iter().map(|rec| {
                            let emoji = rec.emoji.clone();
                            let contract_name = rec.contract_name.clone();
                            let volume = rec.volume.clone();
                            let contract_address = rec.contract_address.clone();
                            let event_signature = rec.event_signature.clone();
                            let use_case = rec.use_case.clone();

                            view! {
                                <div class="recommendation-item">
                                    <div class="rec-header">
                                        <span class="rec-emoji">{emoji}</span>
                                        <strong>{contract_name}</strong>
                                        <span class="rec-volume">{volume}</span>
                                    </div>
                                    <div class="rec-details">
                                        <div class="rec-contract">
                                            <span class="rec-label">"Contract:"</span>
                                            <code class="rec-code">{contract_address}</code>
                                        </div>
                                        <div class="rec-event">
                                            <span class="rec-label">"Event:"</span>
                                            <code class="rec-code">{event_signature}</code>
                                        </div>
                                        <p class="rec-use-case">{use_case}</p>
                                    </div>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>

                    <div class="tooltip-footer">
                        <p style="font-size: 0.75rem; color: var(--text-secondary); margin: 0.5rem 0 0 0;">
                            "💡 Tip: Use 0x00...00 as contract address to match all contracts"
                        </p>
                    </div>
                </div>
            </Show>
        </div>
    }
}

#[component]
pub fn QuickEventButtons<F>(on_select: F) -> impl IntoView
where
    F: Fn(String, String) + 'static + Clone,
{
    let quick_events = vec![
        (
            "💵 USDC Transfers",
            "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238",
            "Transfer(address,address,uint256)",
            "50-200/hr",
        ),
        (
            "💎 WETH Transfers",
            "0x7b79995e5f793A07Bc00c21412e50Ecae098E7f9",
            "Transfer(address,address,uint256)",
            "20-50/hr",
        ),
        (
            "🌊 All ERC20 Transfers",
            "0x0000000000000000000000000000000000000000",
            "Transfer(address,address,uint256)",
            "100-500/hr",
        ),
    ];

    view! {
        <div class="quick-event-buttons">
            <p style="font-size: 0.875rem; color: var(--text-secondary); margin-bottom: 0.5rem;">
                "🚀 Quick Start (High Volume):"
            </p>
            <div style="display: flex; gap: 0.5rem; flex-wrap: wrap;">
                {quick_events.into_iter().map(|(name, contract, event, volume)| {
                    let on_select_clone = on_select.clone();
                    let contract_str = contract.to_string();
                    let event_str = event.to_string();

                    view! {
                        <button
                            type="button"
                            class="quick-event-btn"
                            on:click=move |_| {
                                on_select_clone.clone()(contract_str.clone(), event_str.clone());
                            }
                            title=format!("Click to auto-fill: {} events", volume)
                        >
                            {name} <span class="volume-badge">{volume}</span>
                        </button>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}
