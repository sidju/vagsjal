use super::*;

#[derive(Template)]
#[template(path = "admin/character/id/edit/index.html")]
struct Index {
  character: CharacterRow,
  clans: Vec<ClanRow>,
  users: Vec<UserRow>,
  show_admin_link: bool,
}

async fn index_get(state: &'static State, vampire_id: i64) -> Result<Response, Error> {
  let (character, (clans, users)) = tokio::try_join!(
    get_character(state, vampire_id),
    fetch_options(state),
  )?;
  html(Index {
    character,
    clans,
    users,
    show_admin_link: true,
  }.render()?)
}

async fn fetch_options(state: &'static State) -> Result<(Vec<ClanRow>, Vec<UserRow>), Error> {
  let clans = sqlx::query_as!(
    ClanRow,
    r#"
    SELECT clan_id, name
    FROM clan
    ORDER BY name
    "#
  )
    .fetch_all(&state.db)
    .await?;
  let users = sqlx::query_as!(
    UserRow,
    r#"
    SELECT user_id, name
    FROM app_user
    ORDER BY user_id
    "#
  )
    .fetch_all(&state.db)
    .await?;
  Ok((clans, users))
}

async fn index_post(
  state: &'static State,
  vampire_id: i64,
  mut req: Request,
) -> Result<Response, Error> {
  let form: CharacterForm = parse_body_urlencoded(&mut req, state.max_content_len).await?;
  update_character(state, vampire_id, form).await
}

pub async fn route(
  state: &'static State,
  _session: SessionData,
  req: Request,
  mut path_vec: Vec<String>,
  vampire_id: i64,
) -> Result<Response, Error> {
  match path_vec.pop().as_deref() {
    None => permanent_redirect(&format!("{}/", req.uri().path())),
    Some("") => match req.method() {
      &Method::GET => index_get(state, vampire_id).await,
      &Method::POST => index_post(state, vampire_id, req).await,
      _ => Err(Error::method_not_found(&req)),
    },
    _ => Err(Error::path_not_found(&req)),
  }
}
