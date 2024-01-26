use std::ops::{Deref, DerefMut};

#[derive(Clone)]
pub struct NormalizedPath<'a>(Vec<PathElement<'a>>);

impl<'a> NormalizedPath<'a> {
    pub fn as_json_pointer(&self) -> String {
        self.0
            .iter()
            .map(PathElement::as_json_pointer)
            .fold(String::from(""), |mut acc, s| {
                acc.push('/');
                acc.push_str(&s.replace('~', "~0").replace('/', "~1"));
                acc
            })
    }
}

impl<'a> Deref for NormalizedPath<'a> {
    type Target = Vec<PathElement<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> DerefMut for NormalizedPath<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Clone, Copy)]
pub enum PathElement<'a> {
    Name(&'a str),
    Index(usize),
}

impl<'a> PathElement<'a> {
    fn as_json_pointer(&self) -> String {
        match self {
            PathElement::Name(ref s) => format!("{s}"),
            PathElement::Index(i) => format!("{i}"),
        }
    }
}

impl<'a> From<&'a String> for PathElement<'a> {
    fn from(s: &'a String) -> Self {
        Self::Name(s.as_str())
    }
}

impl<'a> From<usize> for PathElement<'a> {
    fn from(index: usize) -> Self {
        Self::Index(index)
    }
}

#[cfg(test)]
mod tests {
    use super::{NormalizedPath, PathElement};

    #[test]
    fn normalized_path_to_json_pointer() {
        let np = NormalizedPath(vec![
            PathElement::Name("foo"),
            PathElement::Index(42),
            PathElement::Name("bar"),
        ]);
        assert_eq!(np.as_json_pointer(), "/foo/42/bar",);
    }

    #[test]
    fn normalized_path_to_json_pointer_with_escapes() {
        let np = NormalizedPath(vec![
            PathElement::Name("foo~bar"),
            PathElement::Index(42),
            PathElement::Name("baz/bop"),
        ]);
        assert_eq!(np.as_json_pointer(), "/foo~0bar/42/baz~1bop",);
    }
}
