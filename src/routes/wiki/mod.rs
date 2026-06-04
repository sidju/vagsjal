use super::*;

pub(crate) mod wiki_pages {
    include!(concat!(env!("OUT_DIR"), "/wiki_pages.rs"));
}

#[derive(Template)]
#[template(path = "wiki/page.html")]
pub(crate) struct WikiPageTemplate {
  pub title: String,
  pub content: String,
  pub show_admin_link: bool,
}

#[derive(Template)]
#[template(path = "wiki/index.html")]
struct WikiIndex {
  pages: &'static [(&'static str, &'static str)],
  show_admin_link: bool,
}

async fn show_page(
  session: Option<SessionData>,
  page_name: &str,
) -> Result<Response, Error> {
  match wiki_pages::get(page_name) {
    Some(page) => html(WikiPageTemplate {
      title: page.title.to_string(),
      content: page.content.to_string(),
      show_admin_link: session.as_ref().map(|s| s.role.is_storyteller()).unwrap_or(false),
    }.render()?),
    None => Err(ClientError::PathNotFound(format!("/wiki/{page_name}/")).into()),
  }
}

async fn index(
  session: Option<SessionData>,
) -> Result<Response, Error> {
  if let Some(index_page) = wiki_pages::get("index") {
    return html(WikiPageTemplate {
      title: index_page.title.to_string(),
      content: index_page.content.to_string(),
      show_admin_link: session.as_ref().map(|s| s.role.is_storyteller()).unwrap_or(false),
    }.render()?);
  }

  html(WikiIndex {
    pages: wiki_pages::all(),
    show_admin_link: session.as_ref().map(|s| s.role.is_storyteller()).unwrap_or(false),
  }.render()?)
}

pub async fn route(
  session: Option<SessionData>,
  req: Request,
  mut path_vec: Vec<String>,
) -> Result<Response, Error> {
  match path_vec.pop().as_deref() {
    None => permanent_redirect(&format!("{}/", req.uri().path())),
    Some("") => {
      verify_path_end(&path_vec, &req)?;
      match req.method() {
        &Method::GET => index(session).await,
        _ => Err(Error::method_not_found(&req)),
      }
    },
    Some(page) => {
      match path_vec.pop().as_deref() {
        None => return permanent_redirect(&format!("{}/", req.uri().path())),
        Some("") => {},
        _ => return Err(Error::path_not_found(&req)),
      }
      match req.method() {
        &Method::GET => show_page(session, page).await,
        _ => Err(Error::method_not_found(&req)),
      }
    },
  }
}
