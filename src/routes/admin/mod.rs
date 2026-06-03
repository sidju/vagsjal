use super::*;

mod character;
mod user;

#[derive(Template)]
#[template(path = "admin/index.html")]
struct Index {
  show_admin_link: bool,
  oldest_active: Option<OldestActiveCharacter>,
}

async fn index(state: &'static State, session: &SessionData) -> Result<Response, Error> {
  let oldest_active = fetch_oldest_active(state, session.user_id).await?;

  html(Index {
    show_admin_link: true,
    oldest_active,
  }.render()?)
}

pub async fn route(
  state: &'static State,
  session: SessionData,
  req: Request,
  mut path_vec: Vec<String>,
) -> Result<Response, Error> {
  if !session.role.is_storyteller() {
    return Err(Error::forbidden());
  }
  match path_vec.pop().as_deref() {
    None => permanent_redirect(&format!("{}/", req.uri().path())),
    Some("") => {
      verify_method_path_end(&path_vec, &req, &Method::GET)?;
      index(state, &session).await
    },
    Some("user") => user::route(state, session, req, path_vec).await,
    Some("character") => character::route(state, session, req, path_vec).await,
    _ => Err(Error::path_not_found(&req)),
  }
}
