use super::*;
use crate::routes::validate_description_url;
use time::Date;

mod edit;

#[derive(sqlx::FromRow, Debug)]
struct CharacterRow {
  vampire_id: i64,
  user_id: i64,
  owner_name: String,
  status: CharacterStatus,
  name: String,
  apparent_age: i32,
  date_embraced: Date,
  torpor_time: sqlx::postgres::types::PgInterval,
  clan_id: i64,
  clan_name: String,
  remaining_xp: i64,
  character_description_url: Option<String>,
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
  created_at: String,
}

#[derive(sqlx::FromRow, Debug)]
struct SummaryStat {
  name: String,
  value: i64,
  pending_review: bool,
}


#[derive(Template)]
#[template(path = "admin/character/id/index.html")]
struct Index {
  character: CharacterRow,
  stats: Vec<SummaryStat>,
  powers: Vec<SummaryStat>,
  influences: Vec<SummaryStat>,
  pending: Vec<PendingUsageRow>,
  show_admin_link: bool,
  oldest_active: Option<OldestActiveCharacter>,
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
    SELECT
      vampire.vampire_id,
      vampire.user_id,
      app_user.name AS owner_name,
      vampire.status AS "status: CharacterStatus",
      vampire.name,
      vampire.apparent_age,
      vampire.date_embraced,
      vampire.torpor_time,
      vampire.clan_id,
      clan.name AS clan_name,
      COALESCE(xp_remaining.amount, 0) AS "remaining_xp!",
      vampire.character_description_url AS "character_description_url?"
    FROM vampire
    JOIN app_user USING (user_id)
    JOIN clan USING (clan_id)
    LEFT JOIN xp_remaining USING (vampire_id)
    WHERE vampire_id = $1
    "#,
    vampire_id,
  )
    .fetch_one(&state.db)
    .await
    .map_err(Error::from)
}

async fn fetch_stats(
    state: &'static State,
    vampire_id: i64,
) -> Result<Vec<SummaryStat>, Error> {
    let stats = sqlx::query_as!(
      SummaryStat,
      r#"
      SELECT "name!" AS "name!", "value!" AS "value!", "pending_review!" AS "pending_review!"
      FROM vampire_stat
      WHERE vampire_id = $1
      ORDER BY "name!"
      "#,
      vampire_id,
    )
      .fetch_all(&state.db)
      .await?;
    Ok(stats)
}

async fn fetch_powers(
    state: &'static State,
    vampire_id: i64,
) -> Result<Vec<SummaryStat>, Error> {
    let powers = sqlx::query_as!(
      SummaryStat,
      r#"
      SELECT "name!" AS "name!", "value!" AS "value!", "pending_review!" AS "pending_review!"
      FROM vampire_power
      WHERE vampire_id = $1
      ORDER BY "name!"
      "#,
      vampire_id,
    )
      .fetch_all(&state.db)
      .await?;
    Ok(powers)
}

async fn fetch_influences(
    state: &'static State,
    vampire_id: i64,
) -> Result<Vec<SummaryStat>, Error> {
    let influences = sqlx::query_as!(
      SummaryStat,
      r#"
      SELECT "name!" AS "name!", "value!" AS "value!", "pending_review!" AS "pending_review!"
      FROM vampire_influence
      WHERE vampire_id = $1
      ORDER BY "name!"
      "#,
      vampire_id,
    )
      .fetch_all(&state.db)
      .await?;
    Ok(influences)
}

