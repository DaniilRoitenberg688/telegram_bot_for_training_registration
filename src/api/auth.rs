// Авторизация запросов из Telegram Mini App.
//
// Фронт присылает заголовок `Authorization: tma <initData>`. initData подписан токеном бота,
// поэтому подделать id пользователя нельзя. Алгоритм проверки:
// https://core.telegram.org/bots/webapps#validating-data-received-via-the-mini-app
//
// Для разработки в обычном браузере можно выставить API_AUTH_DISABLED=true —
// тогда id берётся из заголовка `X-Dev-User-Id` без проверки. На сервере так делать нельзя.

use std::{
    collections::BTreeMap,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{extract::FromRequestParts, http::request::Parts};
use hmac::{Hmac, KeyInit, Mac};
use serde::Deserialize;
use sha2::Sha256;

use crate::api::{errors::ApiError, state::ApiState};

/// initData старше этого срока не принимаем.
const MAX_AGE_SECS: u64 = 24 * 60 * 60;

/// Пользователь Telegram, от имени которого пришёл запрос.
pub struct TgUser {
    pub id: String,
}

impl TgUser {
    /// Запрос разрешён только самому пользователю `user_id`.
    pub fn ensure_is(&self, user_id: &str) -> Result<(), ApiError> {
        if self.id == user_id { Ok(()) } else { Err(ApiError::Forbidden) }
    }

    /// Запрос разрешён только тренеру.
    pub async fn ensure_trainer(&self, state: &ApiState) -> Result<(), ApiError> {
        let user = state.user_service.get(self.id.clone()).await?;
        if user.is_trainer { Ok(()) } else { Err(ApiError::Forbidden) }
    }
}

impl FromRequestParts<Arc<ApiState>> for TgUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<ApiState>,
    ) -> Result<Self, Self::Rejection> {
        let header = |name: &str| parts.headers.get(name).and_then(|v| v.to_str().ok());

        if dev_auth_disabled() {
            if let Some(id) = header("x-dev-user-id") {
                return Ok(TgUser { id: id.to_string() });
            }
        }

        let init_data = header("authorization")
            .and_then(|v| v.strip_prefix("tma "))
            .ok_or(ApiError::Unauthorized)?;
        let id = validate_init_data(init_data, &state.bot_token).ok_or(ApiError::Unauthorized)?;
        Ok(TgUser { id })
    }
}

fn dev_auth_disabled() -> bool {
    std::env::var("API_AUTH_DISABLED").is_ok_and(|v| v == "true" || v == "1")
}

#[derive(Deserialize)]
struct InitDataUser {
    id: i64,
}

/// Проверяет подпись initData и возвращает id пользователя.
fn validate_init_data(init_data: &str, bot_token: &str) -> Option<String> {
    let mut fields: BTreeMap<String, String> = form_urlencoded::parse(init_data.as_bytes())
        .into_owned()
        .collect();
    let hash = hex::decode(fields.remove("hash")?).ok()?;

    // Все поля, кроме hash, отсортированные по ключу, в виде key=value через \n
    let data_check_string = fields
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("\n");

    let mut secret = Hmac::<Sha256>::new_from_slice(b"WebAppData").ok()?;
    secret.update(bot_token.as_bytes());
    let secret_key = secret.finalize().into_bytes();

    let mut mac = Hmac::<Sha256>::new_from_slice(&secret_key).ok()?;
    mac.update(data_check_string.as_bytes());
    mac.verify_slice(&hash).ok()?; // сравнение за постоянное время

    let auth_date: u64 = fields.get("auth_date")?.parse().ok()?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
    if now.saturating_sub(auth_date) > MAX_AGE_SECS {
        return None;
    }

    let user: InitDataUser = serde_json::from_str(fields.get("user")?).ok()?;
    Some(user.id.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Собирает initData так же, как это делает Telegram.
    fn sign(fields: &[(&str, &str)], bot_token: &str) -> String {
        let mut sorted: Vec<_> = fields.to_vec();
        sorted.sort();
        let dcs = sorted
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join("\n");
        let mut secret = Hmac::<Sha256>::new_from_slice(b"WebAppData").unwrap();
        secret.update(bot_token.as_bytes());
        let mut mac = Hmac::<Sha256>::new_from_slice(&secret.finalize().into_bytes()).unwrap();
        mac.update(dcs.as_bytes());
        let hash = hex::encode(mac.finalize().into_bytes());

        let mut s = form_urlencoded::Serializer::new(String::new());
        for (k, v) in fields {
            s.append_pair(k, v);
        }
        s.append_pair("hash", &hash);
        s.finish()
    }

    fn now() -> String {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs().to_string()
    }

    #[test]
    fn accepts_valid_init_data() {
        let now = now();
        let data = sign(
            &[("auth_date", &now), ("query_id", "AAH"), ("user", r#"{"id":42,"first_name":"Иван"}"#)],
            "123:token",
        );
        assert_eq!(validate_init_data(&data, "123:token"), Some("42".to_string()));
    }

    #[test]
    fn rejects_wrong_token() {
        let now = now();
        let data = sign(&[("auth_date", &now), ("user", r#"{"id":42}"#)], "123:token");
        assert_eq!(validate_init_data(&data, "999:other"), None);
    }

    #[test]
    fn rejects_tampered_user() {
        let now = now();
        let data = sign(&[("auth_date", &now), ("user", r#"{"id":42}"#)], "123:token");
        let tampered = data.replace("42", "43");
        assert_eq!(validate_init_data(&tampered, "123:token"), None);
    }

    #[test]
    fn rejects_expired() {
        let data = sign(&[("auth_date", "1000"), ("user", r#"{"id":42}"#)], "123:token");
        assert_eq!(validate_init_data(&data, "123:token"), None);
    }
}
