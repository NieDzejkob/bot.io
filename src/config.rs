use serde::{Deserialize, Deserializer};
use serde::de::{Visitor, MapAccess};
use std::collections::HashSet;
use std::fmt;
use std::hash::Hash;
use std::marker::PhantomData;

/// For use with serde's `deserialize_with` attribute. Deserializes a map into a HashSet
/// by discarding the keys.
pub fn id_list<'de, T, D>(deserializer: D) -> Result<HashSet<T>, D::Error>
where
    T: Deserialize<'de> + Eq + Hash,
    D: Deserializer<'de>,
{
    struct SetVisitor<T>(PhantomData<T>);

    impl<'de, T> Visitor<'de> for SetVisitor<T>
    where
        T: Deserialize<'de> + Eq + Hash,
    {
        type Value = HashSet<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a map with IDs as values")
        }

        fn visit_map<M>(self, mut map: M) -> Result<HashSet<T>, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut result = HashSet::new();
            while let Some((_, value)) = map.next_entry::<&str, _>()? {
                result.insert(value);
            }

            Ok(result)
        }
    }

    deserializer.deserialize_map(SetVisitor(PhantomData))
}
