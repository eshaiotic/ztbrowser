//! A forgiving, read-only view of a JSON node.
//!
//! DuckDuckGo's documents are large and evolve independently of ZTBrowser. A missing or mistyped field reads
//! as "absent" instead of failing the whole document, the same leniency as DuckDuckGo's Swift decoders.

use serde_json::Value;

#[derive(Debug, Clone, Copy)]
pub struct LenientJson<'document> {
    node: Option<&'document Value>,
}

impl<'document> LenientJson<'document> {
    #[must_use]
    pub fn new(node: &'document Value) -> Self {
        Self { node: Some(node) }
    }

    /// The named field of an object; absent for non-objects and missing fields.
    #[must_use]
    pub fn field(self, name: &str) -> Self {
        Self { node: self.node.and_then(|node| node.get(name)) }
    }

    #[must_use]
    pub fn is_object(self) -> bool {
        self.node.is_some_and(Value::is_object)
    }

    #[must_use]
    pub fn as_str(self) -> Option<&'document str> {
        self.node?.as_str()
    }

    #[must_use]
    pub fn as_bool(self) -> Option<bool> {
        self.node?.as_bool()
    }

    #[must_use]
    pub fn as_u64(self) -> Option<u64> {
        self.node?.as_u64()
    }

    /// `(key, value)` pairs of an object; nothing for any other node.
    pub fn entries(self) -> impl Iterator<Item = (&'document str, Self)> {
        let object = self.node.and_then(Value::as_object);
        object.into_iter().flatten().map(|(key, value)| (key.as_str(), Self::new(value)))
    }

    /// Elements of an array; nothing for any other node.
    pub fn items(self) -> impl Iterator<Item = Self> {
        let array = self.node.and_then(Value::as_array);
        array.into_iter().flatten().map(Self::new)
    }

    /// The string elements of an array (other elements skipped), or `None` if the node is not an array.
    #[must_use]
    pub fn strings(self) -> Option<Vec<String>> {
        self.node?.as_array()?;
        Some(self.items().filter_map(Self::as_str).map(str::to_owned).collect())
    }
}
