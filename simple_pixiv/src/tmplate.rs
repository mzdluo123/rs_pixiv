
use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemp{
    pub meta_cache:usize,
    pub bookmark:usize,
}