
use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemp{
    pub cache_count:usize
}