use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfile {
    pub login: String,
    pub name: Option<String>,
    pub avatar_url: String,
}

/// Fetch the authenticated GitHub user's profile.
/// The token is passed from the frontend (stored in useAuthStore).
#[tauri::command]
pub async fn get_user_profile(token: String) -> Result<UserProfile, String> {
    let resp = reqwest::Client::new()
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "tacoshell/0.1.0")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("GitHub API error: {}", resp.status()));
    }

    resp.json::<UserProfile>().await.map_err(|e| e.to_string())
}
