use crate::Command;
use std::collections::HashSet;
#[derive(Clone, Debug, PartialEq)]
pub struct SearchResult<M: 'static = ()> {
    pub entry: Command<M>,
    pub score: u32,
    pub catalog_index: usize,
}
/// Deterministic token/fuzzy search. Every whitespace token must match; ties preserve registration order.
pub fn search_commands<M: Clone + 'static>(
    commands: &[Command<M>],
    query: &str,
) -> Vec<SearchResult<M>> {
    let tokens = query
        .split_whitespace()
        .map(|v| v.to_lowercase())
        .collect::<Vec<_>>();
    let mut candidates = Vec::new();
    for command in commands {
        candidates.push(command.clone());
        if !tokens.is_empty() && command.searches_children() {
            if let Some(children) = command.resolve_children() {
                for mut child in children {
                    if child.description.is_none() {
                        child.description = Some(command.name.clone())
                    }
                    candidates.push(child)
                }
            }
        }
    }
    let mut seen = HashSet::new();
    let mut results = candidates
        .into_iter()
        .enumerate()
        .filter(|(_, c)| seen.insert(c.id.clone()))
        .filter_map(|(catalog_index, command)| {
            score(&command, &tokens).map(|score| SearchResult {
                entry: command,
                score,
                catalog_index,
            })
        })
        .collect::<Vec<_>>();
    results.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then(a.catalog_index.cmp(&b.catalog_index))
    });
    results
}
fn score<M>(c: &Command<M>, tokens: &[String]) -> Option<u32> {
    if tokens.is_empty() {
        return Some(0);
    }
    let fields = [
        (c.name.as_str(), 400),
        (c.id.as_str(), 250),
        (c.description.as_deref().unwrap_or(""), 100),
        (c.group.as_deref().unwrap_or(""), 75),
    ];
    tokens.iter().try_fold(0, |total, t| {
        fields
            .iter()
            .filter_map(|(f, w)| match_score(&f.to_lowercase(), t).map(|v| v + w))
            .max()
            .map(|v| v + total)
    })
}
fn match_score(field: &str, token: &str) -> Option<u32> {
    if field == token {
        return Some(1000);
    }
    if field.starts_with(token) {
        return Some(700);
    }
    if field
        .split(|c: char| !c.is_alphanumeric())
        .any(|w| w.starts_with(token))
    {
        return Some(500);
    }
    if let Some(i) = field.find(token) {
        return Some(250_u32.saturating_sub(i.min(200) as u32));
    }
    fuzzy_score(field, token)
}
fn fuzzy_score(field: &str, token: &str) -> Option<u32> {
    let mut at = 0usize;
    let mut gaps = 0u32;
    for ch in token.chars() {
        let relative = field[at..].find(ch)?;
        gaps += relative as u32;
        at += relative + ch.len_utf8()
    }
    Some(100_u32.saturating_sub(gaps.min(99)))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn token_fuzzy_and_stability() {
        let xs = vec![
            Command::new("open-file", "Open File", || {}).group("File"),
            Command::new("close", "Close", || {}),
        ];
        assert_eq!(search_commands(&xs, "OF")[0].entry.id, "open-file");
        assert!(search_commands(&xs, "open missing").is_empty());
        assert_eq!(
            search_commands(&xs, "")
                .iter()
                .map(|x| x.catalog_index)
                .collect::<Vec<_>>(),
            [0, 1]
        );
    }
}
