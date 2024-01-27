#[derive(Debug, Clone, Default)]
pub struct NormalizedPath<'a>(Vec<PathElement<'a>>);

impl<'a> NormalizedPath<'a> {
    pub(crate) fn push<T: Into<PathElement<'a>>>(&mut self, elem: T) {
        self.0.push(elem.into())
    }

    pub(crate) fn clone_and_push<T: Into<PathElement<'a>>>(&self, elem: T) -> Self {
        let mut new_path = self.clone();
        new_path.push(elem.into());
        new_path
    }

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

#[derive(Debug, Clone, Copy)]
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