async fn fetch_pending_usage(state: &'static State, vampire_id: i64) -> Result<Vec<PendingUsageRow>, Error> {
  let stat_rows = sqlx::query!(
    r#"
    SELECT
      'stat' AS "kind!",
      stat_raise.stat_raise_id AS "usage_id!",
      stat_raise.stat AS "name!",
      stat_raise.increase AS "increase!",
      stat_raise.xp_cost AS "xp_cost!",
      to_char(stat_raise.creation_time, 'YYYY-MM-DD HH24:MI:SS') AS "created_at!"
    FROM stat_raise
    LEFT JOIN stat_raise_review USING (stat_raise_id)
    WHERE stat_raise.vampire_id = $1 AND stat_raise_review.state IS NULL
    ORDER BY stat_raise.creation_time, stat_raise.stat_raise_id
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
      power_raise.xp_cost AS "xp_cost!",
      to_char(power_raise.creation_time, 'YYYY-MM-DD HH24:MI:SS') AS "created_at!"
    FROM power_raise
    LEFT JOIN power_raise_review USING (power_raise_id)
    WHERE power_raise.vampire_id = $1 AND power_raise_review.state IS NULL
    ORDER BY power_raise.creation_time, power_raise.power_raise_id
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
      influence_raise.xp_cost AS "xp_cost!",
      to_char(influence_raise.creation_time, 'YYYY-MM-DD HH24:MI:SS') AS "created_at!"
    FROM influence_raise
    LEFT JOIN influence_raise_review USING (influence_raise_id)
    WHERE influence_raise.vampire_id = $1 AND influence_raise_review.state IS NULL
    ORDER BY influence_raise.creation_time, influence_raise.influence_raise_id
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
      humanity_change.xp_cost AS "xp_cost!",
      to_char(humanity_change.creation_time, 'YYYY-MM-DD HH24:MI:SS') AS "created_at!"
    FROM humanity_change
    LEFT JOIN humanity_change_review USING (humanity_change_id)
    WHERE humanity_change.vampire_id = $1 AND humanity_change_review.state IS NULL
    ORDER BY humanity_change.creation_time, humanity_change.humanity_change_id
    "#,
    vampire_id,
  )
    .fetch_all(&state.db)
    .await?;
  Ok(stat_rows.into_iter()
    .map(|r| PendingUsageRow { kind: r.kind, usage_id: r.usage_id, name: r.name, increase: r.increase, xp_cost: r.xp_cost, created_at: r.created_at })
    .chain(power_rows.into_iter().map(|r| PendingUsageRow { kind: r.kind, usage_id: r.usage_id, name: r.name, increase: r.increase, xp_cost: r.xp_cost, created_at: r.created_at }))
    .chain(influence_rows.into_iter().map(|r| PendingUsageRow { kind: r.kind, usage_id: r.usage_id, name: r.name, increase: r.increase, xp_cost: r.xp_cost, created_at: r.created_at }))
    .chain(humanity_rows.into_iter().map(|r| PendingUsageRow { kind: r.kind, usage_id: r.usage_id, name: r.name, increase: r.increase, xp_cost: r.xp_cost, created_at: r.created_at }))
    .collect())
}

async fn index_get(state: &'static State, session: &SessionData, vampire_id: i64) -> Result<Response, Error> {
  let (character, stats, powers, influences, pending) = tokio::try_join!(
    get_character(state, vampire_id),
    fetch_stats(state, vampire_id),
    fetch_powers(state, vampire_id),
    fetch_influences(state, vampire_id),
    fetch_pending_usage(state, vampire_id),
  )?;
  let oldest_active = fetch_oldest_active(state, session.user_id).await?;

  html(Index {
    character,
    stats,
    powers,
    influences,
    pending,
    show_admin_link: true,
    oldest_active,
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
    months: form.torpor_years.unwrap_or(0) * 12 + form.torpor_months.unwrap_or(0),
    days: form.torpor_days.unwrap_or(0),
    microseconds: 0,
  };
  let clan_id = form.clan_id.ok_or_else(|| Error::invalid_builder_draft("Missing clan_id"))?;
  let status = form.status.as_deref().unwrap_or("draft");
  let character_description_url = form.character_description_url.map(|u| u.trim().to_string()).filter(|u| !u.is_empty());
  if let Some(ref url) = character_description_url {
    validate_description_url(&state.http_client, url).await?;
  }
  sqlx::query!(
    r#"
    UPDATE vampire
    SET user_id = $2,
        status = $3,
        name = $4,
        apparent_age = $5,
        date_embraced = $6,
        torpor_time = $7,
        clan_id = $8,
        character_description_url = $9
    WHERE vampire_id = $1
    "#,
    vampire_id,
    user_id,
    status,
    name,
    apparent_age,
    date_embraced,
    torpor_time,
    clan_id,
    character_description_url,
  )
    .execute(&state.db)
    .await?;
  see_other(&format!("/admin/character/{vampire_id}/"))
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
  see_other(&format!("/admin/character/{vampire_id}/"))
}

async fn approve_draft(
  state: &'static State,
  _session: &SessionData,
  vampire_id: i64,
) -> Result<Response, Error> {
  sqlx::query!(
    "UPDATE vampire SET status = 'active' WHERE vampire_id = $1 AND status = 'draft'",
    vampire_id,
  )
    .execute(&state.db)
    .await?;
  see_other(&format!("/admin/character/{vampire_id}/"))
}

async fn index_post(
  state: &'static State,
  session: SessionData,
  mut req: Request,
  vampire_id: i64,
) -> Result<Response, Error> {
  let form: CharacterForm = parse_body_urlencoded(&mut req, state.max_content_len).await?;
  match form.action.as_deref().unwrap_or("review") {
    "update" => update_character(state, vampire_id, form).await,
    "review" => review_pending(state, &session, vampire_id, form).await,
    "approve" => approve_draft(state, &session, vampire_id).await,
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
      &Method::GET => index_get(state, &session, vampire_id).await,
      &Method::POST => index_post(state, session, req, vampire_id).await,
      _ => Err(Error::method_not_found(&req)),
    },
    Some("edit") => edit::route(state, session, req, path_vec, vampire_id).await,
    _ => Err(Error::path_not_found(&req)),
  }
}
