use std::cmp::Ordering;
use git2::Commit as Git2Commit;
use crate::commit::CommitType::*;
use colored::*;


#[derive(Debug, Eq, PartialEq)]
pub struct Commit<'a> {
    pub(crate) shorthand: String,
    pub(crate) commit_type: CommitType<'a>,
    pub(crate) scope: Option<String>,
    pub(crate) description: String,
    pub(crate) author: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortCommit {
    ByDate,
    ByType,
    ByScope,
    ByTypeAndScope
}

impl Commit<'_> {
    pub fn from_git_commit(commit: Git2Commit) -> Self {
        let shorthand = commit.as_object().short_id().unwrap().as_str().unwrap().to_string();
        let message = commit.message().unwrap();
        print!("Parsing commit : {} - {}", shorthand, message);
        let author = commit.author().name().unwrap_or_else(|| "").to_string();
        let split: Vec<&str> = message.split(": ").collect();
        let description = split[1].to_owned().replace('\n', "");

        let left_part: Vec<&str> = split[0]
            .split("(")
            .collect();

        let commit_type = CommitType::from(left_part[0]);
        let scope = left_part
            .get(1)
            .map(|scope| scope[0..scope.len() - 1].to_owned());

        Commit {
            shorthand,
            commit_type,
            scope,
            description,
            author,
        }
    }

    pub fn to_markdown(&self) -> String {
        format!("{} - {} - {}\n", self.shorthand.yellow(), self.description, self.author.blue())
    }
}

#[derive(Eq, PartialEq, Debug)]
pub(crate) enum CommitType<'a> {
    Feature,
    BugFix,
    Chore,
    Revert,
    Performances,
    Documentation,
    Style,
    Refactoring,
    Test,
    Build,
    Ci,
    Custom(&'a str, &'a str),
}

impl CommitType<'_> {
    pub(crate) fn get_markdown_title(&self) -> &str {
        match self {
            Feature => "Feature",
            BugFix => "Bug Fixes",
            Chore => "Miscellaneous Chores",
            Revert => "Revert",
            Performances => "Performance Improvements",
            Documentation => "Documentation",
            Style => "Style",
            Refactoring => "Refactoring",
            Test => "Tests",
            Build => "Build System",
            Ci => "Continuous Integration",
            Custom(_, value) => value,
        }
    }
}

impl From<&str> for CommitType<'_> {
    fn from(commit_type: &str) -> Self {
        match commit_type {
            "feat" => Feature,
            "fix" => BugFix,
            "chore" => Chore,
            "revert" => Revert,
            "perf" => Performances,
            "docs" => Documentation,
            "style" => Style,
            "refactor" => Refactoring,
            "test" => Test,
            "build" => Build,
            "ci" => Ci,
            _ => panic!("unknown commit type {}", commit_type)
        }
    }
}

impl PartialOrd for Commit<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.scope.partial_cmp(&other.scope)
    }
}

impl Ord for Commit<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.scope.cmp(&other.scope)
    }
}

#[cfg(test)]
mod test {
    use super::Commit;

    #[test]
    fn should_map_conventional_commit_message_to_struct() {
        // Arrange
        let message = "feat(database): add postgresql driver";

        // Act
        let commit = Commit::from_raw_message(message);

        // Assert
        assert_eq!(commit.commit_type, "feat".to_owned());
        assert_eq!(commit.scope, Some("database".to_owned()));
        assert_eq!(commit.description, "add postgresql driver".to_owned());
    }
}