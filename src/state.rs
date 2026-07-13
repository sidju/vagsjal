use std::env::var;
use std::sync::Arc;
use arc_swap::ArcSwap;
use sqlx::postgres::PgPool;

pub type AppOidcClient = openidconnect::core::CoreClient<
  openidconnect::EndpointSet,
  openidconnect::EndpointNotSet,
  openidconnect::EndpointNotSet,
  openidconnect::EndpointSet,
  openidconnect::EndpointMaybeSet,
  openidconnect::EndpointMaybeSet,
>;

pub struct OidcState {
  client: ArcSwap<AppOidcClient>,
  issuer_url: String,
  client_id: String,
  client_secret: String,
  redirect_url: String,
  revocation_url: String,
}

impl OidcState {
  pub async fn new(
    http_client: &reqwest::Client,
    issuer_url: String,
    client_id: String,
    client_secret: String,
    redirect_url: String,
    revocation_url: String,
  ) -> Self {
    let metadata = openidconnect::core::CoreProviderMetadata::discover_async(
      openidconnect::IssuerUrl::new(issuer_url.clone()).unwrap(),
      http_client,
    )
      .await
      .expect("Failed to get oidc metadata from issuer")
    ;
    let client = Self::build_client(&metadata, &client_id, &client_secret, &redirect_url, &revocation_url);
    Self {
      client: ArcSwap::from(Arc::new(client)),
      issuer_url,
      client_id,
      client_secret,
      redirect_url,
      revocation_url,
    }
  }

  fn build_client(
    metadata: &openidconnect::core::CoreProviderMetadata,
    client_id: &str,
    client_secret: &str,
    redirect_url: &str,
    revocation_url: &str,
  ) -> AppOidcClient {
    openidconnect::core::CoreClient::from_provider_metadata(
      metadata.clone(),
      openidconnect::ClientId::new(client_id.to_owned()),
      Some(openidconnect::ClientSecret::new(client_secret.to_owned())),
    )
      .set_redirect_uri(
        openidconnect::RedirectUrl::new(redirect_url.to_owned())
          .expect("Invalid OIDC_REDIRECT_URL")
      )
      .set_revocation_url(
        openidconnect::RevocationUrl::new(revocation_url.to_owned())
          .expect("Invalid revocation URL")
      )
  }

  pub fn client(&self) -> arc_swap::Guard<Arc<AppOidcClient>> {
    self.client.load()
  }

  pub async fn refresh(&self, http_client: &reqwest::Client) {
    match openidconnect::core::CoreProviderMetadata::discover_async(
      openidconnect::IssuerUrl::new(self.issuer_url.clone()).unwrap(),
      http_client,
    ).await {
      Ok(metadata) => {
        let client = Self::build_client(
          &metadata,
          &self.client_id,
          &self.client_secret,
          &self.redirect_url,
          &self.revocation_url,
        );
        self.client.store(Arc::new(client));
        println!("OIDC signing keys refreshed");
      }
      Err(e) => {
        eprintln!("Failed to refresh OIDC signing keys: {e}");
      }
    }
  }
}

pub struct State {
  pub db: PgPool,
  pub oidc: OidcState,
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
  let oidc_issuer_url = var("OIDC_ISSUER_URL")
    .expect("OIDC_ISSUER_URL must be present in environment or .env file")
  ;
  let oidc_revocation_url = var("OIDC_REVOCATION_URL")
    .expect("OIDC_REVOCATION_URL must be present in environment or .env file")
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
  let oidc = OidcState::new(
    &http_client,
    oidc_issuer_url,
    oidc_client_id,
    oidc_client_secret,
    oidc_redirect_url,
    oidc_revocation_url,
  ).await;

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
    oidc,
    http_client,
    max_content_len,
  }))
}
