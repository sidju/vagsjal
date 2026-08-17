use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use comrak::html::collect_text;
use comrak::nodes::{AstNode, NodeLink, NodeValue};
use comrak::{Arena, Anchorizer, Options, format_html, parse_document};

fn main() {
  println!("cargo:rerun-if-changed=build.rs");
  println!("cargo:rerun-if-changed=wiki");
  println!("cargo:rerun-if-changed=homepage.md");

  let wiki_dir = Path::new("wiki");
  if !wiki_dir.exists() {
    write_wiki_pages(&[], &BTreeSet::new());
    return;
  }

  let mut entries: Vec<(String, String)> = Vec::new();
  for entry in std::fs::read_dir(wiki_dir).unwrap() {
    let path = entry.unwrap().path();
    if path.extension().and_then(|s| s.to_str()) == Some("md") {
      let name = path.file_stem().unwrap().to_str().unwrap().to_string();
      let content = std::fs::read_to_string(&path).unwrap();
      entries.push((name, content));
    }
  }

  // Include homepage.md from repo root
  let homepage_path = Path::new("homepage.md");
  if homepage_path.exists() {
    let content = std::fs::read_to_string(homepage_path).unwrap();
    entries.push(("homepage".to_string(), content));
  }

  entries.sort_by(|a, b| a.0.cmp(&b.0));

  let display_titles: BTreeMap<String, String> = entries.iter()
    .map(|(name, content)| {
      let title = extract_title(content).unwrap_or_else(|| name.clone());
      (name.clone(), title)
    })
    .collect();

  let options = build_comrak_options();

  // FIRST PASS: analyze all files for backlinks and categories
  let mut analyzer = Analyzer::new();
  for (name, content) in &entries {
    analyzer.set_current_file(name.clone());
    let arena = Arena::new();
    let root = parse_document(&arena, content, &options);
    translate_links(root);
    analyzer.analyze_ast(root);
  }

  // Generate search index from first-pass analysis
  let search_data_js = analyzer.search_data_js();
  let out_dir = std::env::var("OUT_DIR").unwrap();
  std::fs::write(Path::new(&out_dir).join("search_data.js"), &search_data_js).unwrap();

  // Auto-tag pages that serve as categories with the "categories" meta-category
  {
    let page_names: BTreeSet<&str> = entries.iter()
      .filter(|(n, _)| n != "homepage")
      .map(|(n, _)| n.as_str())
      .collect();
    let cat_pages: Vec<String> = analyzer.categories.keys()
      .filter(|k| page_names.contains(k.as_str()) && k.as_str() != "index")
      .cloned()
      .collect();
    for cat_page in cat_pages {
      analyzer.categories
        .entry("categories".to_string())
        .or_default()
        .insert(cat_page);
    }
  }

  // Generate wiki nav partial for the nav dropdown
  write_wiki_nav_partial(&analyzer.categories, &display_titles);

  // SECOND PASS: render each page with full analysis data
  let mut pages: Vec<(String, String, String)> = Vec::new();
  for (name, content) in &entries {
    let arena = Arena::new();
    let root = parse_document(&arena, content, &options);
    translate_links(root);
    linkify_hashtags(&arena, root);

    let mut html = String::new();
    format_html(root, &options, &mut html).unwrap();

    let title = extract_title(content).unwrap_or_else(|| name.clone());

    let mut body = String::new();
    body.push_str(&html);

    // Category pages section (index uses grouped rendering below)
    if name != "index" {
      if let Some(pages_in_cat) = analyzer.categories.get(name) {
        let mut cat_list: Vec<&str> = pages_in_cat.iter().map(|s| s.as_str()).collect();
        cat_list.sort();
        if !cat_list.is_empty() {
          body.push_str("<hr>\n<section class=\"wiki-section\">\n<h2>Sidor i denna kategori:</h2>\n<ul>\n");
          for page in &cat_list {
            let display = display_titles.get(*page).map(|s| s.as_str()).unwrap_or(page);
            body.push_str(&format!("<li><a href=\"/wiki/{page}/\">{display}</a></li>\n"));
          }
          body.push_str("</ul>\n</section>\n");
        }
      }
    }

    // Grouped index section
    if name == "index" {
      let (groups, ungrouped) = build_index_groups(&analyzer.categories, &display_titles);
      if !groups.is_empty() || !ungrouped.is_empty() {
        body.push_str("<hr>\n<section class=\"wiki-section\">\n");
        for (cat, pages) in &groups {
          let label = display_titles.get(*cat).map(|s| s.as_str()).unwrap_or(*cat);
          if display_titles.contains_key(*cat) {
            body.push_str(&format!("<h2><a href=\"/wiki/{cat}/\">{label}</a></h2>\n"));
          } else {
            body.push_str(&format!("<h2>{label}</h2>\n"));
          }
          if !pages.is_empty() {
            body.push_str("<ul>\n");
            for page in pages {
              let display = display_titles.get(*page).map(|s| s.as_str()).unwrap_or(*page);
              body.push_str(&format!("<li><a href=\"/wiki/{page}/\">{display}</a></li>\n"));
            }
            body.push_str("</ul>\n");
          }
        }
        if !ungrouped.is_empty() {
          body.push_str("<h2>Övrigt</h2>\n<ul>\n");
          for page in &ungrouped {
            let display = display_titles.get(*page).map(|s| s.as_str()).unwrap_or(*page);
            body.push_str(&format!("<li><a href=\"/wiki/{page}/\">{display}</a></li>\n"));
          }
          body.push_str("</ul>\n");
        }
        body.push_str("</section>\n");
      }
    }

    // Backlinks section
    if let Some(links) = analyzer.backlinks.get(name) {
      let mut link_list: Vec<&str> = links.iter().map(|s| s.as_str()).collect();
      link_list.sort();
      if !link_list.is_empty() {
        body.push_str("<hr>\n<section class=\"wiki-section\">\n<h2>Sidor som länkar hit:</h2>\n<ul>\n");
        for link in &link_list {
          let display = display_titles.get(*link).map(|s| s.as_str()).unwrap_or(link);
          body.push_str(&format!("<li><a href=\"/wiki/{link}/\">{display}</a></li>\n"));
        }
        body.push_str("</ul>\n</section>\n");
      }
    }

    pages.push((name.clone(), title, body));
  }

  // Auto-generated category pages for tags without a matching file
  let real_page_names: BTreeSet<String> = entries.iter()
    .filter(|(n, _)| n != "homepage")
    .map(|(n, _)| n.clone())
    .collect();
  let existing: BTreeSet<String> = pages.iter().map(|(n, _, _)| n.clone()).collect();
  let mut auto_pages: Vec<(String, String, String)> = Vec::new();
  for (category, members) in &analyzer.categories {
    if existing.contains(category) || members.is_empty() {
      continue;
    }
    let mut body = String::from("<h1>") + category + "</h1>\n";
    body.push_str("<hr>\n<section class=\"wiki-section\">\n<h2>Sidor i denna kategori:</h2>\n<ul>\n");
    let mut member_list: Vec<&str> = members.iter().map(|s| s.as_str()).collect();
    member_list.sort();
    for page in &member_list {
      let display = display_titles.get(*page).map(|s| s.as_str()).unwrap_or(page);
      body.push_str(&format!("<li><a href=\"/wiki/{page}/\">{display}</a></li>\n"));
    }
    body.push_str("</ul>\n</section>\n");

    // Backlinks for auto-generated pages
    if let Some(links) = analyzer.backlinks.get(category) {
      let mut link_list: Vec<&str> = links.iter().map(|s| s.as_str()).collect();
      link_list.sort();
      if !link_list.is_empty() {
        body.push_str("<hr>\n<section class=\"wiki-section\">\n<h2>Sidor som länkar hit:</h2>\n<ul>\n");
        for link in &link_list {
          let display = display_titles.get(*link).map(|s| s.as_str()).unwrap_or(link);
          body.push_str(&format!("<li><a href=\"/wiki/{link}/\">{display}</a></li>\n"));
        }
        body.push_str("</ul>\n</section>\n");
      }
    }

    auto_pages.push((category.clone(), category.clone(), body));
  }

  // Append auto-generated pages after real pages (they won't appear in all())
  pages.extend(auto_pages);

  write_wiki_pages(&pages, &real_page_names);
}

