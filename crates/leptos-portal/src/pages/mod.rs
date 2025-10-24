mod applications;
mod dashboard;
mod endpoints;
mod events;
mod home;
mod login;
mod register;
mod settings;

pub use applications::{ApplicationDetailPage, ApplicationsPage};
pub use dashboard::DashboardPage;
pub use endpoints::EndpointsPage;
pub use events::EventsPage;
pub use home::HomePage;
pub use login::LoginPage;
pub use register::RegisterPage;
pub use settings::SettingsPage;
