use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub body: Body,
    pub error: bool,
    pub message: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Body {
    pub alt: String,
    pub book_style: i64,
    pub bookmark_count: i64,
    pub bookmark_data: Value,
    pub comic_promotion: Value,
    pub comment_count: i64,
    pub comment_off: i64,
    pub contest_banners: Vec<Value>,
    pub contest_data: Value,
    pub create_date: String,
    pub description: String,
    pub description_booth_id: Value,
    pub description_youtube_id: Value,
    pub fanbox_promotion: Value,
    pub height: i64,
    pub id: String,
    pub illust_comment: String,
    pub illust_id: String,
    pub illust_title: String,
    pub illust_type: i64,
    pub image_response_count: i64,
    pub image_response_data: Vec<Value>,
    pub image_response_out_data: Vec<Value>,
    pub is_bookmarkable: bool,
    pub is_howto: bool,
    pub is_original: bool,
    pub is_unlisted: bool,
    pub like_count: i64,
    pub like_data: bool,
    pub page_count: i64,
    pub poll_data: Value,
    pub request: Value,
    pub response_count: i64,
    pub restrict: i64,
    pub series_nav_data: Value,
    pub sl: i64,
    pub storable_tags: Vec<String>,
    pub title: String,
    pub upload_date: String,
    pub urls: Urls,
    pub user_account: String,
    pub user_id: String,
    pub user_name: String,
    pub view_count: i64,
    pub width: i64,
    pub x_restrict: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Urls {
    pub mini: String,
    pub original: String,
    pub regular: String,
    pub small: String,
    pub thumb: String,
}
