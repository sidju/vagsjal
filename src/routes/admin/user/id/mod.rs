use super::*;
use serde::Deserialize;

#[derive(sqlx::FromRow, Debug)]
struct UserRow {
  user_id: i64,
  name: String,
  oidc_subject: String,
  role: String,
}

#[derive(Deserialize)]
struct UserForm {
  name: String,
  oidc_subject: String,
  role: String,
}

#[derive(Template)]
#[template(path = "admin/user/id/index.html")]
struct Index {
  user: UserRow,
  show_admin_link: bool,
  oldest_active: Option<OldestActiveCharacter>,
}

async fn get_user(state: &'static State, user_id: i64) -> Result<UserRow, Error> {
  sqlx::query_as!(
    UserRow,
    r#"
    SELECT user_id, name, oidc_subject, role
    FROM app_user
    WHERE user_id = $1
    "#,
    user_id,
  )
    .fetch_one(&state.db)
    .await
    .map_err(Error::from)
}

async fn index_get(state: &'static State, session: &SessionData, user_id: i64) -> Result<Response, Error> {
  let user = get_user(state, user_id).await?;
  let oldest_active = fetch_oldest_active(state, session.user_id).await?;

  html(Index {
    user,
    show_admin_link: true,
    oldest_active,
  }.render()?)
}

async fn index_post(
  state: &'static State,
  mut req: Request,
  user_id: i64,
) -> Result<Response, Error> {
  let form: UserForm = parse_body_urlencoded(&mut req, state.max_content_len).await?;
  let role = match form.role.as_str() {
    "user" => "user",
    "storyteller" => "storyteller",
    _ => return Err(Error::invalid_builder_draft("Invalid role")),
  };
  sqlx::query!(
    r#"
    UPDATE app_user
    SET name = $2, oidc_subject = $3, role = $4
    WHERE user_id = $1
    "#,
    user_id,
    form.name,
    form.oidc_subject,
    role,
  )
    .execute(&state.db)
    .await?;
  see_other(&format!("/admin/user/{user_id}/"))
}

pub async fn route(
  state: &'static State,
  session: SessionData,
  req: Request,
  mut path_vec: Vec<String>,
  user_id: i64,
) -> Result<Response, Error> {
  match path_vec.pop().as_deref() {
    None => permanent_redirect(&format!("{}/", req.uri().path())),
    Some("") => match req.method() {
      &Method::GET => index_get(state, &session, user_id).await,
      &Method::POST => index_post(state, req, user_id).await,
      _ => Err(Error::method_not_found(&req)),
    },
    _ => Err(Error::path_not_found(&req)),
  }
}
