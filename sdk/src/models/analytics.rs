use std::sync::Arc;

use futures_util::future::BoxFuture;
use serde::{Deserialize, Serialize};

use super::{InputParameter, Parameter, Record, ValueSet};
use crate::errors::AnalyticsError;

#[derive(Default, Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsProfile {
    pub id: String,
    pub calculation_interval: core::time::Duration,
    pub calculations: Vec<Calculation>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Calculation {
    pub id: String,
    pub text: String,
    pub parameters: Vec<Parameter>,
    #[serde(skip)]
    pub calculation_function: AsyncCalculationFunctionWrapper<AnalyticsError>,
}

#[derive(Clone)]
pub struct AsyncCalculationFunctionWrapper<E>(pub AsyncCalculationFunction<E>);

impl<E> core::ops::Deref for AsyncCalculationFunctionWrapper<E> {
    type Target = AsyncCalculationFunction<E>;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E> Default for AsyncCalculationFunctionWrapper<E>
where
    E: Send + Sync + 'static,
{
    fn default() -> Self {
        let default_func: AsyncCalculationFunction<E> = Arc::new(|_, _| Box::pin(async { Ok(ValueSet::default()) }));
        AsyncCalculationFunctionWrapper(default_func)
    }
}

impl<E> std::fmt::Debug for AsyncCalculationFunctionWrapper<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AsyncCalculationFunctionWrapper(<function>)")
    }
}

pub type AsyncCalculationFunction<E> =
    Arc<dyn Fn(Vec<InputParameter>, Vec<Record>) -> BoxFuture<'static, Result<ValueSet, E>> + Send + Sync>;
