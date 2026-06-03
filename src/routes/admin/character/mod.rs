use super::*;
use serde::Deserialize;
use time::Date;

mod id;

#[derive(sqlx::FromRow, Debug)]
struct CharacterRow {
  vampire_id: i64,
  user_id: i64,
  owner_name: String,
  active: bool,
  name: String,
  apparent_age: i32,
  date_embraced: Date,
  torpor_time: sqlx::postgres::types::PgInterval,
  clan_name: String,
  remaining_xp: i64,
}

#[derive(sqlx::FromRow, Debug)]
struct ClanRow {
  clan_id: i64,
  name: String,
}

#[derive(sqlx::FromRow, Debug)]
struct UserRow {
  user_id: i64,
  name: String,
}

#[derive(Deserialize)]
struct CharacterForm {
  action: String,
  user_id: Option<i64>,
  name: Option<String>,
  apparent_age: Option<i32>,
  date_embraced: Option<String>,
  torpor_months: Option<i32>,
  torpor_days: Option<i32>,
  clan_id: Option<i64>,
  active: Option<String>,
}

#[derive(Template)]
#[template(path = "admin/character/index.html")]
struct Index {
  characters: Vec<CharacterRow>,
  clans: Vec<ClanRow>,
  users: Vec<UserRow>,
  show_admin_link: bool,
}

fn parse_date(date: &str) -> Result<Date, Error> {
  let fmt = time::format_description::parse("[year]-[month]-[day]")
    .map_err(|e| Error::invalid_builder_draft(&format!("Invalid date format: {e}")))?;
  Date::parse(date, &fmt).map_err(|e| Error::invalid_builder_draft(&format!("Invalid date: {e}")))
}

async fn fetch_index_data(state: &'static State) -> Result<(Vec<CharacterRow>, Vec<ClanRow>, Vec<UserRow>), Error> {
  let characters = sqlx::query_as!(
    CharacterRow,
    r#"
    SELECT
      vampire.vampire_id,
      vampire.user_id,
      app_user.name AS owner_name,
      vampire.active,
      vampire.name,
      vampire.apparent_age,
      vampire.date_embraced,
      vampire.torpor_time,
      clan.name AS clan_name,
      COALESCE(xp_remaining.amount, 0) AS "remaining_xp!"
    FROM vampire
    JOIN app_user USING (user_id)
    JOIN clan USING (clan_id)
    LEFT JOIN xp_remaining USING (vampire_id)
    ORDER BY vampire.active DESC, vampire.name
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
  Ok((characters, clans, users))
}

async fn index_get(state: &'static State) -> Result<Response, Error> {
  let (characters, clans, users) = fetch_index_data(state).await?;
  html(Index {
    characters,
    clans,
    users,
    show_admin_link: true,
  }.render()?)
}

async fn index_post(state: &'static State, mut req: Request) -> Result<Response, Error> {
  let form: CharacterForm = parse_body_urlencoded(&mut req, state.max_content_len).await?;
  match form.action.as_str() {
    "create" => {
      let user_id = form.user_id.ok_or_else(|| Error::invalid_builder_draft("Missing user_id"))?;
      let name = form.name.ok_or_else(|| Error::invalid_builder_draft("Missing name"))?;
      let apparent_age = form.apparent_age.ok_or_else(|| Error::invalid_builder_draft("Missing apparent_age"))?;
      let date_embraced = parse_date(form.date_embraced.as_deref().ok_or_else(|| Error::invalid_builder_draft("Missing date_embraced"))?)?;
      let torpor_time = sqlx::postgres::types::PgInterval {
        months: form.torpor_months.unwrap_or(0),
        days: form.torpor_days.unwrap_or(0),
        microseconds: 0,
      };
      let clan_id = form.clan_id.ok_or_else(|| Error::invalid_builder_draft("Missing clan_id"))?;
      let active = form.active.is_some();
      sqlx::query!(
        r#"
        INSERT INTO vampire (user_id, active, name, apparent_age, date_embraced, torpor_time, clan_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        user_id,
        active,
        name,
        apparent_age,
        date_embraced,
        torpor_time,
        clan_id,
      )
        .execute(&state.db)
        .await?;
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
