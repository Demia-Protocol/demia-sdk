use std::{future::Future, sync::Arc};

use serde::{Deserialize, Serialize};

use super::{InputParameter, Parameter, Record, ValueSet};
use crate::errors::AnalyticsError;

#[derive(Default, Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(bound(deserialize = "'de: 'static"))]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsProfile {
    pub id: String,
    pub calculation_interval: core::time::Duration,
    pub calculations: Vec<Calculation>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Calculation {
    pub id: &'static str,
    pub text: &'static str,
    pub parameters: Vec<Parameter>,
    #[serde(skip)]
    pub calculation_function: AsyncCalculationFunctionWrapper<AnalyticsError>,
}

#[derive(Clone)]
pub struct AsyncCalculationFunctionWrapper<E>(pub AsyncCalculationFunction<E>);

impl<E> Default for AsyncCalculationFunctionWrapper<E>
where
    E: Send + Sync + 'static,
{
    fn default() -> Self {
        let default_func: AsyncCalculationFunction<E> = Arc::new(|_, _| Box::new(async { Ok(ValueSet::default()) }));
        AsyncCalculationFunctionWrapper(default_func)
    }
}

impl<E> std::fmt::Debug for AsyncCalculationFunctionWrapper<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AsyncCalculationFunctionWrapper(<function>)")
    }
}

type AsyncCalculationFunction<E> =
    Arc<dyn Fn(Vec<InputParameter>, Vec<Record>) -> Box<dyn Future<Output = Result<ValueSet, E>> + Send + Sync>>;
