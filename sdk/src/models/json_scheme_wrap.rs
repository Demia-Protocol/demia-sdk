use alvarium_sdk_rust::annotations::{
    constants::{AnnotationType, HashType, LayerType},
    Annotation as AlvariumAnnotation,
};
use rocket_okapi::okapi::schemars;
use serde::{Deserialize, Serialize};

// Types here arent actually used
// They act as a jsonSchema serializer to generate the patterns

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(remote = "AlvariumAnnotation")]
pub struct AnnotationDef {
    pub id: String,
    pub key: String,
    #[serde(with = "HashTypeDef")]
    pub hash: HashType,
    pub host: String,
    #[serde(with = "AnnotationTypeDef")]
    pub kind: AnnotationType,
    pub signature: String,
    #[serde(rename = "isSatisfied")]
    pub is_satisfied: bool,
    pub timestamp: String,
    #[serde(with = "LayerTypeDef")]
    #[serde(default = "default_layer")]
    pub layer: LayerType,
    #[serde(default)]
    pub tag: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(remote = "LayerType")]
pub struct LayerTypeDef(String);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(remote = "HashType")]
pub struct HashTypeDef(String);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(remote = "AnnotationType")]
pub struct AnnotationTypeDef(String);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(transparent)]
pub struct Annotation(#[serde(with = "AnnotationDef")] pub AlvariumAnnotation);

impl From<AlvariumAnnotation> for Annotation {
    fn from(ann: AlvariumAnnotation) -> Self {
        Self(ann)
    }
}

fn default_layer() -> LayerType {
    LayerType("host".to_string())
}
