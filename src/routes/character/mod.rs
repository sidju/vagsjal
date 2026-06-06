use super::*;
use serde::Deserialize;
use time::Date;

// Opening a specific character id is for using XP (and storytellers can approve XP use)
mod id;

#[derive(Debug)]
struct Character {
  vampire_id: i64,
  status: CharacterStatus,
  name: String,
  apparent_age: i32, // in years
  date_embraced: Date,
  torpor_time: sqlx::postgres::types::PgInterval,
  clan_name: String,
  covenant_name: String,
  covenant_slug: String,
  remaining_xp: i64,
  character_description_url: Option<String>,
  public_knowledge: String,
  home_domain: String,
  known_age: String,
}

impl Character {
  fn torpor_display(&self) -> String {
    fmt_torpor(&self.torpor_time)
  }
}

fn fmt_torpor(t: &sqlx::postgres::types::PgInterval) -> String {
  let years = t.months / 12;
  let months = t.months % 12;
  let days = t.days;
  let mut parts = Vec::new();
  if years > 0 { parts.push(format!("{years} år")); }
  if months > 0 { parts.push(format!("{months} månader")); }
  if days > 0 { parts.push(format!("{days} dagar")); }
  if parts.is_empty() { "0 dagar".into() } else { parts.join(", ") }
}
#[derive(Debug)]
struct Stat {
  id: String,
  name: String,
  value: i64,
  pending_review: bool,
}
#[derive(Debug)]
struct ClanRow {
  clan_id: i64,
  name: String,
}
#[derive(Debug)]
struct CovenantRow {
  covenant_id: i64,
  name: String,
}
#[derive(Deserialize)]
struct CreateCharacterForm {
  name: Option<String>,
  apparent_age: Option<i32>,
  date_embraced: Option<String>,
  torpor_years: Option<i32>,
  torpor_months: Option<i32>,
  torpor_days: Option<i32>,
  clan_id: Option<i64>,
  covenant_id: Option<String>,
  character_description_url: Option<String>,
  public_knowledge: Option<String>,
  home_domain: Option<String>,
  known_age: Option<String>,
}
#[derive(Template)]
#[template(path = "character/index.html")]
struct Index {
  active_chars: Vec<(
    Character,
    Vec<Stat>, Vec<Stat>, Vec<Stat>
  )>,
  draft_chars: Vec<(
    Character,
    Vec<Stat>, Vec<Stat>, Vec<Stat>
  )>,
  inactive_chars: Vec<(
    Character,
    Vec<Stat>, Vec<Stat>, Vec<Stat>
  )>,
  clans: Vec<ClanRow>,
  covenants: Vec<CovenantRow>,
  show_form: bool,
  saved: bool,
  show_admin_link: bool,
}

impl Index {
  fn xp_link(&self) -> bool {
    true
  }
}

fn parse_date(date: &str) -> Result<Date, Error> {
  let fmt = time::format_description::parse("[year]-[month]-[day]")
    .map_err(|e| Error::invalid_builder_draft(&format!("Invalid date format: {e}")))?;
  Date::parse(date, &fmt).map_err(|e| Error::invalid_builder_draft(&format!("Invalid date: {e}")))
}

async fn fetch_clans(state: &'static State) -> Result<Vec<ClanRow>, Error> {
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
  Ok(clans)
}

async fn fetch_covenants(state: &'static State) -> Result<Vec<CovenantRow>, Error> {
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
  Ok(covenants)
}

