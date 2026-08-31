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

const CSS: &'static str = include_str!("../../assets/styles.css");
const SEARCH_JS: &str = include_str!("../../assets/search.js");
const SEARCH_DATA: &str = include_str!(concat!(env!("OUT_DIR"), "/search_data.js"));
const AWE_SIGN: &'static [u8] = include_bytes!("../../assets/awe-sign.jpg");
const BEAST_SIGN: &'static [u8] = include_bytes!("../../assets/beast-sign.jpg");
const CAMOFLAGE_SIGN: &'static [u8] = include_bytes!("../../assets/camoflage-sign.jpg");
const CARTHIAN_MOVEMENT: &'static [u8] = include_bytes!("../../assets/carthian-movement.webp");
const CIRCLE_OF_THE_CRONE: &'static [u8] = include_bytes!("../../assets/circle-of-the-crone.webp");
const DAEVA: &'static [u8] = include_bytes!("../../assets/daeva.webp");
const DOMINATE_SIGN: &'static [u8] = include_bytes!("../../assets/dominate-sign.jpg");
const DREAD_SIGN: &'static [u8] = include_bytes!("../../assets/dread-sign.jpg");
const FAVICON: &'static [u8] = include_bytes!("../../assets/favicon.png");
const GANGREL: &'static [u8] = include_bytes!("../../assets/gangrel.webp");
const INDEPENDENT: &'static [u8] = include_bytes!("../../assets/independent.webp");
const INVICTUS: &'static [u8] = include_bytes!("../../assets/invictus.webp");
const LANCEA_ET_SANCTUM: &'static [u8] = include_bytes!("../../assets/lancea-et-sanctum.webp");
const MEKHET: &'static [u8] = include_bytes!("../../assets/mekhet.webp");
const NOSFERATU: &'static [u8] = include_bytes!("../../assets/nosferatu.webp");
const OBFUSCATE_SIGN: &'static [u8] = include_bytes!("../../assets/obfuscate-sign.jpg");
const OFF_SIGN: &'static [u8] = include_bytes!("../../assets/off-sign.jpg");
const ORDO_DRACUL: &'static [u8] = include_bytes!("../../assets/ordo-dracul.png");
const VENTRUE: &'static [u8] = include_bytes!("../../assets/ventrue.webp");
const VS_RR: &'static [u8] = include_bytes!("../../assets/vs-rr.png");

enum RootAsset {
  Text {
    data: &'static str,
    content_type: &'static str,
  },
  Binary {
    data: &'static [u8],
    content_type: &'static str,
  },
}

fn find_root_asset(filename: &str) -> Option<RootAsset> {
  match filename {
    "search.js" => Some(RootAsset::Text {
      data: SEARCH_JS,
      content_type: "application/javascript; charset=utf-8",
    }),
    "search-data.js" => Some(RootAsset::Text {
      data: SEARCH_DATA,
      content_type: "application/javascript; charset=utf-8",
    }),
    "styles.css" => Some(RootAsset::Text {
      data: CSS,
      content_type: "text/css; charset=utf-8",
    }),
    "awe-sign.jpg" => Some(RootAsset::Binary {
      data: AWE_SIGN,
      content_type: "image/jpeg",
    }),
    "beast-sign.jpg" => Some(RootAsset::Binary {
      data: BEAST_SIGN,
      content_type: "image/jpeg",
    }),
    "camoflage-sign.jpg" => Some(RootAsset::Binary {
      data: CAMOFLAGE_SIGN,
      content_type: "image/jpeg",
    }),
    "carthian-movement.webp" => Some(RootAsset::Binary {
      data: CARTHIAN_MOVEMENT,
      content_type: "image/webp",
    }),
    "circle-of-the-crone.webp" => Some(RootAsset::Binary {
      data: CIRCLE_OF_THE_CRONE,
      content_type: "image/webp",
    }),
    "daeva.webp" => Some(RootAsset::Binary {
      data: DAEVA,
      content_type: "image/webp",
    }),
    "dominate-sign.jpg" => Some(RootAsset::Binary {
      data: DOMINATE_SIGN,
      content_type: "image/jpeg",
    }),
    "dread-sign.jpg" => Some(RootAsset::Binary {
      data: DREAD_SIGN,
      content_type: "image/jpeg",
    }),
    "favicon.png" => Some(RootAsset::Binary {
      data: FAVICON,
      content_type: "image/png",
    }),
    "gangrel.webp" => Some(RootAsset::Binary {
      data: GANGREL,
      content_type: "image/webp",
    }),
    "independent.webp" => Some(RootAsset::Binary {
      data: INDEPENDENT,
      content_type: "image/webp",
    }),
    "invictus.webp" => Some(RootAsset::Binary {
      data: INVICTUS,
      content_type: "image/webp",
    }),
    "lancea-et-sanctum.webp" => Some(RootAsset::Binary {
      data: LANCEA_ET_SANCTUM,
      content_type: "image/webp",
    }),
    "mekhet.webp" => Some(RootAsset::Binary {
      data: MEKHET,
      content_type: "image/webp",
    }),
    "nosferatu.webp" => Some(RootAsset::Binary {
      data: NOSFERATU,
      content_type: "image/webp",
    }),
    "obfuscate-sign.jpg" => Some(RootAsset::Binary {
      data: OBFUSCATE_SIGN,
      content_type: "image/jpeg",
    }),
    "off-sign.jpg" => Some(RootAsset::Binary {
      data: OFF_SIGN,
      content_type: "image/jpeg",
    }),
    "ordo-dracul.png" => Some(RootAsset::Binary {
      data: ORDO_DRACUL,
      content_type: "image/png",
    }),
    "ventrue.webp" => Some(RootAsset::Binary {
      data: VENTRUE,
      content_type: "image/webp",
    }),
    "vs-rr.png" => Some(RootAsset::Binary {
      data: VS_RR,
      content_type: "image/png",
    }),
    _ => None,
  }
}

fn serve_root_asset(asset: RootAsset) -> Result<Response, Error> {
  let mut response = match asset {
    RootAsset::Text { data, content_type } => {
      let mut re = Response::new(data.into());
      re.headers_mut().insert("Content-Type", HeaderValue::from_static(content_type));
      re
    },
    RootAsset::Binary { data, content_type } => {
      let mut re = Response::new(data.into());
      re.headers_mut().insert("Content-Type", HeaderValue::from_static(content_type));
      re
    },
  };
  response.headers_mut().insert(
    hyper::header::CACHE_CONTROL,
    HeaderValue::from_static("public, max-age=86400"),
  );
  Ok(response)
}

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
      add_header(wiki::route(session, req, path_vec).await, hyper::header::CACHE_CONTROL, HeaderValue::from_static("public, max-age=86400"))
    },
    Some(path) => {
      if let Some(asset) = find_root_asset(path) {
        prevent_cache = false;
        verify_method_path_end(&path_vec, &req, &Method::GET)?;
        serve_root_asset(asset)
      } else {
        Err(Error::path_not_found(&req))
      }
    },
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
