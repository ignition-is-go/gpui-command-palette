use crate::Command;
use std::collections::HashSet;

/// A result in registration order. `score` is retained for API compatibility;
/// the reference palette does not rank matches.
#[derive(Clone, Debug, PartialEq)]
pub struct SearchResult<M: 'static = ()> {
    pub entry: Command<M>,
    pub score: u32,
    pub catalog_index: usize,
}

/// Reference-compatible case-insensitive substring filtering.
///
/// The trimmed query is matched against name, description, and group. IDs are
/// deliberately not searchable. Searchable direct children are promoted after
/// their parent in child registration order and de-duplicated by id, first win.
pub fn search_commands<M: Clone + 'static>(
    commands: &[Command<M>],
    query: &str,
) -> Vec<SearchResult<M>> {
    let query = query.trim().to_lowercase();
    let mut candidates = Vec::new();
    for command in commands {
        if query.is_empty() || matches_query(command, &query) {
            candidates.push(command.clone());
        }
        if !query.is_empty() && command.searches_children() {
            if let Some(children) = command.resolve_children() {
                for mut child in children {
                    if matches_query(&child, &query) {
                        if child.description.is_none() {
                            child.description = Some(command.name.clone());
                        }
                        candidates.push(child);
                    }
                }
            }
        }
    }
    let mut seen = HashSet::new();
    candidates
        .into_iter()
        .enumerate()
        .filter(|(_, c)| seen.insert(c.id.clone()))
        .map(|(catalog_index, entry)| SearchResult {
            entry,
            score: 0,
            catalog_index,
        })
        .collect()
}

fn matches_query<M>(command: &Command<M>, query: &str) -> bool {
    command.name.to_lowercase().contains(query)
        || command
            .description
            .as_ref()
            .is_some_and(|v| v.to_lowercase().contains(query))
        || command
            .group
            .as_ref()
            .is_some_and(|v| v.to_lowercase().contains(query))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn substring_only_and_registration_order() {
        let xs = vec![
            Command::new("z-id-match", "Second Alpha", || {}).group("File"),
            Command::new("a", "Alpha", || {}),
            Command::new("of", "Open File", || {}),
        ];
        assert_eq!(
            search_commands(&xs, "ALPHA")
                .iter()
                .map(|x| x.entry.id.as_str())
                .collect::<Vec<_>>(),
            ["z-id-match", "a"]
        );
        assert!(search_commands(&xs, "z-id").is_empty());
        assert!(search_commands(&xs, "OF").is_empty());
        assert_eq!(
            search_commands(&xs, "")
                .iter()
                .map(|x| x.entry.id.as_str())
                .collect::<Vec<_>>(),
            ["z-id-match", "a", "of"]
        );
    }
    #[test]
    fn promotions_context_and_first_id_wins() {
        let direct = Command::new("same", "Direct Sunset", || {});
        let branch = Command::submenu("scenes", "Open Scene", || {
            vec![
                Command::new("same", "Sunset", || {}),
                Command::new("dawn", "Sunset Dawn", || {}).description("kept"),
            ]
        })
        .searchable_children();
        let out = search_commands(&[direct, branch], "sunset");
        assert_eq!(
            out.iter().map(|x| x.entry.id.as_str()).collect::<Vec<_>>(),
            ["same", "dawn"]
        );
        assert_eq!(out[1].entry.description.as_deref(), Some("kept"));
    }
}
