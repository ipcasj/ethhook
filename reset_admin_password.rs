use bcrypt::{hash, DEFAULT_COST};
use sqlx::SqlitePool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:config.db".to_string());
    
    let pool = SqlitePool::connect(&database_url).await?;
    
    let password = "SecureAdmin123!";
    let password_hash = hash(password, DEFAULT_COST)?;
    
    println!("Resetting password for admin@ethhook.io...");
    
    sqlx::query!(
        "UPDATE users SET password_hash = $1 WHERE email = 'admin@ethhook.io'",
        password_hash
    )
    .execute(&pool)
    .await?;
    
    println!("✅ Password reset successfully!");
    println!("Email: admin@ethhook.io");
    println!("Password: {}", password);
    
    Ok(())
}