fn write_wiki_nav_partial(
  categories: &BTreeMap<String, BTreeSet<String>>,
  display_titles: &BTreeMap<String, String>,
) {
  let (groups, ungrouped) = build_index_groups(categories, display_titles);

  let mut html = String::new();
  for (cat, pages) in &groups {
    let label = display_titles.get(*cat).map(|s| s.as_str()).unwrap_or(*cat);
    if display_titles.contains_key(*cat) {
      html.push_str(&format!("<strong><a href=\"/wiki/{cat}/\">{label}</a></strong>\n"));
    } else {
      html.push_str(&format!("<strong>{label}</strong>\n"));
    }
    if !pages.is_empty() {
      html.push_str("<ul>\n");
      for page in pages {
        let display = display_titles.get(*page).map(|s| s.as_str()).unwrap_or(*page);
        html.push_str(&format!("<li><a href=\"/wiki/{page}/\">{display}</a></li>\n"));
      }
      html.push_str("</ul>\n");
    }
  }
  if !ungrouped.is_empty() {
    html.push_str("<strong>Övrigt</strong>\n<ul>\n");
    for page in &ungrouped {
      let display = display_titles.get(*page).map(|s| s.as_str()).unwrap_or(*page);
      html.push_str(&format!("<li><a href=\"/wiki/{page}/\">{display}</a></li>\n"));
    }
    html.push_str("</ul>\n");
  }

  std::fs::write(Path::new("templates").join("wiki_nav_partial.html"), &html).unwrap();
}

