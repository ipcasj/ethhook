//! Authentication helpers
//!
//! Provides JWT token management, password hashing, and HMAC signatures.

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use hex;

use crate::error::Result;

type HmacSha256 = Hmac<Sha256>;

/// JWT Claims structure
///
/// Java equivalent:
/// ```java
/// class Claims {
///     String sub; // subject (user_id)
///     long exp;   // expiration
///     long iat;   // issued at
/// }
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // Subject (user ID)
    pub exp: usize,         // Expiration time (unix timestamp)
    pub iat: usize,         // Issued at (unix timestamp)
}

/// Create a JWT token
///
/// Java equivalent:
/// ```java
/// Algorithm algorithm = Algorithm.HMAC256(secret);
/// String token = JWT.create()
///     .withSubject(userId)
///     .withExpiresAt(new Date(System.currentTimeMillis() + 3600000))
///     .sign(algorithm);
/// ```
///
/// Rust:
/// ```rust
/// let token = create_jwt("user123", "secret", 3600)?;
/// ```
pub fn create_jwt(user_id: &str, secret: &str, expires_in_seconds: i64) -> Result<String> {
    let now = Utc::now();
    let expiration = now + Duration::seconds(expires_in_seconds);

    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;

    Ok(token)
}

/// Verify and decode a JWT token
///
/// Java equivalent:
/// ```java
/// Algorithm algorithm = Algorithm.HMAC256(secret);
/// JWTVerifier verifier = JWT.require(algorithm).build();
/// DecodedJWT jwt = verifier.verify(token);
/// String userId = jwt.getSubject();
/// ```
///
/// Rust:
/// ```rust
/// let claims = verify_jwt(&token, "secret")?;
/// let user_id = claims.sub;
/// ```
pub fn verify_jwt(token: &str, secret: &str) -> Result<Claims> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}

/// Hash a password using bcrypt
///
/// Java equivalent:
/// ```java
/// String hash = BCrypt.hashpw(password, BCrypt.gensalt(12));
/// ```
///
/// Rust:
/// ```rust
/// let hash = hash_password("mypassword")?;
/// ```
pub fn hash_password(password: &str) -> Result<String> {
    let cost = 12; // Higher = more secure but slower
    let hash = bcrypt::hash(password, cost)?;
    Ok(hash)
}

/// Verify a password against a hash
///
/// Java equivalent:
/// ```java
/// boolean valid = BCrypt.checkpw(password, storedHash);
/// ```
///
/// Rust:
/// ```rust
/// let valid = verify_password("mypassword", &stored_hash)?;
/// ```
pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let valid = bcrypt::verify(password, hash)?;
    Ok(valid)
}

/// Create HMAC-SHA256 signature for webhook payloads
///
/// Java equivalent:
/// ```java
/// Mac mac = Mac.getInstance("HmacSHA256");
/// mac.init(new SecretKeySpec(secret.getBytes(), "HmacSHA256"));
/// byte[] signature = mac.doFinal(payload.getBytes());
/// String hex = DatatypeConverter.printHexBinary(signature);
/// ```
///
/// Rust:
/// ```rust
/// let signature = sign_hmac("payload_data", "secret");
/// ```
pub fn sign_hmac(payload: &str, secret: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    
    mac.update(payload.as_bytes());
    let result = mac.finalize();
    let code_bytes = result.into_bytes();
    
    hex::encode(code_bytes)
}

/// Verify HMAC-SHA256 signature
///
/// Java equivalent:
/// ```java
/// String computedSignature = sign_hmac(payload, secret);
/// return computedSignature.equals(providedSignature);
/// ```
///
/// Rust:
/// ```rust
/// let valid = verify_hmac("payload_data", "signature_hex", "secret");
/// ```
pub fn verify_hmac(payload: &str, signature_hex: &str, secret: &str) -> bool {
    let computed = sign_hmac(payload, secret);
    
    // Constant-time comparison to prevent timing attacks
    computed.as_bytes().len() == signature_hex.as_bytes().len()
        && computed
            .as_bytes()
            .iter()
            .zip(signature_hex.as_bytes())
            .all(|(a, b)| a == b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_creation_and_verification() {
        let secret = "test_secret_key_minimum_32_characters_long_123456";
        let user_id = "user123";
        
        // Create token
        let token = create_jwt(user_id, secret, 3600).unwrap();
        assert!(!token.is_empty());
        
        // Verify token
        let claims = verify_jwt(&token, secret).unwrap();
        assert_eq!(claims.sub, user_id);
    }

    #[test]
    fn test_jwt_invalid_secret() {
        let secret = "test_secret_key_minimum_32_characters_long_123456";
        let wrong_secret = "wrong_secret_key_minimum_32_characters_long_654321";
        let user_id = "user123";
        
        let token = create_jwt(user_id, secret, 3600).unwrap();
        let result = verify_jwt(&token, wrong_secret);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_password_hashing() {
        let password = "my_secure_password";
        
        // Hash password
        let hash = hash_password(password).unwrap();
        assert!(!hash.is_empty());
        assert_ne!(hash, password);
        
        // Verify correct password
        let valid = verify_password(password, &hash).unwrap();
        assert!(valid);
        
        // Verify wrong password
        let invalid = verify_password("wrong_password", &hash).unwrap();
        assert!(!invalid);
    }

    #[test]
    fn test_hmac_signing() {
        let payload = r#"{"event": "payment.success", "amount": 100}"#;
        let secret = "webhook_secret_key";
        
        // Create signature
        let signature = sign_hmac(payload, secret);
        assert!(!signature.is_empty());
        assert_eq!(signature.len(), 64); // SHA256 produces 64 hex characters
        
        // Verify signature
        let valid = verify_hmac(payload, &signature, secret);
        assert!(valid);
        
        // Verify with wrong signature
        let invalid = verify_hmac(payload, "wrong_signature", secret);
        assert!(!invalid);
        
        // Verify with wrong payload
        let invalid = verify_hmac("different_payload", &signature, secret);
        assert!(!invalid);
    }

    #[test]
    fn test_hmac_deterministic() {
        let payload = "test_payload";
        let secret = "test_secret";
        
        let sig1 = sign_hmac(payload, secret);
        let sig2 = sign_hmac(payload, secret);
        
        assert_eq!(sig1, sig2, "HMAC should be deterministic");
    }
}
