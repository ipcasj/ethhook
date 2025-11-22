use sqlx::SqlitePool;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenvy::dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");

    let email = env::args()
        .nth(1)
        .unwrap_or_else(|| "admin@ethhook.io".to_string());

    println!("Connecting to database...");
    let pool = PgPool::connect(&database_url).await?;

    println!("Setting admin status for: {email}");

    let result = sqlx::query!(
        "UPDATE users SET is_admin = true WHERE email = ? RETURNING id, email, is_admin",
        email
    )
    .fetch_one(&pool)
    .await?;

    println!("\n✓ SUCCESS: Admin status updated");
    println!("  User ID: {}", result.id);
    println!("  Email: {}", result.email);
    println!("  Is Admin: {:?}", result.is_admin);
    println!();

    Ok(())
}