fn build_index_groups<'a>(
  categories: &'a BTreeMap<String, BTreeSet<String>>,
  display_titles: &'a BTreeMap<String, String>,
) -> (BTreeMap<&'a str, Vec<&'a str>>, Vec<&'a str>) {
  let Some(index_pages) = categories.get("index") else {
    return (BTreeMap::new(), Vec::new());
  };

  let mut page_cats: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
  for page in index_pages {
    page_cats.entry(page.as_str()).or_default();
  }
  for (cat, members) in categories {
    if cat == "index" || cat == "categories" { continue; }
    for member in members {
      if let Some(cats) = page_cats.get_mut(member.as_str()) {
        cats.insert(cat.as_str());
      }
    }
  }

  let mut groups: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
  let mut ungrouped: Vec<&str> = Vec::new();
  for (page, cats) in &page_cats {
    if cats.is_empty() {
      ungrouped.push(*page);
    } else {
      for cat in cats {
        groups.entry(*cat).or_default().push(*page);
      }
    }
  }

  for (cat, pages) in &mut groups {
    pages.retain(|p| p != cat);
    pages.sort_by(|a, b| {
      let a_title = display_titles.get(*a).map(|s| s.as_str()).unwrap_or(*a);
      let b_title = display_titles.get(*b).map(|s| s.as_str()).unwrap_or(*b);
      a_title.cmp(b_title)
    });
  }
  ungrouped.retain(|page| !groups.contains_key(*page));
  ungrouped.sort_by(|a, b| {
    let a_title = display_titles.get(*a).map(|s| s.as_str()).unwrap_or(*a);
    let b_title = display_titles.get(*b).map(|s| s.as_str()).unwrap_or(*b);
    a_title.cmp(b_title)
  });

  (groups, ungrouped)
}

fn write_wiki_pages(pages: &[(String, String, String)], real_pages: &BTreeSet<String>) {
  let out_dir = std::env::var("OUT_DIR").unwrap();
  let dest = Path::new(&out_dir).join("wiki_pages.rs");

  let mut code = String::new();
  code.push_str(
    "#[allow(non_upper_case_globals)]\n\
     pub struct Page {\n\
     pub title: &'static str,\n\
     pub content: &'static str,\n\
     }\n\
     \n"
  );

  // get() lookup function
  code.push_str("pub fn get(name: &str) -> Option<Page> {\n");
  code.push_str("match name {\n");
  for (name, title, content) in pages {
    code.push_str(&format!(
      "  {name:?} => Some(Page {{ title: {}, content: {} }}),\n",
      escape_rust_str(title),
      escape_rust_str(content),
    ));
  }
  code.push_str("  _ => None,\n");
  code.push_str("}\n");
  code.push_str("}\n\n");

  // all() returns name + title pairs for listing
  code.push_str("pub fn all() -> &'static [(&'static str, &'static str)] {\n");
  code.push_str("  &[\n");
  for (name, title, _) in pages {
    if real_pages.contains(name) {
      code.push_str(&format!(
        "    ({name:?}, {}),\n",
        escape_rust_str(title),
      ));
    }
  }
  code.push_str("  ]\n");
  code.push_str("}\n");

  // Trim trailing whitespace/newlines for cleanliness
  let code = code.trim_end().to_string();
  std::fs::write(&dest, code).unwrap();
}

