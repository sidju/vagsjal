use super::*;
use crate::routes::validate_description_url;
use serde::Deserialize;
use time::Date;

mod id;

#[derive(sqlx::FromRow, Debug)]
struct CharacterRow {
  vampire_id: i64,
  user_id: i64,
  owner_name: String,
  status: CharacterStatus,
  name: String,
  clan_name: String,
  covenant_name: String,
  covenant_slug: String,
  remaining_xp: i64,
  character_description_url: Option<String>,
}

#[derive(sqlx::FromRow, Debug)]
struct ClanRow {
  clan_id: i64,
  name: String,
}

#[derive(sqlx::FromRow, Debug)]
struct CovenantRow {
  covenant_id: i64,
  name: String,
}

#[derive(sqlx::FromRow, Debug)]
struct UserRow {
  user_id: i64,
  name: String,
}

#[derive(Deserialize)]
struct CharacterForm {
  action: Option<String>,
  user_id: Option<i64>,
  name: Option<String>,
  apparent_age: Option<i32>,
  date_embraced: Option<String>,
  torpor_years: Option<String>,
  torpor_months: Option<String>,
  torpor_days: Option<String>,
  clan_id: Option<i64>,
  covenant_id: Option<String>,
  character_description_url: Option<String>,
  public_knowledge: Option<String>,
  home_domain: Option<String>,
  known_age: Option<String>,
  status: Option<String>,
  review_kind: Option<String>,
  usage_id: Option<i64>,
  decision: Option<String>,
  bp_change: Option<i32>,
  bp_note: Option<String>,
  humanity_change: Option<i32>,
  review_note: Option<String>,
}

#[derive(Template)]
#[template(path = "admin/character/index.html")]
struct Index {
  draft_chars: Vec<CharacterRow>,
  active_chars: Vec<CharacterRow>,
  inactive_chars: Vec<CharacterRow>,
  clans: Vec<ClanRow>,
  covenants: Vec<CovenantRow>,
  users: Vec<UserRow>,
  show_admin_link: bool,
}

fn parse_date(date: &str) -> Result<Date, Error> {
  let fmt = time::format_description::parse("[year]-[month]-[day]")
    .map_err(|e| Error::invalid_builder_draft(&format!("Invalid date format: {e}")))?;
  Date::parse(date, &fmt).map_err(|e| Error::invalid_builder_draft(&format!("Invalid date: {e}")))
}

