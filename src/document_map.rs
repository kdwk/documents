use std::{
    collections::{hash_map::IntoIter, HashMap},
    fmt::Display,
    ops::{Index, IndexMut},
};

use serde::{Deserialize, Serialize};

use crate::Document;

/// A type that wraps a HashMap between a String and Documents. Access the Documents with any type of index that can be converted to a String.
///
/// An instance of this type is provided by [`with`](with) containing all of the [`Document`](Document)s
/// given in the `documents` parameter as the values, and their respective [`alias`](Document::alias)es as keys.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DocumentMap(pub(crate) HashMap<String, Document>);

impl<'a, Str> Index<Str> for DocumentMap
where
    Str: Display,
{
    type Output = Document;
    fn index(&self, index: Str) -> &Self::Output {
        &self.0[index.to_string().as_str()]
    }
}

impl<'a, Str> IndexMut<Str> for DocumentMap
where
    Str: Display,
{
    fn index_mut(&mut self, index: Str) -> &mut Self::Output {
        self.0.get_mut(index.to_string().as_str()).unwrap()
    }
}

impl<'a> IntoIterator for DocumentMap {
    type Item = (String, Document);
    type IntoIter = IntoIter<String, Document>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
