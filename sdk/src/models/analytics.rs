use std::{
    fmt,
    hash::{Hash, Hasher},
    sync::Arc,
};

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

impl PartialEq for AnalyticsProfile {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for AnalyticsProfile {}

impl Hash for AnalyticsProfile {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl AnalyticsProfile {
    pub async fn get_calculation(&mut self, id: &str) -> Option<&AsyncCalculationFunction<AnalyticsError>> {
        if let Some(calc) = self.calculations.iter_mut().find(|calc| calc.id.eq(id)) {
            Some(&calc.calculation_function.0)
        } else {
            None
        }
    }
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Calculation {
    pub id: String,
    pub text: String,
    pub parameters: Vec<Parameter>,
    #[serde(skip)]
    pub calculation_function: AsyncCalculationFunctionWrapper<AnalyticsError>,
}

impl Default for Calculation {
    fn default() -> Self {
        Calculation {
            id: "".to_owned(),
            text: "".to_owned(),
            parameters: Vec::new(),
            calculation_function: AsyncCalculationFunctionWrapper(Arc::new(|_, _| {
                Box::pin(async { Ok(ValueSet::default()) })
            })),
        }
    }
}

impl fmt::Debug for Calculation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AnalyticsProfile")
            .field("id", &self.id)
            .field("text", &self.text)
            .field("parameters", &self.parameters)
            .finish()
    }
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
