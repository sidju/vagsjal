use hyper::header::HeaderValue;
use hyper::{Method, StatusCode};
use askama::Template;

use crate::{
  State,
  Error,
  ClientError,
  Request,
  Response,
};

// A utils file for common operations while routing
mod utils;
use utils::*;
mod auth;
use auth::*;

#[derive(sqlx::Type, Debug, PartialEq, Clone)]
#[sqlx(rename_all = "lowercase", type_name = "VARCHAR")]
pub enum CharacterStatus {
  Draft,
  Active,
  Inactive,
}
impl CharacterStatus {
  pub fn is_active(&self) -> bool {
    matches!(self, CharacterStatus::Active)
  }
  pub fn is_draft(&self) -> bool {
    matches!(self, CharacterStatus::Draft)
  }
  pub fn is_inactive(&self) -> bool {
    matches!(self, CharacterStatus::Inactive)
  }
}

// And the actual route modules
mod admin;
mod character;
mod characters;
mod wiki;
use wiki::WikiPageTemplate;

const CSS: &'static str = include_str!("styles.css");
const LOGO: &'static [u8] = include_bytes!("vs-rr.png");
const FAVICON: &'static [u8] = include_bytes!("favicon.png");

#[derive(Template)]
#[template(path = "index.html")]
struct Index {
  show_admin_link: bool,
}

pub async fn route(
  state: &'static State,
  req: Request,
) -> Result<Response, Error> {
  // Put path into a list so we can match on it step by step
  let mut path_vec: Vec<String> = req
    .uri()
    .path()
    .split('/')
    .rev() // Reverse the iterator
    .map(|s| s.to_owned()) // Take ownership of the string, probably clones data
    .collect() // Aggregate into the variable
  ;
  // If the first path is something the uri is malformed
  // (such as http://localhost:8080wrong/path)
  match path_vec.pop().as_deref() {
    None | Some("") => (),
    Some(unexpected) => {
      return Err(Error::path_data_before_root(unexpected.to_owned()));
    },
  }

  // The actual routing
  let cookies = parse_cookies(&req)?;
  let auth = authenticate_from_cookies(state, &cookies).await?;
  let session = auth.session;
  let mut set_cookies = auth.set_cookies;
  let mut prevent_cache = true;
  let mut response = match path_vec.pop().as_deref() {
    // Means a missing trailing slash, redirect to with slash
    None => permanent_redirect(&format!("{}/", req.uri().path())),
    Some("") => {
      // verify that the path ends here and that the method is correct
      // utility function for simple paths
      verify_method_path_end(&path_vec, &req, &Method::GET)?;

      let show_admin = session.as_ref().map(|s| s.role.is_storyteller()).unwrap_or(false);

      // Render from homepage.md if available
      let body = match wiki::wiki_pages::get("homepage") {
        Some(page) => WikiPageTemplate {
          title: page.title.to_string(),
          content: page.content.to_string(),
          show_admin_link: show_admin,
        }.render()?,
        None => Index {
          show_admin_link: show_admin,
        }.render()?,
      };

      html(body)
    },
//    Some("login") => {
//      verify_method_path_end(&path_vec, &req, &Method::GET)?;
//      start_oidc_login_flow(state).await
//    },
    Some("post-login") => {
      verify_method_path_end(&path_vec, &req, &Method::GET)?;
      // Discard stale-cookie-clearing headers — finish_oidc_login_flow
      // sets its own fresh session/refresh cookies and we must not
      // append clearing headers after them.
      set_cookies.clear();
      add_header(
        finish_oidc_login_flow(state, req).await,
        hyper::header::CACHE_CONTROL,
        hyper::header::HeaderValue::from_static("no-store")
      )
    },
    Some("characters") => characters::route(state, session, req, path_vec).await,
    Some("character") => {
      match session {
        None => start_oidc_login_flow(state).await,
        Some(session) => character::route(state, session, req, path_vec).await,
      }
    },
    Some("admin") => {
      match session {
        None => start_oidc_login_flow(state).await,
        Some(session) => admin::route(state, session, req, path_vec).await,
      }
    },
    Some("wiki") => {
      prevent_cache = false;
      wiki::route(session, req, path_vec).await
    },
    Some("styles.css") => {
      prevent_cache = false;
      verify_method_path_end(&path_vec, &req, &Method::GET)?;
      css(CSS)
    },
    Some("vs-rr.png") => {
      prevent_cache = false;
      verify_method_path_end(&path_vec, &req, &Method::GET)?;
      add_header(png(LOGO), hyper::header::CACHE_CONTROL, HeaderValue::from_static("public, max-age=2592000, immutable"))
    },
    Some("favicon.png") => {
      prevent_cache = false;
      verify_method_path_end(&path_vec, &req, &Method::GET)?;
      add_header(png(FAVICON), hyper::header::CACHE_CONTROL, HeaderValue::from_static("public, max-age=2592000, immutable"))
    },
    _ => Err(Error::path_not_found(&req)),
  };
  for cookie in set_cookies {
    response = add_header(response, hyper::header::SET_COOKIE, cookie);
  }
  if prevent_cache {
    response = add_header(
      response,
      hyper::header::CACHE_CONTROL,
      HeaderValue::from_static("no-store"),
    );
  }
  response
}

pub(crate) async fn validate_description_url(http_client: &reqwest::Client, url: &str) -> Result<(), Error> {
  if !url.starts_with("https://") {
    return Err(Error::invalid_builder_draft("URL must use HTTPS"));
  }
  let parsed = reqwest::Url::parse(url)
    .map_err(|e| Error::invalid_builder_draft(&format!("Invalid URL: {e}")))?;
  if parsed.host_str().is_none() {
    return Err(Error::invalid_builder_draft("URL must have a host"));
  }
  let response = http_client
    .get(url)
    .timeout(std::time::Duration::from_secs(10))
    .send()
    .await
    .map_err(|e| Error::invalid_builder_draft(&format!("Could not reach URL: {e}")))?;
  if !response.status().is_success() {
    return Err(Error::invalid_builder_draft(&format!(
      "URL returned status {} — the document may not be publicly accessible",
      response.status().as_u16()
    )));
  }
  Ok(())
}
