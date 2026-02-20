use super::*;

// Duplicate declaration to groupings/id/transactions/id/mod.rs
// Kept since the usage may differ in the future
#[derive(Debug)]
struct Account{
  id: i64,
  name: String,
  t: String,
}
#[derive(Debug)]
struct ImportedAccountChange {
  id: i64,
  account_name: String,
  date: Date,
  amount: Decimal,
  other_data: String,
}
#[derive(Debug, Template)]
#[template(path = "bookkeepings/id/imported_account_changes/index.html")]
struct Index {
  bookkeeping_name: String,
  imported_account_changes: Vec<ImportedAccountChange>,
  accounts_by_type: std::collections::HashMap<String, Vec<Account>>,
}
async fn index(
  state: &'static State,
  req: Request,
  session: SessionData,
  bookkeeping: Bookkeeping,
) -> Result<Response, Error> {
  // Get all the imported account changes valid for this bookkeeping
  let imported_account_changes = sqlx::query_as!(ImportedAccountChange,
    "
SELECT ImportedAccountChanges.id, Accounts.name AS account_name,
    ImportedAccountChanges.day as date, ImportedAccountChanges.amount,
    ImportedAccountCHanges.other_data
  FROM ImportedAccountChanges
  INNER JOIN Accounts ON ImportedAccountChanges.account_id = Accounts.id
WHERE Accounts.bookkeeping_id = $1
ORDER BY ImportedAccountChanges.day
    ",
    bookkeeping.id
  )
    .fetch_all(&state.db)
    .await?
  ;
  // We need all the accounts (by type) for the form creating account changes
  let accounts = sqlx::query_as!(Account,
    "
SELECT Accounts.id, Accounts.name, Accounts.type AS t
  FROM Accounts
WHERE Accounts.bookkeeping_id = $1
    ",
    bookkeeping.id,
  )
    .fetch_all(&state.db)
    .await?
  ;
  // Then we sort them by account type
  let mut accounts_by_type = std::collections::HashMap::<String,Vec<Account>>::new();
  for account in accounts {
    match accounts_by_type.get_mut(&account.t) {
      Some(x) => x.push(account),
      None => { accounts_by_type.insert(account.t.clone(), vec![account]); },
    }
  }

  html(Index{
    bookkeeping_name: bookkeeping.name,
    imported_account_changes,
    accounts_by_type,
  }.render()?)
}

#[derive(Debug,Deserialize)]
struct NewImportedAccountChange{
  #[serde(alias = "Transaktionsdag")]
  date: Date,
  #[serde(alias = "Belopp")]
  amount: Decimal,
  #[serde(flatten)]
  resten: HashMap<String, String>,
}
#[derive(Debug,Deserialize)]
struct NewImportedAccountChanges{
  account: i64,
  changes_csv: String,
}
async fn index_post(
  state: &'static State,
  mut req: Request,
  session: SessionData,
  bookkeeping: Bookkeeping,
) -> Result<Response, Error> {
  // Parse out the submitted new bookkeeping
  let post_body: NewImportedAccountChanges = parse_body_urlencoded(
    &mut req,
    state.max_content_len,
  ).await?;

  // Furthermore parse the given CSV into the account changes
  let mut reader = csv::ReaderBuilder::new()
    .delimiter(b',')
    .flexible()
    .from_reader(&post_body.changes_csv)
  ;

  let mut to_insert = Vec::new();
  // The header row is automatically skipped, so we go straight to data here
  for row in reader.deserialize()? {
    // Try to create an ImportedAccountChanges from each row
    let new: NewImportedAccountChange = row?;
    // If that was valid, add it to a list and then insert them in bulk
    to_insert.push(new);
  }

  // Finally insert it into the database

  //// Insert into database
  //let created = sqlx::query!(
  //  "INSERT INTO Bookkeepings(name, owner_id) VALUES($1, $2) RETURNING id",
  //  new_bookkeeping.name,
  //  session.user_id,
  //)
  //  .fetch_one(&state.db)
  //  .await
  //  .map_err(|e| -> Error { match e {
  //    sqlx::Error::Database(ref dbe) if dbe.is_unique_violation() => {
  //      ClientError::AlreadyExists(format!(
  //        "A Bookkeeping by name {} already exists.",
  //        new_bookkeeping.name,
  //      )).into()
  //    },
  //    e => e.into(),
  //  }})
  //  ?
  //  .id
  //;

  // Return a the created object
  see_other(&format!("{}/", "dummy"))
}

pub async fn route(
  state: &'static State,
  req: Request,
  mut path_vec: Vec<String>,
  session: SessionData,
  bookkeeping: Bookkeeping,
) -> Result<Response, Error> {
  match path_vec.pop().as_deref() {
    None => permanent_redirect(&format!("{}/", req.uri().path())),
    Some("") => {
      verify_path_end(&path_vec, &req)?;
      match req.method() {
        &Method::GET => index(
          state,
          req,
          session,
          bookkeeping,
        ).await,
        &Method::POST => index_post(
          state,
          req,
          session,
          bookkeeping,
        ).await,
        _ => Err(Error::method_not_found(&req)),
      }
    },
    _ => Err(Error::path_not_found(&req)),
  }
}
