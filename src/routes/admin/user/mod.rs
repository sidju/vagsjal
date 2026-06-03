use super::*;

mod id;

#[derive(sqlx::FromRow, Debug)]
struct UserRow {
  user_id: i64,
  name: String,
  oidc_subject: String,
  role: String,
}

#[derive(Template)]
#[template(path = "admin/user/index.html")]
struct Index {
  users: Vec<UserRow>,
  show_admin_link: bool,
  oldest_active: Option<OldestActiveCharacter>,
}

async fn index(state: &'static State, session: &SessionData) -> Result<Response, Error> {
  let users = sqlx::query_as!(
    UserRow,
    r#"
    SELECT user_id, name, oidc_subject, role
    FROM app_user
    ORDER BY user_id
    "#
  )
    .fetch_all(&state.db)
    .await?;
  let oldest_active = fetch_oldest_active(state, session.user_id).await?;

  html(Index {
    users,
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
  match path_vec.pop().as_deref() {
    None => permanent_redirect(&format!("{}/", req.uri().path())),
    Some("") => {
      verify_method_path_end(&path_vec, &req, &Method::GET)?;
      index(state, &session).await
    },
    Some(id) => id::route(state, session, req, path_vec, id.parse()?).await,
  }
}
