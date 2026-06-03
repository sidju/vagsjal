use super::*;

// Opening a specific character id is for using XP (and storytellers can approve XP use)
mod id;

#[derive(Debug)]
struct Character {
  vampire_id: i64,
  name: String,
  apparent_age: i32, // in years
  date_embraced: Date,
  torpor_time: sqlx::postgres::types::PgInterval,
  clan_name: String,
  remaining_xp: i64,
}
#[derive(Debug)]
struct Stat {
  name: String,
  value: i64,
  pending_review: bool,
}
#[derive(Template)]
#[template(path = "characters/index.html")]
struct Index {
  user_id: i64,
  character_stats: Vec<(
    Character,
    // Split into stats, powers, influences
    Vec<Stat>, Vec<Stat>, Vec<Stat>
  )>,
}

async fn index(
  state: &'static State,
  session: SessionData,
) -> Result<Response, Error> {
  // Storytellers see all characters; users see only their own
  let characters = sqlx::query_as!(Character,
    "
SELECT
  vampire.vampire_id,
  vampire.name,
  vampire.apparent_age,
  vampire.date_embraced,
  vampire.torpor_time,
  clan.name AS clan_name,
  COALESCE(xp_remaining.amount, 0) AS \"remaining_xp!\"
FROM vampire
JOIN clan USING (clan_id)
JOIN xp_remaining USING (vampire_id)
WHERE ($1 OR vampire.user_id = $2)
    ",
    session.role.is_storyteller(),
    session.user_id,
  )
    .fetch_all(&state.db)
    .await?
  ;
  // Fetch the stats for each character
  let mut character_stats = Vec::new();
  for c in characters {
    let stats = sqlx::query_as!(Stat,
      "
SELECT \"name!\", \"value!\", \"pending_review!\"
FROM vampire_stat
WHERE vampire_id = $1
      ",
      c.vampire_id
    )
      .fetch_all(&state.db)
      .await?
    ;
    let powers = sqlx::query_as!(Stat,
      "
SELECT \"name!\", \"value!\", \"pending_review!\"
FROM vampire_power
WHERE vampire_id = $1
      ",
      c.vampire_id
    )
      .fetch_all(&state.db)
      .await?
    ;
    let influences = sqlx::query_as!(Stat,
      "
SELECT \"name!\", \"value!\", \"pending_review!\"
FROM vampire_influence
WHERE vampire_id = $1
      ",
      c.vampire_id
    )
      .fetch_all(&state.db)
      .await?
    ;
    character_stats.push((c,stats,powers,influences));
  }

  // Render and return
  html(Index{
    user_id: session.user_id,
    character_stats,
  }.render()?)
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
        &Method::GET => index(state, session).await,
        _ => Err(Error::method_not_found(&req)),
      }
    },
    // Parse the path into an integer id and keep routing
    Some(id) => id::route(state, req, path_vec, session, id.parse()?).await,
  }
}