async fn fetch_index_data(state: &'static State) -> Result<(Vec<CharacterRow>, Vec<ClanRow>, Vec<CovenantRow>, Vec<UserRow>), Error> {
  let characters = sqlx::query_as!(
    CharacterRow,
    r#"
    SELECT
      vampire.vampire_id,
      vampire.user_id,
      app_user.name AS owner_name,
      vampire.status AS "status: CharacterStatus",
      vampire.name,
      clan.name AS clan_name,
      COALESCE(covenant.name, '') AS "covenant_name!",
      COALESCE(covenant.id, '') AS "covenant_slug!",
      COALESCE(xp_remaining.amount, 0) AS "remaining_xp!",
      vampire.character_description_url AS "character_description_url?"
    FROM vampire
    JOIN app_user USING (user_id)
    JOIN clan USING (clan_id)
    LEFT JOIN covenant USING (covenant_id)
    LEFT JOIN xp_remaining USING (vampire_id)
    ORDER BY
      CASE vampire.status
        WHEN 'draft' THEN 0
        WHEN 'active' THEN 1
        WHEN 'inactive' THEN 2
      END,
      vampire.name
    "#
  )
    .fetch_all(&state.db)
    .await?;
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
  let covenants = sqlx::query_as!(
    CovenantRow,
    r#"
    SELECT covenant_id, name
    FROM covenant
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
  Ok((characters, clans, covenants, users))
}

async fn index_get(state: &'static State) -> Result<Response, Error> {
  let (characters, clans, covenants, users) = fetch_index_data(state).await?;
  let mut draft_chars = Vec::new();
  let mut active_chars = Vec::new();
  let mut inactive_chars = Vec::new();
  for c in characters {
    match c.status {
      CharacterStatus::Draft => draft_chars.push(c),
      CharacterStatus::Active => active_chars.push(c),
      CharacterStatus::Inactive => inactive_chars.push(c),
    }
  }

  html(Index {
    draft_chars,
    active_chars,
    inactive_chars,
    clans,
    covenants,
    users,
    show_admin_link: true,
  }.render()?)
}

async fn index_post(state: &'static State, mut req: Request) -> Result<Response, Error> {
  let form: CharacterForm = parse_body_urlencoded(&mut req, state.max_content_len).await?;
  match form.action.as_deref().unwrap_or("") {
    "create" => {
      let user_id = form.user_id.ok_or_else(|| Error::invalid_builder_draft("Missing user_id"))?;
      let name = form.name.ok_or_else(|| Error::invalid_builder_draft("Missing name"))?;
      let apparent_age = form.apparent_age.ok_or_else(|| Error::invalid_builder_draft("Missing apparent_age"))?;
      let date_embraced = parse_date(form.date_embraced.as_deref().ok_or_else(|| Error::invalid_builder_draft("Missing date_embraced"))?)?;
      let parse_torpor = |s: Option<String>| -> i32 { s.and_then(|s| { let s = s.trim().to_string(); if s.is_empty() { None } else { s.parse().ok() } }).unwrap_or(0) };
      let torpor_time = sqlx::postgres::types::PgInterval {
        months: parse_torpor(form.torpor_years) * 12 + parse_torpor(form.torpor_months),
        days: parse_torpor(form.torpor_days),
        microseconds: 0,
      };
      let clan_id = form.clan_id.ok_or_else(|| Error::invalid_builder_draft("Missing clan_id"))?;
      let status = form.status.as_deref().unwrap_or("draft");
      let character_description_url = form.character_description_url.map(|u| u.trim().to_string()).filter(|u| !u.is_empty());
      if let Some(ref url) = character_description_url {
        validate_description_url(&state.http_client, url).await?;
      }
  let covenant_id: Option<i64> = form.covenant_id.and_then(|s| { let s = s.trim(); if s.is_empty() { None } else { s.parse().ok() } });
  let public_knowledge = form.public_knowledge.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).unwrap_or_default();
  let home_domain = form.home_domain.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).unwrap_or_default();
  let known_age = form.known_age.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).unwrap_or_default();
  sqlx::query!(
    r#"
    INSERT INTO vampire (user_id, status, name, apparent_age, date_embraced, torpor_time, clan_id, covenant_id, character_description_url, public_knowledge, home_domain, known_age)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
    "#,
    user_id,
    status,
    name,
    apparent_age,
    date_embraced,
    torpor_time,
    clan_id,
    covenant_id,
    character_description_url,
    public_knowledge,
    home_domain,
    known_age,
      )
        .execute(&state.db)
        .await
        .map_err(|e| {
          if let sqlx::Error::Database(ref dbe) = e {
            if let Some(code) = dbe.code() {
              if code == "23514" {
                return Error::invalid_builder_draft("Ogiltig karaktär. Omfamningsdatumet kan inte vara i framtiden, torpor-tid kan inte vara negativ och mänsklig ålder måste vara positiv.");
              }
            }
          }
          Error::from(e)
        })?;
      see_other("/admin/character/")
    },
    _ => Err(Error::invalid_builder_draft("Unknown character action")),
  }
}

pub async fn route(
  state: &'static State,
  session: SessionData,
  req: Request,
  mut path_vec: Vec<String>,
) -> Result<Response, Error> {
  match path_vec.pop().as_deref() {
    None => permanent_redirect(&format!("{}/", req.uri().path())),
    Some("") => match req.method() {
      &Method::GET => index_get(state).await,
      &Method::POST => index_post(state, req).await,
      _ => Err(Error::method_not_found(&req)),
    },
    Some(id) => id::route(state, session, req, path_vec, id.parse()?).await,
  }
}
