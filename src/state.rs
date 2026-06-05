use std::env::var;
use sqlx::postgres::PgPool;

pub type AppOidcClient = openidconnect::core::CoreClient<
  openidconnect::EndpointSet,
  openidconnect::EndpointNotSet,
  openidconnect::EndpointNotSet,
  openidconnect::EndpointSet,
  openidconnect::EndpointMaybeSet,
  openidconnect::EndpointMaybeSet,
>;

pub struct State {
  pub db: PgPool,
  pub oidc_client: AppOidcClient,
  pub http_client: reqwest::Client,

  // Only relevant if accepting POST/PUT
  pub max_content_len: usize,
}

pub async fn init_state() -> &'static State {
  dotenvy::dotenv().expect("Failed to read .env file into environment");

  // Get needed data from environment
  let max_content_len = var("MAX_CONTENT_LEN")
    .expect("MAX_CONTENT_LEN must be present in environment or .env file")
    .parse::<usize>()
    .expect("MAX_CONTENT_LEN could not be parsed as an unsigned integer")
  ;
  let db_url = var("DATABASE_URL")
    .expect("DATABASE_URL must be present in environment or .env file")
  ;
  let oidc_client_id = var("OIDC_CLIENT_ID")
    .expect("OIDC_CLIENT_ID must be present in environment or .env file")
  ;
  let oidc_client_secret = var("OIDC_CLIENT_SECRET")
    .expect("OIDC_CLIENT_SECRET must be present in environment or .env file")
  ;
  let mut oidc_redirect_url = var("OIDC_REDIRECT_URI")
    .expect("OIDC_REDIRECT_URI must be present in environment or .env file")
  ;
  oidc_redirect_url.push_str("/post-login");
  let admin_oidc_subject = var("ADMIN_OIDC_SUBJECT")
    .expect("ADMIN_OIDC_SUBJECT must be present in environment or .env file")
  ;

  // Construct requisite objects
  let db = sqlx::postgres::PgPoolOptions::new()
    .max_connections(8)
    .min_connections(1)
    .max_lifetime(std::time::Duration::from_secs(24 * 60 * 60))
    .connect(&db_url)
    .await
    .expect("Failed to connect to database")
  ;
  let http_client = reqwest::Client::new();
  let oidc_metadata = openidconnect::core::CoreProviderMetadata::discover_async(
    openidconnect::IssuerUrl::new(
      "https://accounts.google.com".to_string()
    ).unwrap(),
    &http_client,
  )
    .await
    .expect("Failed to get oidc metadata from google")
  ;
  let revocation_url = openidconnect::RevocationUrl::new(
    "https://oauth2.googleapis.com/revoke".to_string()
  )
    .expect("Invalid revocation URL")
  ;
  let oidc_client = openidconnect::core::CoreClient::from_provider_metadata(
    oidc_metadata,
    openidconnect::ClientId::new(oidc_client_id),
    Some(openidconnect::ClientSecret::new(oidc_client_secret)),
  )
    .set_redirect_uri(
      openidconnect::RedirectUrl::new(oidc_redirect_url)
        .expect("Invalid OIDC_REDIRECT_URL")
    )
    .set_revocation_url(revocation_url)
  ;

  // Perform any setup operations
  sqlx::migrate!()
    .run(&db)
    .await
    .expect("Failed to run database migrations. Usually caused by an already applied migration having changed in the source code")
  ;
  sqlx::query!(
    "
INSERT INTO app_user(user_id, oidc_subject, name, email, role) VALUES(0, $1, '', '', 'storyteller')
  ON CONFLICT (user_id)
  DO UPDATE SET oidc_subject = $1, name = CASE WHEN app_user.name = '' THEN '' ELSE app_user.name END, email = CASE WHEN app_user.email = '' THEN '' ELSE app_user.email END, role = 'storyteller' WHERE app_user.user_id = 0
    ",
    admin_oidc_subject,
  )
    .execute(&db)
    .await
    .expect("Failed to create admin account.")
  ;

  // Construct and return pointer to eternal instance
  Box::leak(Box::new(State{
    db,
    oidc_client,
    http_client,
    max_content_len,
  }))
}