fn escape_rust_str(s: &str) -> String {
  let mut out = String::with_capacity(s.len() + 2);
  out.push('"');
  for ch in s.chars() {
    match ch {
      '"' => out.push_str("\\\""),
      '\\' => out.push_str("\\\\"),
      '\n' => out.push('\n'),
      '\r' => out.push('\r'),
      '\t' => out.push('\t'),
      c if c.is_control() => {
        out.push_str(&format!("\\u{{{:04x}}}", c as u32));
      }
      c => out.push(c),
    }
  }
  out.push('"');
  out
}

// ---- ported from md-wiki ----

fn build_comrak_options() -> Options<'static> {
  let mut options = Options::default();
  options.extension.strikethrough = true;
  options.extension.table = true;
  options.extension.header_id_prefix = Some(String::new());
  options.render.r#unsafe = true;
  options
}

fn translate_links<'a>(root: &'a AstNode<'a>) {
  for node in root.descendants() {
    let url = match &node.data().value {
      NodeValue::Link(link) => Some(link.url.clone()),
      NodeValue::Image(link) => Some(link.url.clone()),
      _ => None,
    };
    if let Some(url) = url {
      let new_url = if let Some(hash_pos) = url.find('#') {
        let (path, fragment) = url.split_at(hash_pos);
        if path.ends_with(".md") {
          format!("/wiki/{}/{}", &path[..path.len() - 3], fragment)
        } else {
          url
        }
      } else if url.ends_with(".md") {
        format!("/wiki/{}/", &url[..url.len() - 3])
      } else {
        url
      };
      match &mut node.data_mut().value {
        NodeValue::Link(link) => link.url = new_url,
        NodeValue::Image(link) => link.url = new_url,
        _ => {}
      }
    }
  }
}

fn linkify_hashtags<'a>(arena: &'a Arena<'a>, root: &'a AstNode<'a>) {
  let text_nodes: Vec<&'a AstNode<'a>> = root
    .descendants()
    .filter(|node| matches!(&node.data().value, NodeValue::Text(t) if t.contains('#')))
    .collect();

  for node in text_nodes {
    let text = match &node.data().value {
      NodeValue::Text(t) => t.to_string(),
      _ => continue,
    };

    let mut hashtags: Vec<(usize, usize, String)> = Vec::new();
    parse_hashtags(&text, |start, end, tag| {
      hashtags.push((start, end, tag.to_string()));
    });

    if hashtags.is_empty() {
      continue;
    }

    let mut last_end = 0;
    for (start, end, tag) in &hashtags {
      if *start > last_end {
        let before = arena.alloc(AstNode::from(NodeValue::Text(
          Cow::Owned(text[last_end..*start].to_string()),
        )));
        node.insert_before(before);
      }

      let link_node = arena.alloc(AstNode::from(NodeValue::Link(Box::new(NodeLink {
        url: format!("/wiki/{tag}/"),
        title: String::new(),
      }))));
      let link_text = arena.alloc(AstNode::from(NodeValue::Text(
        Cow::Owned(format!("#{tag}")),
      )));
      link_node.append(link_text);
      node.insert_before(link_node);

      last_end = *end;
    }

    if last_end < text.len() {
      node.data_mut().value =
        NodeValue::Text(Cow::Owned(text[last_end..].to_string()));
    } else {
      node.detach();
    }
  }
}

