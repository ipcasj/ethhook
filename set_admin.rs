// Quick script to set admin status for a user
use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    let pool = PgPool::connect(&database_url).await?;
    
    let email = std::env::args().nth(1)
        .unwrap_or_else(|| "admin@ethhook.io".to_string());
    
    let result = sqlx::query!(
        "UPDATE users SET is_admin = true WHERE email = $1 RETURNING id, email, is_admin",
        email
    )
    .fetch_one(&pool)
    .await?;
    
    println!("✓ Updated user:");
    println!("  ID: {}", result.id);
    println!("  Email: {}", result.email);
    println!("  Is Admin: {}", result.is_admin);
    
    Ok(())
}
