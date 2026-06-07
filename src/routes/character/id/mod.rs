use super::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug)]
struct CharacterHeader {
  name: String,
  status: CharacterStatus,
  remaining_xp: i64,
  character_description_url: Option<String>,
  owner_name: String,
  apparent_age: i32,
  date_embraced: String,
  clan_name: String,
  covenant_name: String,
  covenant_slug: String,
  torpor_time: sqlx::postgres::types::PgInterval,
  public_knowledge: String,
  home_domain: String,
  known_age: String,
}

impl CharacterHeader {
  fn torpor_display(&self) -> String {
    fmt_torpor(&self.torpor_time)
  }
}
#[derive(Debug, Serialize)]
struct StatLine {
  id: String,
  name: String,
  value: i64,
  pending_review: bool,
}
/// Combined row for a power: current state plus the in-clan flag for XP pricing.
struct PowerRow {
  id: String,
  name: String,
  value: i64,
  pending_review: bool,
  in_clan: bool,
}
/// Combined row for an influence: current state plus its XP cost per level.
struct InfluenceRow {
  id: String,
  name: String,
  value: i64,
  pending_review: bool,
  xp_cost: Option<i32>,
}
/// Stat option with optional XP cost (options without a cost row are display-only).
#[derive(Debug, Serialize)]
struct StatOptionRow {
  id: String,
  name: String,
  xp_cost: Option<i32>,
}
#[derive(Debug, Clone, Serialize)]
struct PowerOption {
  id: String,
  name: String,
  in_clan: bool,
}
#[derive(Debug, Clone, Serialize)]
struct InfluenceOption {
  id: String,
  name: String,
}
#[derive(Debug, Serialize)]
struct XpCosts {
  stat: HashMap<String, i32>,
  influence: HashMap<String, i32>,
  /// level → xp_cost for in-clan powers
  power_in_clan: HashMap<i32, i32>,
  /// level → xp_cost for out-of-clan powers
  power_out_clan: HashMap<i32, i32>,
  humanity_gain: i32,
  humanity_loss: i32,
}
#[derive(Debug, Serialize)]
struct DraftOptions {
  stats: Vec<StatOptionRow>,
  powers: Vec<PowerOption>,
  influences: Vec<InfluenceOption>,
}
#[derive(Debug, Deserialize)]
struct SaveForm {
  ops_json: String,
}
#[derive(Debug, Deserialize)]
struct SavedQuery {
  saved: Option<i32>,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum DraftOperation {
  Stat { stat: String, increase: i32 },
  Power { power: String, increase: i32 },
  Influence { influence: String, increase: i32 },
  Humanity { change: i32, note: String },
}
#[derive(Debug, Serialize)]
struct InitialData<'a> {
  user_id: i64,
  vampire_id: i64,
  remaining_xp: i64,
  stats: &'a [StatLine],
  powers: &'a [StatLine],
  influences: &'a [StatLine],
  xp_costs: XpCosts,
  options: DraftOptions,
}
#[derive(Debug, Template)]
#[template(path = "character/id/index.html")]
struct Index {
  character: CharacterHeader,
  stats: Vec<StatLine>,
  powers: Vec<StatLine>,
  influences: Vec<StatLine>,
  /// Safe JSON embedded via <script type="application/json"> (</> escaped)
  initial_data_json: String,
  saved: bool,
  show_admin_link: bool,
}
#[derive(Debug, Template)]
#[template(path = "character/id/draft.html")]
struct DraftView {
  character: CharacterHeader,
  show_admin_link: bool,
}
#[derive(Debug, Template)]
#[template(path = "character/id/inactive.html")]
struct InactiveView {
  character: CharacterHeader,
  show_admin_link: bool,
}