async fn index_get(
  state: &'static State,
  session: SessionData,
  saved: bool,
) -> Result<Response, Error> {
  let characters = sqlx::query_as!(Character,
    "
SELECT
  vampire.vampire_id,
  vampire.status AS \"status: CharacterStatus\",
  vampire.name,
  vampire.apparent_age,
  vampire.date_embraced,
  vampire.torpor_time,
      clan.name AS clan_name,
      COALESCE(covenant.name, '') AS \"covenant_name!\",
      COALESCE(covenant.id, '') AS \"covenant_slug!\",
      COALESCE(xp_remaining.amount, 0) AS \"remaining_xp!\",
      vampire.character_description_url AS \"character_description_url?\",
      vampire.public_knowledge,
      vampire.home_domain,
      vampire.known_age
FROM vampire
JOIN clan USING (clan_id)
LEFT JOIN covenant USING (covenant_id)
LEFT JOIN xp_remaining USING (vampire_id)
WHERE vampire.user_id = $1
ORDER BY vampire.name
    ",
    session.user_id,
  )
    .fetch_all(&state.db)
    .await?
  ;
  // Fetch the stats for each character
  let mut character_stats = Vec::new();
  for c in characters {
    let stats = sqlx::query!(
      r#"
SELECT "id!" AS "id!", "name!" AS "name!", "value!" AS "value!: i64", "pending_review!" AS "pending_review!"
FROM vampire_stat
WHERE vampire_id = $1
      "#,
      c.vampire_id
    )
      .fetch_all(&state.db)
      .await?
      .into_iter()
      .map(|r| Stat {
        id: r.id,
        name: r.name,
        value: r.value,
        pending_review: r.pending_review,
      })
      .collect()
    ;
    let powers = sqlx::query!(
      r#"
SELECT "id!" AS "id!", "name!" AS "name!", "value!" AS "value!: i64", "pending_review!" AS "pending_review!"
FROM vampire_power
WHERE vampire_id = $1
      "#,
      c.vampire_id
    )
      .fetch_all(&state.db)
      .await?
      .into_iter()
      .map(|r| Stat {
        id: r.id,
        name: r.name,
        value: r.value,
        pending_review: r.pending_review,
      })
      .collect()
    ;
    let influences = sqlx::query!(
      r#"
SELECT "id!" AS "id!", "name!" AS "name!", "value!" AS "value!: i64", "pending_review!" AS "pending_review!"
FROM vampire_influence
WHERE vampire_id = $1
      "#,
      c.vampire_id
    )
      .fetch_all(&state.db)
      .await?
      .into_iter()
      .map(|r| Stat {
        id: r.id,
        name: r.name,
        value: r.value,
        pending_review: r.pending_review,
      })
      .collect()
    ;
    character_stats.push((c,stats,powers,influences));
  }

  // Group by status
  let mut active_chars = Vec::new();
  let mut draft_chars = Vec::new();
  let mut inactive_chars = Vec::new();
  for entry in character_stats {
    if entry.0.status.is_active() {
      active_chars.push(entry);
    } else if entry.0.status.is_draft() {
      draft_chars.push(entry);
    } else {
      inactive_chars.push(entry);
    }
  }

  let (clans, covenants) = tokio::join!(
    fetch_clans(state),
    fetch_covenants(state),
  );
  let show_form = active_chars.is_empty() && draft_chars.is_empty();

  // Render and return
  html(Index{
    active_chars,
    draft_chars,
    inactive_chars,
    clans: clans?,
    covenants: covenants?,
    show_form,
    saved,
    show_admin_link: session.role.is_storyteller(),
  }.render()?)
}

async fn index_post(
  state: &'static State,
  session: &SessionData,
  mut req: Request,
) -> Result<Response, Error> {
  let form: CreateCharacterForm = parse_body_urlencoded(&mut req, state.max_content_len).await?;
  let character_description_url = form.character_description_url.map(|u| u.trim().to_string()).filter(|u| !u.is_empty());
  if let Some(ref url) = character_description_url {
    validate_description_url(&state.http_client, url).await?;
  }
  let name = form.name.ok_or_else(|| Error::invalid_builder_draft("Missing name"))?;
  let apparent_age = form.apparent_age.ok_or_else(|| Error::invalid_builder_draft("Missing apparent_age"))?;
  let date_embraced = parse_date(form.date_embraced.as_deref().ok_or_else(|| Error::invalid_builder_draft("Missing date_embraced"))?)?;
  let torpor_time = sqlx::postgres::types::PgInterval {
    months: form.torpor_years.unwrap_or(0) * 12 + form.torpor_months.unwrap_or(0),
    days: form.torpor_days.unwrap_or(0),
    microseconds: 0,
  };
  let clan_id = form.clan_id.ok_or_else(|| Error::invalid_builder_draft("Missing clan_id"))?;
  let covenant_id: Option<i64> = form.covenant_id.and_then(|s| { let s = s.trim(); if s.is_empty() { None } else { s.parse().ok() } });
  let public_knowledge = form.public_knowledge.map(|u| u.trim().to_string()).filter(|u| !u.is_empty()).unwrap_or_default();
  let home_domain = form.home_domain.map(|u| u.trim().to_string()).filter(|u| !u.is_empty()).unwrap_or_default();
  let known_age = form.known_age.map(|u| u.trim().to_string()).filter(|u| !u.is_empty()).unwrap_or_default();
  sqlx::query!(
    r#"
    INSERT INTO vampire (user_id, status, name, apparent_age, date_embraced, torpor_time, clan_id, covenant_id, character_description_url, public_knowledge, home_domain, known_age)
    VALUES ($1, 'draft', $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
    "#,
    session.user_id,
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
    .await?;
  see_other("/character/?saved=1")
}

pub async fn route(
  state: &'static State,
  session: SessionData,
  req: Request,
  mut path_vec: Vec<String>,
) -> Result<Response, Error> {
  match path_vec.pop().as_deref() {
    // Means a missing trailing slash, redirect to with slash
    None => permanent_redirect(&format!("{}/", req.uri().path())),
    Some("") => {
      verify_path_end(&path_vec, &req)?;
      match req.method() {
        &Method::GET => {
          let query: std::collections::HashMap<String, String> = parse_query(&req)?;
          let saved = query.get("saved").map(|s| s == "1").unwrap_or(false);
          index_get(state, session, saved).await
        },
        &Method::POST => index_post(state, &session, req).await,
        _ => Err(Error::method_not_found(&req)),
      }
    },
    // Parse the path into an integer id and keep routing
    Some(id) => id::route(state, req, path_vec, session, id.parse()?).await,
  }
}
