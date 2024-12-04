use ::serde::Deserialize;

pub fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: serde::Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

pub mod map_serialize {
    use std::{collections::HashMap, fmt};

    use convert_case::{Boundary, Case, Casing};
    use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Visitor, ser::SerializeMap};

    pub fn serialize<'a, T, K, V, S>(target: T, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: IntoIterator<Item = (&'a K, &'a V)>,
        K: Serialize + Casing<String> + 'a,
        V: Serialize + 'a,
    {
        let container: Vec<_> = target.into_iter().collect();
        let mut map = ser.serialize_map(Some(container.len()))?;
        for (k, v) in container {
            let key = k.to_case(Case::Camel);
            map.serialize_entry(&key, v)?;
        }
        map.end()
    }

    pub fn deserialize<'de, K, V, D>(deserializer: D) -> Result<HashMap<K, V>, D::Error>
    where
        D: Deserializer<'de>,
        K: Deserialize<'de> + Casing<String> + From<String> + Eq + std::hash::Hash,
        V: Deserialize<'de>,
    {
        struct MapVisitor<K, V> {
            marker: std::marker::PhantomData<(K, V)>,
        }

        impl<'de, K, V> Visitor<'de> for MapVisitor<K, V>
        where
            K: Deserialize<'de> + Casing<String> + From<String> + Eq + std::hash::Hash,
            V: Deserialize<'de>,
        {
            type Value = HashMap<K, V>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map with camelCase keys")
            }

            fn visit_map<A>(self, mut de_map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut map = HashMap::with_capacity(de_map.size_hint().unwrap_or_default());

                while let Some((k, v)) = de_map.next_entry::<String, _>()? {
                    map.insert(
                        k.from_case(Case::Camel)
                            .without_boundaries(&[Boundary::UpperDigit, Boundary::LowerDigit]) // Needed to not make ch_4_emission
                            .to_case(Case::Snake)
                            .into(),
                        v,
                    );
                }

                Ok(map)
            }
        }

        deserializer.deserialize_map(MapVisitor {
            marker: std::marker::PhantomData,
        })
    }
}