async fn get_character(
  state: &'static State,
  req: &Request,
  user_id: i64,
  vampire_id: i64,
) -> Result<CharacterHeader, Error> {
  sqlx::query_as!(
    CharacterHeader,
    r#"
SELECT
  vampire.name,
  vampire.status AS "status: CharacterStatus",
  COALESCE(xp_remaining.amount, 0) AS "remaining_xp!",
  vampire.character_description_url AS "character_description_url?",
  app_user.name AS "owner_name!",
  vampire.apparent_age,
  to_char(vampire.date_embraced, 'YYYY-MM-DD') AS "date_embraced!",
  clan.name AS "clan_name!",
  COALESCE(covenant.name, '') AS "covenant_name!",
  COALESCE(covenant.id, '') AS "covenant_slug!",
  vampire.torpor_time,
  vampire.public_knowledge,
  vampire.home_domain,
  vampire.known_age
FROM vampire
JOIN app_user USING (user_id)
JOIN clan USING (clan_id)
LEFT JOIN covenant USING (covenant_id)
LEFT JOIN xp_remaining USING (vampire_id)
WHERE vampire.vampire_id = $1
  AND vampire.user_id = $2
    "#,
    vampire_id,
    user_id,
  )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| Error::path_not_found(req))
}
/// Fetches computed stats from the view (includes Blood Potency/HP formula rows).
async fn fetch_stats(
  state: &'static State,
  vampire_id: i64,
) -> Result<Vec<StatLine>, Error> {
  let rows = sqlx::query!(
    r#"
SELECT
  "id!" AS "id!",
  "name!" AS "name!",
  "value!" AS "value!: i64",
  "pending_review!" AS "pending_review!"
FROM vampire_stat
WHERE vampire_id = $1
ORDER BY CASE "id!" WHEN 'humanity' THEN 0 WHEN 'blood-potency' THEN 1 WHEN 'hp' THEN 2 WHEN 'physical-ability' THEN 3 WHEN 'mental-ability' THEN 4 WHEN 'organizational-ability' THEN 5 END
    "#,
    vampire_id,
  )
    .fetch_all(&state.db)
    .await?;
  Ok(rows.into_iter().map(|r| StatLine {
    id: r.id,
    name: r.name,
    value: r.value,
    pending_review: r.pending_review,
  }).collect())
}
/// Fetches all powers with current state (if any raises exist) and the in-clan flag
/// for this vampire, joining all three tables in one round-trip.
async fn fetch_powers_combined(
  state: &'static State,
  vampire_id: i64,
) -> Result<Vec<PowerRow>, Error> {
  let rows = sqlx::query!(
    r#"
SELECT
  power.id AS "id!",
  power.name AS "name!",
  COALESCE(vp."value!", 0) AS "value!: i64",
  COALESCE(vp."pending_review!", false) AS "pending_review!",
  EXISTS(
    SELECT 1
    FROM vampire
    JOIN clan USING (clan_id)
    WHERE vampire.vampire_id = $1
      AND (power.id = clan.unique_power
        OR power.id = clan.power_one
        OR power.id = clan.power_two)
  ) AS "in_clan!"
FROM power
LEFT JOIN vampire_power vp ON vp."id!" = power.id AND vp.vampire_id = $1
ORDER BY power.name
    "#,
    vampire_id,
  )
    .fetch_all(&state.db)
    .await?;
  Ok(rows.into_iter().map(|r| PowerRow {
    id: r.id,
    name: r.name,
    value: r.value,
    pending_review: r.pending_review,
    in_clan: r.in_clan,
  }).collect())
}
/// Fetches all influences with current state (if any raises exist) and xp cost per
/// level, joining all three tables in one round-trip.
async fn fetch_influences_combined(
  state: &'static State,
  vampire_id: i64,
) -> Result<Vec<InfluenceRow>, Error> {
  let rows = sqlx::query!(
    r#"
SELECT
  influence.id AS "id!",
  influence.name AS "name!",
  COALESCE(vi."value!", 0) AS "value!: i64",
  COALESCE(vi."pending_review!", false) AS "pending_review!",
  ixc.xp_cost AS "xp_cost?"
FROM influence
LEFT JOIN vampire_influence vi ON vi."id!" = influence.id AND vi.vampire_id = $1
LEFT JOIN influence_xp_cost ixc ON ixc.influence = influence.id
ORDER BY influence.name
    "#,
    vampire_id,
  )
    .fetch_all(&state.db)
    .await?;
  Ok(rows.into_iter().map(|r| InfluenceRow {
    id: r.id,
    name: r.name,
    value: r.value,
    pending_review: r.pending_review,
    xp_cost: r.xp_cost,
  }).collect())
}
/// Fetches all stat names with their XP costs in a single query.
async fn fetch_stat_options_and_costs(
  state: &'static State,
) -> Result<Vec<StatOptionRow>, Error> {
  let rows = sqlx::query!(
    "
SELECT stat.id, stat.name, sxc.xp_cost AS \"xp_cost?\"
FROM stat
LEFT JOIN stat_xp_cost sxc ON sxc.stat = stat.id
ORDER BY stat.id
    ",
  )
    .fetch_all(&state.db)
    .await?;
  Ok(rows.into_iter().map(|r| StatOptionRow {
    id: r.id,
    name: r.name,
    xp_cost: r.xp_cost,
  }).collect())
}
async fn fetch_power_xp_costs(
  state: &'static State,
) -> Result<(HashMap<i32, i32>, HashMap<i32, i32>), Error> {
  let rows = sqlx::query!(
    "SELECT in_clan, level, xp_cost FROM power_xp_cost"
  )
    .fetch_all(&state.db)
    .await?;
  let mut in_clan: HashMap<i32, i32> = HashMap::new();
  let mut out_clan: HashMap<i32, i32> = HashMap::new();
  for r in rows {
    if r.in_clan {
      in_clan.insert(r.level, r.xp_cost);
    } else {
      out_clan.insert(r.level, r.xp_cost);
    }
  }
  Ok((in_clan, out_clan))
}
async fn fetch_humanity_xp_costs(
  state: &'static State,
) -> Result<(i32, i32), Error> {
  let rows = sqlx::query!(
    "SELECT change_type, xp_cost FROM humanity_xp_cost"
  )
    .fetch_all(&state.db)
    .await?;
  let mut gain = 0i32;
  let mut loss = 0i32;
  for r in rows {
    match r.change_type.as_str() {
      "gain" => gain = r.xp_cost,
      "loss" => loss = r.xp_cost,
      _ => {},
    }
  }
  Ok((gain, loss))
}
async fn index_get(
  state: &'static State,
  req: Request,
  session: SessionData,
  character: CharacterHeader,
  vampire_id: i64,
) -> Result<Response, Error> {
  if character.status.is_draft() {
    return html(DraftView {
      character,
      show_admin_link: session.role.is_storyteller(),
    }.render()?);
  }
  if character.status.is_inactive() {
    return html(InactiveView {
      character,
      show_admin_link: session.role.is_storyteller(),
    }.render()?);
  }

  let query: SavedQuery = parse_query(&req)?;
  let (
    stats,
    power_rows,
    influence_rows,
    stat_option_rows,
  ) = tokio::try_join!(
    fetch_stats(state, vampire_id),
    fetch_powers_combined(state, vampire_id),
    fetch_influences_combined(state, vampire_id),
    fetch_stat_options_and_costs(state),
  )?;
  let (power_xp_in_clan, power_xp_out_clan) = fetch_power_xp_costs(state).await?;
  let (humanity_gain, humanity_loss) = fetch_humanity_xp_costs(state).await?;

  let powers: Vec<StatLine> = power_rows.iter()
    .map(|r| StatLine { id: r.id.clone(), name: r.name.clone(), value: r.value, pending_review: r.pending_review })
    .collect();
  let power_options: Vec<PowerOption> = power_rows.into_iter()
    .map(|r| PowerOption { id: r.id.clone(), name: r.name, in_clan: r.in_clan })
    .collect();
  let influences: Vec<StatLine> = influence_rows.iter()
    .map(|r| StatLine { id: r.id.clone(), name: r.name.clone(), value: r.value, pending_review: r.pending_review })
    .collect();
  let influence_options: Vec<InfluenceOption> = influence_rows.iter().map(|r| InfluenceOption { id: r.id.clone(), name: r.name.clone() }).collect();
  let influence_xp_costs: HashMap<String, i32> = influence_rows.into_iter()
    .filter_map(|r| r.xp_cost.map(|c| (r.id, c)))
    .collect();
  let stat_options: Vec<StatOptionRow> = stat_option_rows.iter().map(|r| StatOptionRow { id: r.id.clone(), name: r.name.clone(), xp_cost: r.xp_cost }).collect();
  let stat_xp_costs: HashMap<String, i32> = stat_option_rows.into_iter()
    .filter_map(|r| r.xp_cost.map(|c| (r.id, c)))
    .collect();

  let initial_data_json = serde_json::to_string(&InitialData {
    user_id: session.user_id,
    vampire_id,
    remaining_xp: character.remaining_xp,
    stats: &stats,
    powers: &powers,
    influences: &influences,
    xp_costs: XpCosts {
      stat: stat_xp_costs,
      influence: influence_xp_costs,
      power_in_clan: power_xp_in_clan,
      power_out_clan: power_xp_out_clan,
      humanity_gain,
      humanity_loss,
    },
    options: DraftOptions {
      stats: stat_options,
      powers: power_options.clone(),
      influences: influence_options.clone(),
    },
  })?.replace("</", r"<\/");

  html(Index {
    character,
    stats,
    powers,
    influences,
    initial_data_json,
    saved: query.saved == Some(1),
    show_admin_link: session.role.is_storyteller(),
  }.render()?)
}
async fn index_post(
  state: &'static State,
  mut req: Request,
  _session: SessionData,
  character: CharacterHeader,
  vampire_id: i64,
) -> Result<Response, Error> {
  if !character.status.is_active() {
    return Err(Error::character_not_active());
  }
  let save_form: SaveForm = parse_body_urlencoded(&mut req, state.max_content_len).await?;
  let operations: Vec<DraftOperation> = serde_json::from_str(&save_form.ops_json)
    .map_err(|e| Error::invalid_builder_draft(&format!("Invalid draft operations JSON: {e}")))?;

  let mut tx = state.db.begin().await?;
  for op in operations {
    match op {
      DraftOperation::Stat { stat, increase } => {
        let result = sqlx::query!(
          "
INSERT INTO stat_raise(vampire_id, stat, increase, xp_cost)
SELECT $1, $2::VARCHAR, $3::INT, stat_xp_cost.xp_cost * $3::INT
FROM stat_xp_cost
WHERE stat_xp_cost.stat = $2::VARCHAR
          ",
          vampire_id,
          stat,
          increase,
        )
          .execute(&mut *tx)
          .await?
        ;
        if result.rows_affected() != 1 {
          return Err(Error::invalid_builder_draft(&format!("Missing XP rule for stat '{stat}'")));
        }
      },
      DraftOperation::Power { power, increase } => {
        let result = sqlx::query!(
          "
INSERT INTO power_raise(vampire_id, power, increase, xp_cost)
SELECT $1, $2::VARCHAR, $3::INT, SUM(power_xp_cost.xp_cost)::INT
FROM power_xp_cost
JOIN generate_series(1, $3::INT) AS g(lev)
  ON power_xp_cost.level = COALESCE(
    (SELECT \"value!\"::INT FROM vampire_power WHERE vampire_id = $1 AND \"name!\" = $2::VARCHAR),
    0
  ) + g.lev
WHERE power_xp_cost.in_clan = (
    SELECT EXISTS(
      SELECT 1
      FROM vampire
      JOIN clan USING (clan_id)
      WHERE vampire.vampire_id = $1
        AND ($2::VARCHAR = clan.unique_power OR $2::VARCHAR = clan.power_one OR $2::VARCHAR = clan.power_two)
    )
  )
          ",
          vampire_id,
          power,
          increase,
        )
          .execute(&mut *tx)
          .await
          .map_err(|e| {
            if let sqlx::Error::Database(ref dbe) = e {
              if let Some(code) = dbe.code() {
                if code == "23514" {
                  return Error::invalid_builder_draft(
                    "Det finns redan en pågående höjning för denna kraft som väntar på granskning",
                  );
                }
              }
            }
            Error::from(e)
          })?
        ;
        if result.rows_affected() != 1 {
          return Err(Error::invalid_builder_draft(&format!("Missing XP rule for power '{power}'")));
        }
      },
      DraftOperation::Influence { influence, increase } => {
        let result = sqlx::query!(
          "
INSERT INTO influence_raise(vampire_id, influence, increase, xp_cost)
SELECT $1, $2::VARCHAR, $3::INT, influence_xp_cost.xp_cost * $3::INT
FROM influence_xp_cost
WHERE influence_xp_cost.influence = $2::VARCHAR
          ",
          vampire_id,
          influence,
          increase,
        )
          .execute(&mut *tx)
          .await?
        ;
        if result.rows_affected() != 1 {
          return Err(Error::invalid_builder_draft(&format!("Missing XP rule for influence '{influence}'")));
        }
      },
      DraftOperation::Humanity { change, note } => {
        let result = sqlx::query!(
          "
INSERT INTO humanity_change(vampire_id, change, xp_cost, note)
SELECT
  $1,
  $2::INT,
  humanity_xp_cost.xp_cost * ABS($2::INT),
  $3::VARCHAR
FROM humanity_xp_cost
WHERE humanity_xp_cost.change_type = CASE WHEN $2::INT > 0 THEN 'gain' ELSE 'loss' END
          ",
          vampire_id,
          change,
          note,
        )
          .execute(&mut *tx)
          .await?
        ;
        if result.rows_affected() != 1 {
          return Err(Error::invalid_builder_draft("Missing XP rule for humanity change"));
        }
      },
    }
  }
  tx.commit().await?;
  see_other(&format!("/character/{vampire_id}/?saved=1"))
}
pub async fn route(
  state: &'static State,
  req: Request,
  mut path_vec: Vec<String>,
  session: SessionData,
  vampire_id: i64,
) -> Result<Response, Error> {
  let character = get_character(state, &req, session.user_id, vampire_id).await?;
  match path_vec.pop().as_deref() {
    None => permanent_redirect(&format!("{}/", req.uri().path())),
    Some("") => {
      verify_path_end(&path_vec, &req)?;
      match req.method() {
        &Method::GET => index_get(state, req, session, character, vampire_id).await,
        &Method::POST => index_post(state, req, session, character, vampire_id).await,
        _ => Err(Error::method_not_found(&req)),
      }
    },
    _ => Err(Error::path_not_found(&req)),
  }
}
