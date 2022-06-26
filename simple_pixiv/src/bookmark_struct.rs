use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;


#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub error: bool,
    pub message: String,
    pub body: Body,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Body {
    pub works: Vec<Work>,
    pub total: i64,
    pub zone_config: ZoneConfig,
    pub extra_data: ExtraData,
    pub bookmark_tags: Vec<Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Work {
    pub id: String,
    pub title: String,
    pub illust_type: i64,
    pub x_restrict: i64,
    pub restrict: i64,
    pub sl: i64,
    pub url: String,
    pub description: String,
    pub tags: Vec<String>,
    pub user_id: String,
    pub user_name: String,
    pub width: i64,
    pub height: i64,
    pub page_count: i64,
    pub is_bookmarkable: bool,
    pub bookmark_data: BookmarkData,
    pub alt: String,
    pub title_caption_translation: TitleCaptionTranslation,
    pub create_date: String,
    pub update_date: String,
    pub is_unlisted: bool,
    pub is_masked: bool,
    pub profile_image_url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkData {
    pub id: String,
    pub private: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TitleCaptionTranslation {
    pub work_title: Value,
    pub work_caption: Value,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZoneConfig {
    pub header: Header,
    pub footer: Footer,
    pub logo: Logo,
    #[serde(rename = "500x500")]
    pub n500x500: n500x500,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Header {
    pub url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Footer {
    pub url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Logo {
    pub url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct n500x500 {
    pub url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtraData {
    pub meta: Meta,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    pub title: String,
    pub description: String,
    pub canonical: String,
    pub ogp: Ogp,
    pub twitter: Twitter,
    pub alternate_languages: AlternateLanguages,
    pub description_header: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ogp {
    pub description: String,
    pub image: String,
    pub title: String,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Twitter {
    pub description: String,
    pub image: String,
    pub title: String,
    pub card: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlternateLanguages {
    pub ja: String,
    pub en: String,
}
