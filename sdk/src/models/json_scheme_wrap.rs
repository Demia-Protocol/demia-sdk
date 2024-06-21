use alvarium_sdk_rust::annotations::{
    constants::{AnnotationType, HashType},
    Annotation as AlvariumAnnotation,
};
use rocket_okapi::okapi::schemars;
use serde::{Deserialize, Serialize};

// Types here anret actually used
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
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(remote = "HashType")]
pub struct HashTypeDef(String);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(remote = "AnnotationType")]
pub struct AnnotationTypeDef(String);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(transparent)]
pub struct Annotation(#[serde(with = "AnnotationDef")] AlvariumAnnotation);
