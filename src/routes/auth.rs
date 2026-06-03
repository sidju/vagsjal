use super::*;

use openidconnect::{ClaimsVerificationError, OAuth2TokenResponse};
use serde::{Deserialize, Serialize};

pub const SESSION_COOKIE_NAME: &str = "__Host-session_token";
pub const REFRESH_COOKIE_NAME: &str = "__Host-refresh_token";
const SESSION_COOKIE_MAX_AGE_SECONDS: i64 = 60 * 60;
const REFRESH_COOKIE_MAX_AGE_SECONDS: i64 = 60 * 60 * 24 * 30;

#[derive(Deserialize, Serialize)]
struct PostLoginQueryData {
  code: String,
  state: String,
}

#[derive(sqlx::Type, Debug)]
#[sqlx(rename_all = "lowercase")]
pub enum Role {
  User,
  Storyteller,
}
impl Role {
  pub fn is_storyteller(&self) -> bool {
    matches!(self, Role::Storyteller)
  }
}

#[derive(sqlx::FromRow, Debug)]
pub struct SessionData {
  pub user_id: i64,
  pub role: Role,
}

pub struct AuthResult {
  pub session: Option<SessionData>,
  pub set_cookies: Vec<HeaderValue>,
}

fn format_cookie(name: &str, value: &str, max_age_seconds: i64) -> Result<HeaderValue, Error> {
  HeaderValue::try_from(format!(
    "{name}={value}; Path=/; Max-Age={max_age_seconds}; Secure; HttpOnly; SameSite=Strict"
  )).map_err(Into::into)
}
fn format_clear_cookie(name: &str) -> Result<HeaderValue, Error> {
  HeaderValue::try_from(format!(
    "{name}=; Path=/; Max-Age=0; Secure; HttpOnly; SameSite=Strict"
  )).map_err(Into::into)
}
pub fn clear_auth_cookies() -> Result<Vec<HeaderValue>, Error> {
  Ok(vec![
    format_clear_cookie(SESSION_COOKIE_NAME)?,
    format_clear_cookie(REFRESH_COOKIE_NAME)?,
  ])
}

fn allow_any_nonce(_nonce: Option<&openidconnect::Nonce>) -> Result<(), String> {
  Ok(())
}
fn verify_id_token_allow_missing_nonce<'a>(
  state: &'static State,
  id_token: &'a openidconnect::core::CoreIdToken,
) -> Result<
  &'a openidconnect::IdTokenClaims<
    openidconnect::EmptyAdditionalClaims,
    openidconnect::core::CoreGenderClaim,
  >,
  ClaimsVerificationError,
> {
  id_token.claims(&state.oidc_client.id_token_verifier(), allow_any_nonce)
}
fn display_name_from_claims(
  claims: &openidconnect::IdTokenClaims<
    openidconnect::EmptyAdditionalClaims,
    openidconnect::core::CoreGenderClaim,
  >,
) -> String {
  claims
    .name()
    .and_then(|name| name.get(None))
    .map(|name| name.as_str().to_owned())
    .or_else(|| claims.preferred_username().map(|username| username.as_str().to_owned()))
    .or_else(|| claims.email().map(|email| email.as_str().to_owned()))
    .unwrap_or_else(|| claims.subject().as_str().to_owned())
}
async fn session_for_claims(
  state: &'static State,
  claims: &openidconnect::IdTokenClaims<
    openidconnect::EmptyAdditionalClaims,
    openidconnect::core::CoreGenderClaim,
  >,
) -> Result<SessionData, Error> {
  let display_name = display_name_from_claims(claims);
  let row = sqlx::query!(
    "
INSERT INTO app_user(oidc_subject, name, role)
VALUES ($1, $2, 'user')
ON CONFLICT (oidc_subject)
DO UPDATE SET name = CASE
  WHEN app_user.name = '' THEN EXCLUDED.name
  ELSE app_user.name
END
RETURNING user_id, role AS \"role!:Role\"
    ",
    claims.subject().as_str(),
    display_name,
  )
    .fetch_one(&state.db)
    .await
    .map_err(Error::from)?
  ;
  Ok(SessionData {
    user_id: row.user_id,
    role: row.role,
  })
}
async fn refresh_session_from_cookie(
  state: &'static State,
  refresh_token_raw: &str,
) -> Result<Option<(SessionData, Vec<HeaderValue>)>, Error> {
  let token_response: openidconnect::core::CoreTokenResponse = match state.oidc_client
    .exchange_refresh_token(&openidconnect::RefreshToken::new(refresh_token_raw.to_owned()))
  {
    Ok(req) => match req.request_async(&state.http_client).await {
      Ok(resp) => resp,
      Err(_) => return Ok(None),
    },
    Err(_) => return Ok(None),
  };
  let id_token = match token_response.extra_fields().id_token() {
    Some(token) => token,
    None => return Ok(None),
  };
  let claims = match verify_id_token_allow_missing_nonce(state, id_token) {
    Ok(claims) => claims,
    Err(_) => return Ok(None),
  };
  let session = session_for_claims(state, claims).await?;
  let refresh_token = token_response
    .refresh_token()
    .map(|t| t.secret().to_owned())
    .unwrap_or_else(|| refresh_token_raw.to_owned())
  ;
  Ok(Some((
    session,
    vec![
      format_cookie(
        SESSION_COOKIE_NAME,
        &id_token.to_string(),
        SESSION_COOKIE_MAX_AGE_SECONDS,
      )?,
      format_cookie(
        REFRESH_COOKIE_NAME,
        &refresh_token,
        REFRESH_COOKIE_MAX_AGE_SECONDS,
      )?,
    ],
  )))
}

