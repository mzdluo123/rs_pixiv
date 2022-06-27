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
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Work {
    pub id: Value,
    pub x_restrict: i64,
    pub restrict: i64,
}
