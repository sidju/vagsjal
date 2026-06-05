use super::*;

#[derive(sqlx::FromRow, Debug)]
struct PublicCharacter {
  name: String,
  apparent_age: i32,
  clan_name: String,
  covenant_name: String,
  covenant_slug: String,
  public_knowledge: String,
  home_domain: String,
  known_age: String,
}

#[derive(Template)]
#[template(path = "characters/index.html")]
struct Index {
  active_chars: Vec<PublicCharacter>,
  inactive_chars: Vec<PublicCharacter>,
  show_admin_link: bool,
}

async fn index(state: &'static State, session: Option<SessionData>) -> Result<Response, Error> {
  let rows = sqlx::query_as!(
    PublicCharacter,
    r#"
    SELECT
      vampire.name,
      vampire.apparent_age,
      clan.name AS clan_name,
      COALESCE(covenant.name, '') AS "covenant_name!",
      COALESCE(covenant.id, '') AS "covenant_slug!",
      vampire.public_knowledge,
      vampire.home_domain,
      vampire.known_age
    FROM vampire
    JOIN clan USING (clan_id)
    LEFT JOIN covenant USING (covenant_id)
    WHERE vampire.status = 'active'
    ORDER BY vampire.name
    "#
  )
    .fetch_all(&state.db)
    .await?
  ;
  let active_chars = rows;

  let rows = sqlx::query_as!(
    PublicCharacter,
    r#"
    SELECT
      vampire.name,
      vampire.apparent_age,
      clan.name AS clan_name,
      COALESCE(covenant.name, '') AS "covenant_name!",
      COALESCE(covenant.id, '') AS "covenant_slug!",
      vampire.public_knowledge,
      vampire.home_domain,
      vampire.known_age
    FROM vampire
    JOIN clan USING (clan_id)
    LEFT JOIN covenant USING (covenant_id)
    WHERE vampire.status = 'inactive'
    ORDER BY vampire.name
    "#
  )
    .fetch_all(&state.db)
    .await?
  ;
  let inactive_chars = rows;

  let show_admin_link = session.as_ref().map(|s| s.role.is_storyteller()).unwrap_or(false);

  html(Index {
    active_chars,
    inactive_chars,
    show_admin_link,
  }.render()?)
}

pub async fn route(
  state: &'static State,
  session: Option<SessionData>,
  req: Request,
  mut path_vec: Vec<String>,
) -> Result<Response, Error> {
  match path_vec.pop().as_deref() {
    None => permanent_redirect(&format!("{}/", req.uri().path())),
    Some("") => {
      verify_method_path_end(&path_vec, &req, &Method::GET)?;
      index(state, session).await
    },
    _ => Err(Error::path_not_found(&req)),
  }
}