pub async fn start_oidc_login_flow(
  state: &'static State,
) -> Result<Response, Error> {
  let (authorize_url, csrf_state, nonce) = state.oidc_client
    .authorize_url(
      openidconnect::AuthenticationFlow::<openidconnect::core::CoreResponseType>::AuthorizationCode,
      openidconnect::CsrfToken::new_random,
      openidconnect::Nonce::new_random,
    )
    .add_extra_param("access_type", "offline")
    .add_extra_param("prompt", "consent")
    .url()
  ;
  sqlx::query!(
    "INSERT INTO login_process(state_id, nonce) VALUES($1, $2)",
    csrf_state.secret(),
    nonce.secret(),
  )
    .execute(&state.db)
    .await
  ?;

  add_header(
    see_other(authorize_url.as_str()),
    hyper::header::CACHE_CONTROL,
    hyper::header::HeaderValue::from_static("no-store"),
  )
}

pub async fn authenticate_from_cookies(
  state: &'static State,
  cookies: &std::collections::HashMap<&str, &str>,
) -> Result<AuthResult, Error> {
  let session_cookie = match cookies.get(SESSION_COOKIE_NAME) {
    Some(cookie) => *cookie,
    None => {
      // Session cookie absent (most likely expired and purged by the browser).
      // Try the refresh cookie before falling back to the login flow.
      if let Some(refresh_cookie) = cookies.get(REFRESH_COOKIE_NAME) {
        if let Some((session, set_cookies)) = refresh_session_from_cookie(state, refresh_cookie).await? {
          return Ok(AuthResult { session: Some(session), set_cookies });
        }
        // Refresh present but failed — clear it so the browser doesn't keep sending it.
        return Ok(AuthResult { session: None, set_cookies: clear_auth_cookies()? });
      }
      return Ok(AuthResult { session: None, set_cookies: vec![] });
    },
  };
  let id_token: openidconnect::core::CoreIdToken = match session_cookie.parse() {
    Ok(token) => token,
    Err(_) => {
      return Ok(AuthResult {
        session: None,
        set_cookies: clear_auth_cookies()?,
      });
    },
  };
  let claims = match verify_id_token_allow_missing_nonce(state, &id_token) {
    Ok(claims) => claims,
    Err(ClaimsVerificationError::Expired(_)) => {
      if let Some(refresh_cookie) = cookies.get(REFRESH_COOKIE_NAME) {
        if let Some((session, set_cookies)) = refresh_session_from_cookie(state, refresh_cookie).await? {
          return Ok(AuthResult { session: Some(session), set_cookies });
        }
      }
      return Ok(AuthResult {
        session: None,
        set_cookies: clear_auth_cookies()?,
      });
    },
    Err(_) => {
      return Ok(AuthResult {
        session: None,
        set_cookies: clear_auth_cookies()?,
      });
    },
  };
  let session = session_for_claims(state, claims).await?;
  Ok(AuthResult {
    session: Some(session),
    set_cookies: vec![],
  })
}

pub async fn finish_oidc_login_flow(
  state: &'static State,
  req: Request,
) -> Result<Response, Error> {
  let oidc_response: PostLoginQueryData = parse_query(&req)?;
  let nonce: String = sqlx::query_scalar(
    "
DELETE FROM login_process
WHERE state_id = $1
  AND creation_time >= NOW() - INTERVAL '5 minutes'
RETURNING nonce
    "
  )
    .bind(oidc_response.state)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ClientError::UnknownOIDCProcess)?
  ;

  let token_response: openidconnect::core::CoreTokenResponse = state.oidc_client
    .exchange_code(openidconnect::AuthorizationCode::new(oidc_response.code))?
    .request_async(&state.http_client)
    .await
  ?;
  let id_token = token_response
    .extra_fields()
    .id_token()
    .ok_or(ClientError::OIDCGaveNoToken)?
  ;
  let id_token_claims = id_token.claims(
    &state.oidc_client.id_token_verifier(),
    &openidconnect::Nonce::new(nonce),
  )?;
  let _session = session_for_claims(state, id_token_claims).await?;
  let refresh_token = token_response
    .refresh_token()
    .ok_or(ClientError::OIDCGaveNoRefreshToken)?
    .secret()
  ;

  let res = add_header(
    redirect("/"),
    hyper::header::SET_COOKIE,
    format_cookie(
      SESSION_COOKIE_NAME,
      &id_token.to_string(),
      SESSION_COOKIE_MAX_AGE_SECONDS,
    )?,
  );
  let res = add_header(
    res,
    hyper::header::SET_COOKIE,
    format_cookie(
      REFRESH_COOKIE_NAME,
      refresh_token,
      REFRESH_COOKIE_MAX_AGE_SECONDS,
    )?,
  );
  add_header(
    res,
    hyper::header::CACHE_CONTROL,
    HeaderValue::from_static("no-store"),
  )
}
