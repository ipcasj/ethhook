use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};

const TOKEN_KEY: &str = "auth_token";
const USER_KEY: &str = "auth_user";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthUser {
    pub id: String, // UUID serialized as string
    pub email: String,
    pub name: String, // Changed from username to match API
}

pub fn save_token(token: &str) {
    let _ = LocalStorage::set(TOKEN_KEY, token);
}

pub fn get_token() -> Option<String> {
    LocalStorage::get(TOKEN_KEY).ok()
}

pub fn remove_token() {
    LocalStorage::delete(TOKEN_KEY);
}

pub fn save_user(user: &AuthUser) {
    let _ = LocalStorage::set(USER_KEY, user);
}

pub fn get_user() -> Option<AuthUser> {
    LocalStorage::get(USER_KEY).ok()
}

pub fn remove_user() {
    LocalStorage::delete(USER_KEY);
}

pub fn is_authenticated() -> bool {
    get_token().is_some()
}

pub fn logout() {
    remove_token();
    remove_user();
}
