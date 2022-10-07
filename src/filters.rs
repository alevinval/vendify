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

impl Filters {
    pub fn new() -> Filters {
        Filters {
            targets: vec![],
            ignores: vec![],
            extensions: vec![],
        }
    }

    pub fn add_targets(&mut self, targets: &[String]) -> &mut Self {
        self.targets.extend(targets.to_vec());
        self.targets.sort();
        self.targets.dedup();
        self
    }

    pub fn add_ignores(&mut self, ignores: &[String]) -> &mut Self {
        self.ignores.extend(ignores.to_vec());
        self.ignores.sort();
        self.ignores.dedup();
        self
    }

    pub fn add_extensions(&mut self, extension: &[String]) -> &mut Self {
        self.extensions.extend(extension.to_vec());
        self.extensions.sort();
        self.extensions.dedup();
        self
    }

    pub fn merge(&mut self, other: &Filters) -> &mut Self {
        self.add_targets(&other.targets)
            .add_ignores(&other.ignores)
            .add_extensions(&other.extensions);
        self
    }

    pub fn clear(&mut self) -> &mut Self {
        self.targets.clear();
        self.ignores.clear();
        self.extensions.clear();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::Filters;
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

        sut.add_targets(input);

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

        sut.add_ignores(input);

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

        sut.add_extensions(input);

        assert_eq!(
            get_expected(input),
            sut.extensions,
            "extensions should be sorted and unique",
        );
    }

    #[test]
    fn test_filters_clear() {
        let input = &get_input();
        let sut = &mut Filters::new();
        sut.add_targets(input)
            .add_ignores(input)
            .add_extensions(input);

        sut.clear();

        assert_eq!(0, sut.targets.len(), "there should be no targets");
        assert_eq!(0, sut.ignores.len(), "there should be no ignores");
        assert_eq!(0, sut.extensions.len(), "there should be no extensions");
    }

    #[test]
    fn test_filters_merge() {
        let input = &get_input();
        let sut = &mut Filters::new();
        sut.add_targets(input).add_ignores(input);

        let extra = &svec!["extra"];
        let other = &mut Filters::new();
        other.add_targets(extra).add_extensions(input);

        sut.merge(other);

        let extended_input = &mut get_input();
        extended_input.extend(extra.clone());

        assert_eq!(get_expected(extended_input), sut.targets,);
        assert_eq!(get_expected(input), sut.ignores);
        assert_eq!(get_expected(input), sut.extensions,);
    }
}
