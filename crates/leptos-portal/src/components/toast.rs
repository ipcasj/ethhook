use leptos::*;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Toast {
    pub message: String,
    pub variant: ToastVariant,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ToastVariant {
    Success,
    Error,
    Info,
    Warning,
}

/// Global toast state
#[derive(Clone, Copy)]
pub struct ToastContext {
    pub toasts: RwSignal<Vec<(usize, Toast)>>,
    next_id: RwSignal<usize>,
}

impl ToastContext {
    pub fn new() -> Self {
        Self {
            toasts: create_rw_signal(Vec::new()),
            next_id: create_rw_signal(0),
        }
    }

    pub fn show(&self, message: String, variant: ToastVariant) {
        let id = self.next_id.get();
        self.next_id.update(|n| *n += 1);

        self.toasts.update(|toasts| {
            toasts.push((id, Toast { message, variant }));
        });

        // Auto-dismiss after 4 seconds
        let toasts = self.toasts;
        set_timeout(
            move || {
                toasts.update(|toasts| {
                    toasts.retain(|(toast_id, _)| *toast_id != id);
                });
            },
            Duration::from_secs(4),
        );
    }

    pub fn success(&self, message: impl Into<String>) {
        self.show(message.into(), ToastVariant::Success);
    }

    pub fn error(&self, message: impl Into<String>) {
        self.show(message.into(), ToastVariant::Error);
    }

    pub fn info(&self, message: impl Into<String>) {
        self.show(message.into(), ToastVariant::Info);
    }

    pub fn warning(&self, message: impl Into<String>) {
        self.show(message.into(), ToastVariant::Warning);
    }

    pub fn dismiss(&self, id: usize) {
        self.toasts.update(|toasts| {
            toasts.retain(|(toast_id, _)| *toast_id != id);
        });
    }
}

#[component]
pub fn ToastContainer() -> impl IntoView {
    let toast_ctx = use_context::<ToastContext>().expect("ToastContext must be provided");

    view! {
        <div style="position: fixed; top: 1rem; right: 1rem; z-index: 9999; display: flex; flex-direction: column; gap: 0.75rem; max-width: 400px;">
            <For
                each=move || toast_ctx.toasts.get()
                key=|(id, _)| *id
                children=move |(id, toast)| {
                    let variant_class = match toast.variant {
                        ToastVariant::Success => "toast-success",
                        ToastVariant::Error => "toast-error",
                        ToastVariant::Info => "toast-info",
                        ToastVariant::Warning => "toast-warning",
                    };

                    let icon = match toast.variant {
                        ToastVariant::Success => "✓",
                        ToastVariant::Error => "✕",
                        ToastVariant::Info => "ℹ",
                        ToastVariant::Warning => "⚠",
                    };

                    view! {
                        <div
                            class=format!("toast {}", variant_class)
                            style="animation: slideIn 0.3s ease-out;"
                        >
                            <div style="display: flex; align-items: center; gap: 0.75rem;">
                                <span style="font-size: 1.25rem; font-weight: bold;">{icon}</span>
                                <span style="flex: 1;">{toast.message.clone()}</span>
                                <button
                                    class="toast-close"
                                    on:click=move |_| toast_ctx.dismiss(id)
                                    aria-label="Close"
                                >
                                    "×"
                                </button>
                            </div>
                        </div>
                    }
                }
            />
        </div>
    }
}
