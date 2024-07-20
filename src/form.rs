use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum FieldType {
    Number,
    String,
    Boolean
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Field {
    pub(crate) name: String,
    pub(crate) field_type: FieldType,
}



#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FormData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) id: Option<String>,
    pub(crate) fields: Vec<Field>,
    #[serde(skip_deserializing, skip_serializing, default)]
    // #[serde(serialize_with = "serialize")]
    pub(crate) expires_at: Option<DateTime<Utc>>,
}
