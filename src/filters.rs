use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct Filters {
    #[serde(default = "Vec::new")]
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub targets: Vec<String>,

    #[serde(default = "Vec::new")]
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub ignores: Vec<String>,

    #[serde(default = "Vec::new")]
    #[serde(skip_serializing_if = "<[_]>::is_empty")]
    pub extensions: Vec<String>,
}

pub enum FilterKind {
    Target(Vec<String>),
    Ignore(Vec<String>),
    Extension(Vec<String>),
}

impl Filters {
    pub fn new() -> Self {
        Self {
            targets: vec![],
            ignores: vec![],
            extensions: vec![],
        }
    }

    pub fn add(&mut self, element: FilterKind) -> &mut Self {
        match element {
            FilterKind::Target(target) => Self::extend(&mut self.targets, &target),
            FilterKind::Ignore(ignore) => Self::extend(&mut self.ignores, &ignore),
            FilterKind::Extension(extension) => Self::extend(&mut self.extensions, &extension),
        };
        self
    }

    pub fn merge(&mut self, other: &Filters) -> &mut Self {
        self.add(FilterKind::Target(other.targets.clone()))
            .add(FilterKind::Ignore(other.ignores.clone()))
            .add(FilterKind::Extension(other.extensions.clone()))
    }

    pub fn clear(&mut self) -> &mut Self {
        self.targets.clear();
        self.ignores.clear();
        self.extensions.clear();
        self
    }

    fn extend(target: &mut Vec<String>, elems: &[String]) {
        target.extend(elems.to_vec());
        target.sort();
        target.dedup();
    }
}

impl Default for Filters {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::svec;

    fn get_input() -> Vec<String> {
        svec!["d", "c", "b", "a", "c", "a", "a", "b", "f", "g", "d"]
    }

    fn get_expected(input: &[String]) -> Vec<String> {
        let input = &mut input.to_vec();
        input.sort();
        input.dedup();

        input.clone()
    }

    #[test]
    fn test_filters_add_targets() {
        let input = &get_input();
        let sut = &mut Filters::new();

        sut.add(FilterKind::Target(input.clone()));

        assert_eq!(
            get_expected(input),
            sut.targets,
            "targets should be sorted and unique",
        );
    }

    #[test]
    fn test_filters_add_ignores() {
        let input = &get_input();
        let sut = &mut Filters::new();

        sut.add(FilterKind::Ignore(input.clone()));

        assert_eq!(
            get_expected(input),
            sut.ignores,
            "ignores should be sorted and unique",
        );
    }

    #[test]
    fn test_filters_add_extensions() {
        let input = &get_input();
        let sut = &mut Filters::new();

        sut.add(FilterKind::Extension(input.clone()));

        assert_eq!(
            get_expected(input),
            sut.extensions,
            "extensions should be sorted and unique",
        );
    }

    #[test]
    fn test_filters_clear() {
        let input = get_input();
        let sut = &mut Filters::new();
        sut.add(FilterKind::Target(input.clone()))
            .add(FilterKind::Ignore(input.clone()))
            .add(FilterKind::Extension(input));

        sut.clear();

        assert_eq!(0, sut.targets.len(), "there should be no targets");
        assert_eq!(0, sut.ignores.len(), "there should be no ignores");
        assert_eq!(0, sut.extensions.len(), "there should be no extensions");
    }

    #[test]
    fn test_filters_merge() {
        let input = &get_input();
        let sut = &mut Filters::new();
        sut.add(FilterKind::Target(input.clone()))
            .add(FilterKind::Ignore(input.clone()));

        let extra = &svec!["extra"];
        let other = &mut Filters::new();
        other
            .add(FilterKind::Target(extra.clone()))
            .add(FilterKind::Extension(input.clone()));

        sut.merge(other);

        let extended_input = &mut get_input();
        extended_input.extend(extra.clone());

        assert_eq!(get_expected(extended_input), sut.targets,);
        assert_eq!(get_expected(input), sut.ignores);
        assert_eq!(get_expected(input), sut.extensions,);
    }
}