fn parse_hashtags<F>(text: &str, mut callback: F)
where
  F: FnMut(usize, usize, &str),
{
  for (idx, _) in text.match_indices('#') {
    let valid_prefix = if idx == 0 {
      true
    } else {
      let mut prev_idx = idx - 1;
      while prev_idx > 0 && !text.is_char_boundary(prev_idx) {
        prev_idx -= 1;
      }
      if let Some(prev_char) = text[prev_idx..idx].chars().next() {
        prev_char.is_whitespace() || matches!(prev_char, '(' | '[' | '{')
      } else {
        false
      }
    };

    if valid_prefix {
      let after_hash = &text[idx + 1..];
      let category_end = after_hash
        .find(|c: char| !(c.is_alphanumeric() || c == '_' || c == '-'))
        .unwrap_or(after_hash.len());
      let category = &after_hash[..category_end];

      if !category.is_empty() {
        let hashtag_end = idx + 1 + category_end;
        callback(idx, hashtag_end, category);
      }
    }
  }
}

struct Analyzer {
  backlinks: BTreeMap<String, BTreeSet<String>>,
  categories: BTreeMap<String, BTreeSet<String>>,
  current_file: String,
  current_headings: Vec<(String, String)>,
  documents: Vec<(String, Vec<(String, String)>)>,
}

impl Analyzer {
  fn new() -> Self {
    Self {
      backlinks: BTreeMap::new(),
      categories: BTreeMap::new(),
      current_file: String::new(),
      current_headings: Vec::new(),
      documents: Vec::new(),
    }
  }

  fn set_current_file(&mut self, filename: String) {
    if !self.current_file.is_empty() && !self.current_headings.is_empty() {
      self.documents.push((
        self.current_file.clone(),
        std::mem::take(&mut self.current_headings),
      ));
    }
    self.current_file = filename;
  }

  fn analyze_ast<'a>(&mut self, root: &'a AstNode<'a>) {
    let mut anchorizer = Anchorizer::new();
    for node in root.descendants() {
      match &node.data().value {
        NodeValue::Link(link) => {
          let url = &link.url;
          if let Some(target) = url.strip_prefix("/wiki/") {
            // Strip any fragment
            let target = match target.find('#') {
              Some(pos) => &target[..pos],
              None => target,
            }.trim_end_matches('/');
            if !target.is_empty() {
              self.backlinks
                .entry(target.to_string())
                .or_default()
                .insert(self.current_file.clone());
            }
          }
        }
        NodeValue::Text(text) => {
          parse_hashtags(text, |_start, _end, category| {
            self.categories
              .entry(category.to_string())
              .or_default()
              .insert(self.current_file.clone());
          });
        }
        NodeValue::Heading(_) => {
          let text = collect_text(node);
          let id = anchorizer.anchorize(&text);
          self.current_headings.push((text, id));
        }
        _ => {}
      }
    }
  }

  fn search_data_js(&mut self) -> String {
    if !self.current_file.is_empty() && !self.current_headings.is_empty() {
      self.documents.push((
        self.current_file.clone(),
        std::mem::take(&mut self.current_headings),
      ));
    }
    build_search_index_js(&self.documents)
  }
}

fn extract_title(content: &str) -> Option<String> {
  for line in content.lines() {
    let trimmed = line.trim();
    if trimmed.starts_with("# ") {
      return Some(trimmed[2..].trim().to_string());
    }
  }
  None
}

fn escape_json_str(s: &str) -> String {
  let mut out = String::with_capacity(s.len() + 2);
  out.push('"');
  for ch in s.chars() {
    match ch {
      '"' => out.push_str("\\\""),
      '\\' => out.push_str("\\\\"),
      '\n' => out.push_str("\\n"),
      '\r' => out.push_str("\\r"),
      '\t' => out.push_str("\\t"),
      c if c.is_control() => {
        out.push_str(&format!("\\u{:04x}", c as u32));
      }
      c => out.push(c),
    }
  }
  out.push('"');
  out
}

fn build_search_index_js(docs: &[(String, Vec<(String, String)>)]) -> String {
  let mut js = String::from("window.SEARCH_INDEX_DATA={\"documents\":[");
  for (i, (path, headings)) in docs.iter().enumerate() {
    if i > 0 {
      js.push(',');
    }
    js.push_str(&format!(
      "{{\"path\":{}, \"headings\":[",
      escape_json_str(path)
    ));
    for (j, (text, id)) in headings.iter().enumerate() {
      if j > 0 {
        js.push(',');
      }
      js.push_str(&format!(
        "{{\"text\":{}, \"id\":{}}}",
        escape_json_str(text),
        escape_json_str(id)
      ));
    }
    js.push_str("]}");
  }
  js.push_str("]};");
  js
}
