use super::*;
use serde::Deserialize;
use time::Date;

#[derive(sqlx::FromRow, Debug)]
struct CharacterRow {
  vampire_id: i64,
  user_id: i64,
  active: bool,
  name: String,
  apparent_age: i32,
  date_embraced: Date,
  torpor_time: sqlx::postgres::types::PgInterval,
  clan_id: i64,
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

#[derive(Debug)]
struct PendingUsageRow {
  kind: String,
  usage_id: i64,
  name: String,
  increase: i32,
  xp_cost: i32,
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
  review_kind: Option<String>,
  usage_id: Option<i64>,
  decision: Option<String>,
}

#[derive(Template)]
#[template(path = "admin/characters/id/index.html")]
struct Index {
  character: CharacterRow,
  clans: Vec<ClanRow>,
  users: Vec<UserRow>,
  pending: Vec<PendingUsageRow>,
  show_admin_link: bool,
}

fn parse_date(date: &str) -> Result<Date, Error> {
  let fmt = time::format_description::parse("[year]-[month]-[day]")
    .map_err(|e| Error::invalid_builder_draft(&format!("Invalid date format: {e}")))?;
  Date::parse(date, &fmt).map_err(|e| Error::invalid_builder_draft(&format!("Invalid date: {e}")))
}

async fn get_character(state: &'static State, vampire_id: i64) -> Result<CharacterRow, Error> {
  sqlx::query_as!(
    CharacterRow,
    r#"
    SELECT vampire_id, user_id, active, name, apparent_age, date_embraced, torpor_time, clan_id
    FROM vampire
    WHERE vampire_id = $1
    "#,
    vampire_id,
  )
    .fetch_one(&state.db)
    .await
    .map_err(Error::from)
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

async fn fetch_pending_usage(state: &'static State, vampire_id: i64) -> Result<Vec<PendingUsageRow>, Error> {
  let stat_rows = sqlx::query!(
    r#"
    SELECT
      'stat' AS "kind!",
      stat_raise.stat_raise_id AS "usage_id!",
      stat_raise.stat AS "name!",
      stat_raise.increase AS "increase!",
      stat_raise.xp_cost AS "xp_cost!"
    FROM stat_raise
    LEFT JOIN stat_raise_review USING (stat_raise_id)
    WHERE stat_raise.vampire_id = $1 AND stat_raise_review.state IS NULL
    ORDER BY stat_raise.stat_raise_id
    "#,
    vampire_id,
  )
    .fetch_all(&state.db)
    .await?;
  let power_rows = sqlx::query!(
    r#"
    SELECT
      'power' AS "kind!",
      power_raise.power_raise_id AS "usage_id!",
      power_raise.power AS "name!",
      power_raise.increase AS "increase!",
      power_raise.xp_cost AS "xp_cost!"
    FROM power_raise
    LEFT JOIN power_raise_review USING (power_raise_id)
    WHERE power_raise.vampire_id = $1 AND power_raise_review.state IS NULL
    ORDER BY power_raise.power_raise_id
    "#,
    vampire_id,
  )
    .fetch_all(&state.db)
    .await?;
  let influence_rows = sqlx::query!(
    r#"
    SELECT
      'influence' AS "kind!",
      influence_raise.influence_raise_id AS "usage_id!",
      influence_raise.influence AS "name!",
      influence_raise.increase AS "increase!",
      influence_raise.xp_cost AS "xp_cost!"
    FROM influence_raise
    LEFT JOIN influence_raise_review USING (influence_raise_id)
    WHERE influence_raise.vampire_id = $1 AND influence_raise_review.state IS NULL
    ORDER BY influence_raise.influence_raise_id
    "#,
    vampire_id,
  )
    .fetch_all(&state.db)
    .await?;
  let humanity_rows = sqlx::query!(
    r#"
    SELECT
      'humanity' AS "kind!",
      humanity_change.humanity_change_id AS "usage_id!",
      humanity_change.note AS "name!",
      ABS(humanity_change.change)::INT AS "increase!",
      humanity_change.xp_cost AS "xp_cost!"
    FROM humanity_change
    LEFT JOIN humanity_change_review USING (humanity_change_id)
    WHERE humanity_change.vampire_id = $1 AND humanity_change_review.state IS NULL
    ORDER BY humanity_change.humanity_change_id
    "#,
    vampire_id,
  )
    .fetch_all(&state.db)
    .await?;
  Ok(stat_rows.into_iter()
    .map(|r| PendingUsageRow { kind: r.kind, usage_id: r.usage_id, name: r.name, increase: r.increase, xp_cost: r.xp_cost })
    .chain(power_rows.into_iter().map(|r| PendingUsageRow { kind: r.kind, usage_id: r.usage_id, name: r.name, increase: r.increase, xp_cost: r.xp_cost }))
    .chain(influence_rows.into_iter().map(|r| PendingUsageRow { kind: r.kind, usage_id: r.usage_id, name: r.name, increase: r.increase, xp_cost: r.xp_cost }))
    .chain(humanity_rows.into_iter().map(|r| PendingUsageRow { kind: r.kind, usage_id: r.usage_id, name: r.name, increase: r.increase, xp_cost: r.xp_cost }))
    .collect())
}

async fn index_get(state: &'static State, vampire_id: i64) -> Result<Response, Error> {
  let (character, (clans, users), pending) = tokio::try_join!(
    get_character(state, vampire_id),
    fetch_options(state),
    fetch_pending_usage(state, vampire_id),
  )?;
  html(Index {
    character,
    clans,
    users,
    pending,
    show_admin_link: true,
  }.render()?)
}

async fn update_character(
  state: &'static State,
  vampire_id: i64,
  form: CharacterForm,
) -> Result<Response, Error> {
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
    UPDATE vampire
    SET user_id = $2,
        active = $3,
        name = $4,
        apparent_age = $5,
        date_embraced = $6,
        torpor_time = $7,
        clan_id = $8
    WHERE vampire_id = $1
    "#,
    vampire_id,
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
  see_other(&format!("/admin/characters/{vampire_id}/"))
}

async fn review_pending(
  state: &'static State,
  session: &SessionData,
  vampire_id: i64,
  form: CharacterForm,
) -> Result<Response, Error> {
  let kind = form.review_kind.ok_or_else(|| Error::invalid_builder_draft("Missing review_kind"))?;
  let usage_id = form.usage_id.ok_or_else(|| Error::invalid_builder_draft("Missing usage_id"))?;
  let decision = form.decision.ok_or_else(|| Error::invalid_builder_draft("Missing decision"))?;
  let approved = match decision.as_str() {
    "approve" => "approved",
    "reject" => "denied",
    _ => return Err(Error::invalid_builder_draft("Invalid review decision")),
  };
  let rows_affected = match kind.as_str() {
    "stat" => sqlx::query!(
      r#"
      INSERT INTO stat_raise_review (stat_raise_id, state, reviewer_id)
      VALUES ($1, CASE WHEN $2 = 'approved' THEN 'approved' ELSE 'denied' END::approval_state_t, $3)
      ON CONFLICT DO NOTHING
      "#,
      usage_id,
      approved,
      session.user_id,
    )
      .execute(&state.db)
      .await?
      .rows_affected(),
    "power" => sqlx::query!(
      r#"
      INSERT INTO power_raise_review (power_raise_id, state, reviewer_id)
      VALUES ($1, CASE WHEN $2 = 'approved' THEN 'approved' ELSE 'denied' END::approval_state_t, $3)
      ON CONFLICT DO NOTHING
      "#,
      usage_id,
      approved,
      session.user_id,
    )
      .execute(&state.db)
      .await?
      .rows_affected(),
    "influence" => sqlx::query!(
      r#"
      INSERT INTO influence_raise_review (influence_raise_id, state, reviewer_id)
      VALUES ($1, CASE WHEN $2 = 'approved' THEN 'approved' ELSE 'denied' END::approval_state_t, $3)
      ON CONFLICT DO NOTHING
      "#,
      usage_id,
      approved,
      session.user_id,
    )
      .execute(&state.db)
      .await?
      .rows_affected(),
    "humanity" => sqlx::query!(
      r#"
      INSERT INTO humanity_change_review (humanity_change_id, state, reviewer_id)
      VALUES ($1, CASE WHEN $2 = 'approved' THEN 'approved' ELSE 'denied' END::approval_state_t, $3)
      ON CONFLICT DO NOTHING
      "#,
      usage_id,
      approved,
      session.user_id,
    )
      .execute(&state.db)
      .await?
      .rows_affected(),
    _ => return Err(Error::invalid_builder_draft("Invalid review kind")),
  };
  if rows_affected != 1 {
    return Err(Error::invalid_builder_draft("That XP usage has already been reviewed"));
  }
  see_other(&format!("/admin/characters/{vampire_id}/"))
}

async fn index_post(
  state: &'static State,
  session: SessionData,
  mut req: Request,
  vampire_id: i64,
) -> Result<Response, Error> {
  let form: CharacterForm = parse_body_urlencoded(&mut req, state.max_content_len).await?;
  match form.action.as_str() {
    "update" => update_character(state, vampire_id, form).await,
    "review" => review_pending(state, &session, vampire_id, form).await,
    _ => Err(Error::invalid_builder_draft("Unknown character action")),
  }
}

pub async fn route(
  state: &'static State,
  session: SessionData,
  req: Request,
  mut path_vec: Vec<String>,
  vampire_id: i64,
) -> Result<Response, Error> {
  match path_vec.pop().as_deref() {
    None => permanent_redirect(&format!("{}/", req.uri().path())),
    Some("") => match req.method() {
      &Method::GET => index_get(state, vampire_id).await,
      &Method::POST => index_post(state, session, req, vampire_id).await,
      _ => Err(Error::method_not_found(&req)),
    },
    _ => Err(Error::path_not_found(&req)),
  }
}
