use bcrypt::{hash, DEFAULT_COST};
use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://ethhook:password@localhost:5432/ethhook".to_string());
    
    let pool = PgPool::connect(&database_url).await?;
    
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
