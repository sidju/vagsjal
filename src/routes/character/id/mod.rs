use super::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug)]
struct CharacterHeader {
  name: String,
  status: CharacterStatus,
  remaining_xp: i64,
  character_description_url: Option<String>,
}
#[derive(Debug, Serialize)]
struct StatLine {
  name: String,
  value: i64,
  pending_review: bool,
}
/// Combined row for a power: current state plus the in-clan flag for XP pricing.
struct PowerRow {
  name: String,
  value: i64,
  pending_review: bool,
  in_clan: bool,
}
/// Combined row for an influence: current state plus its XP cost per level.
struct InfluenceRow {
  name: String,
  value: i64,
  pending_review: bool,
  xp_cost: Option<i32>,
}
/// Stat option with optional XP cost (options without a cost row are display-only).
struct StatOptionRow {
  name: String,
  xp_cost: Option<i32>,
}
#[derive(Debug, Serialize)]
struct PowerOption {
  name: String,
  in_clan: bool,
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
  stats: Vec<String>,
  powers: Vec<PowerOption>,
  influences: Vec<String>,
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
  Power { power: String },
  Influence { influence: String },
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
  oldest_active: Option<OldestActiveCharacter>,
}
#[derive(Debug, Template)]
#[template(path = "character/id/draft.html")]
struct DraftView {
  character: CharacterHeader,
  show_admin_link: bool,
  oldest_active: Option<OldestActiveCharacter>,
}
#[derive(Debug, Template)]
#[template(path = "character/id/inactive.html")]
struct InactiveView {
  character: CharacterHeader,
  show_admin_link: bool,
  oldest_active: Option<OldestActiveCharacter>,
}

async fn get_character(
  state: &'static State,
  req: &Request,
  user_id: i64,
  vampire_id: i64,
) -> Result<CharacterHeader, Error> {
  sqlx::query_as!(
    CharacterHeader,
    "
SELECT vampire.name, vampire.status AS \"status: CharacterStatus\", COALESCE(xp_remaining.amount, 0) AS \"remaining_xp!\", vampire.character_description_url AS \"character_description_url?\"
FROM vampire
LEFT JOIN xp_remaining USING (vampire_id)
WHERE vampire.vampire_id = $1
  AND vampire.user_id = $2
    ",
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
  sqlx::query_as!(
    StatLine,
    "
SELECT
  \"name!\" AS \"name!\",
  \"value!\" AS \"value!\",
  \"pending_review!\" AS \"pending_review!\"
FROM vampire_stat
WHERE vampire_id = $1
ORDER BY \"name!\"
    ",
    vampire_id,
  )
    .fetch_all(&state.db)
    .await
    .map_err(Error::from)
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
  power.name AS "name!",
  COALESCE(vp."value!", 0) AS "value!: i64",
  COALESCE(vp."pending_review!", false) AS "pending_review!",
  EXISTS(
    SELECT 1
    FROM vampire
    JOIN clan USING (clan_id)
    WHERE vampire.vampire_id = $1
      AND (power.name = clan.unique_power
        OR power.name = clan.power_one
        OR power.name = clan.power_two)
  ) AS "in_clan!"
FROM power
LEFT JOIN vampire_power vp ON vp."name!" = power.name AND vp.vampire_id = $1
ORDER BY power.name
    "#,
    vampire_id,
  )
    .fetch_all(&state.db)
    .await?;
  Ok(rows.into_iter().map(|r| PowerRow {
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
  influence.name AS "name!",
  COALESCE(vi."value!", 0) AS "value!: i64",
  COALESCE(vi."pending_review!", false) AS "pending_review!",
  ixc.xp_cost AS "xp_cost?"
FROM influence
LEFT JOIN vampire_influence vi ON vi."name!" = influence.name AND vi.vampire_id = $1
LEFT JOIN influence_xp_cost ixc ON ixc.influence = influence.name
ORDER BY influence.name
    "#,
    vampire_id,
  )
    .fetch_all(&state.db)
    .await?;
  Ok(rows.into_iter().map(|r| InfluenceRow {
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
SELECT stat.name, sxc.xp_cost AS \"xp_cost?\"
FROM stat
LEFT JOIN stat_xp_cost sxc ON sxc.stat = stat.name
ORDER BY stat.name
    ",
  )
    .fetch_all(&state.db)
    .await?;
  Ok(rows.into_iter().map(|r| StatOptionRow {
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
  let oldest_active = fetch_oldest_active(state, session.user_id).await?;

  if character.status.is_draft() {
    return html(DraftView {
      character,
      show_admin_link: session.role.is_storyteller(),
      oldest_active,
    }.render()?);
  }
  if character.status.is_inactive() {
    return html(InactiveView {
      character,
      show_admin_link: session.role.is_storyteller(),
      oldest_active,
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
    .filter(|r| r.value > 0 || r.pending_review)
    .map(|r| StatLine { name: r.name.clone(), value: r.value, pending_review: r.pending_review })
    .collect();
  let power_options: Vec<PowerOption> = power_rows.into_iter()
    .map(|r| PowerOption { name: r.name, in_clan: r.in_clan })
    .collect();
  let influences: Vec<StatLine> = influence_rows.iter()
    .filter(|r| r.value > 0 || r.pending_review)
    .map(|r| StatLine { name: r.name.clone(), value: r.value, pending_review: r.pending_review })
    .collect();
  let influence_options: Vec<String> = influence_rows.iter().map(|r| r.name.clone()).collect();
  let influence_xp_costs: HashMap<String, i32> = influence_rows.into_iter()
    .filter_map(|r| r.xp_cost.map(|c| (r.name, c)))
    .collect();
  let stat_options: Vec<String> = stat_option_rows.iter().map(|r| r.name.clone()).collect();
  let stat_xp_costs: HashMap<String, i32> = stat_option_rows.into_iter()
    .filter_map(|r| r.xp_cost.map(|c| (r.name, c)))
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
      powers: power_options,
      influences: influence_options,
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
    oldest_active,
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
      DraftOperation::Power { power } => {
        let result = sqlx::query!(
          "
INSERT INTO power_raise(vampire_id, power, xp_cost)
SELECT $1, $2::VARCHAR, power_xp_cost.xp_cost
FROM power_xp_cost
WHERE power_xp_cost.in_clan = (
    SELECT EXISTS(
      SELECT 1
      FROM vampire
      JOIN clan USING (clan_id)
      WHERE vampire.vampire_id = $1
        AND ($2::VARCHAR = clan.unique_power OR $2::VARCHAR = clan.power_one OR $2::VARCHAR = clan.power_two)
    )
  )
  AND power_xp_cost.level = (
    SELECT COALESCE(\"value!\"::INT, 0) + 1
    FROM vampire_power
    WHERE vampire_id = $1 AND \"name!\" = $2::VARCHAR
  )
          ",
          vampire_id,
          power,
        )
          .execute(&mut *tx)
          .await?
        ;
        if result.rows_affected() != 1 {
          return Err(Error::invalid_builder_draft(&format!("Missing XP rule for power '{power}'")));
        }
      },
      DraftOperation::Influence { influence } => {
        let result = sqlx::query!(
          "
INSERT INTO influence_raise(vampire_id, influence, xp_cost)
SELECT $1, $2::VARCHAR, influence_xp_cost.xp_cost
FROM influence_xp_cost
WHERE influence_xp_cost.influence = $2::VARCHAR
          ",
          vampire_id,
          influence,
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
