use super::JsonLike;

pub trait JsonLikeList<'json>: JsonLike<'json> {
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    fn map<Err>(self, mapper: &mut impl FnMut(Self) -> Result<Self, Err>) -> Result<Self, Err> {
        if self.as_array().is_some() {
            let new = self
                .into_array()
                .unwrap_or_else(|| unreachable!("as_array() confirmed Some above"))
                .into_iter()
                .map(|value| value.map(mapper))
                .collect::<Result<_, _>>()?;

            Ok(Self::array(new))
        } else {
            mapper(self)
        }
    }

    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    fn map_ref<Err>(
        &self,
        mapper: &mut impl FnMut(&Self) -> Result<Self, Err>,
    ) -> Result<Self, Err> {
        if self.as_array().is_some() {
            let new = self
                .as_array()
                .unwrap_or_else(|| unreachable!("as_array() confirmed Some above"))
                .iter()
                .map(|value| value.map_ref(mapper))
                .collect::<Result<_, _>>()?;

            Ok(Self::array(new))
        } else {
            mapper(self)
        }
    }

    fn for_each(&'json self, f: &mut impl FnMut(&'json Self)) {
        if let Some(arr) = self.as_array() {
            for value in arr {
                value.for_each(f);
            }
        } else {
            f(self);
        }
    }
}

impl<'json, T: JsonLike<'json>> JsonLikeList<'json> for T {}

#[cfg(test)]
mod tests {
    #![expect(clippy::unwrap_used, reason = "test code")]
    use serde_json::{Map, json};

    use super::*;

    #[test]
    fn test_map() {
        let value = json!([
            [[null, null, null], [null, null, null]],
            [[null, null, null], [null, null, null]]
        ]);

        let value = value
            .map(&mut |_| anyhow::Ok(serde_json::Value::Object(Map::new())))
            .unwrap();

        assert_eq!(
            value,
            json!([[[{}, {}, {}], [{}, {}, {}]], [[{}, {}, {}], [{}, {}, {}]]])
        );
    }

    #[test]
    fn test_map_ref() {
        let value = json!([
            [[null, null, null], [null, null, null]],
            [[null, null, null], [null, null, null]]
        ]);

        let value = value
            .map_ref(&mut |_| anyhow::Ok(serde_json::Value::Object(Map::new())))
            .unwrap();

        assert_eq!(
            value,
            json!([[[{}, {}, {}], [{}, {}, {}]], [[{}, {}, {}], [{}, {}, {}]]])
        );
    }

    #[test]
    fn test_for_each() {
        let value = json!([
            [[null, null, null], [null, null, null]],
            [[null, null, null], [null, null, null]]
        ]);

        let mut store = Vec::new();

        value.for_each(&mut |value| {
            store.push(value);
        });

        assert_eq!(store.len(), 12);
    }
}
