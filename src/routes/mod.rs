use hyper::header::HeaderValue;
use hyper::{Method, StatusCode};
use askama::Template;
use time::Date;

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

// And the actual route modules
mod admin;
mod character;

const CSS: &'static str = include_str!("styles.css");

#[derive(Template)]
#[template(path = "index.html")]
struct Index{
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
  let mut response = match path_vec.pop().as_deref() {
    // Means a missing trailing slash, redirect to with slash
    None => permanent_redirect(&format!("{}/", req.uri().path())),
    Some("") => {
      // verify that the path ends here and that the method is correct
      // utility function for simple paths
      verify_method_path_end(&path_vec, &req, &Method::GET)?;

 //     let character_data = match session {
 //       // TODO, make use of the data if it exists
 //       Some(sess_data) => todo!(),
 //       None => None,
 //     };

      // Utility function to build html response around given str
      html(Index{
        show_admin_link: session.as_ref().map(|s| s.role.is_storyteller()).unwrap_or(false),
      }.render()?)
    },
//    Some("login") => {
//      verify_method_path_end(&path_vec, &req, &Method::GET)?;
//      start_oidc_login_flow(state).await
//    },
    Some("post-login") => {
      verify_method_path_end(&path_vec, &req, &Method::GET)?;
      add_header(
        finish_oidc_login_flow(state, req).await,
        hyper::header::CACHE_CONTROL,
        hyper::header::HeaderValue::from_static("no-store")
      )
    },
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
    Some("styles.css") => {
      verify_method_path_end(&path_vec, &req, &Method::GET)?;
      css(CSS)
    },
    _ => Err(Error::path_not_found(&req)),
  };
  for cookie in auth.set_cookies {
    response = add_header(response, hyper::header::SET_COOKIE, cookie);
  }
  response
}
