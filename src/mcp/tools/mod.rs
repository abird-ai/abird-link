//! Dotlink-owned tool projections around the shared canonical filesystem core.
//! Filesystem semantics live in `aw-tools`; this module retains only product
//! transport helpers plus shell/job control surfaces.

#[cfg(test)]
pub(crate) mod differential;
pub(crate) mod graph;
pub(crate) mod jobs;
pub(crate) mod shared_native;
pub(crate) mod shell;
#[cfg(test)]
pub(crate) mod test_support;

use rmcp::model::{CallToolResult, ContentBlock};

use super::LocalMachine;

fn dot() -> String {
    ".".to_owned()
}

fn ok_text(text: impl Into<String>) -> CallToolResult {
    CallToolResult::success(vec![ContentBlock::text(text.into())])
}

fn tool_error(error: impl std::fmt::Display) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(error.to_string())])
}

/// Client-controlled value under a host-selected policy ceiling.
pub(crate) fn resolve_capped<T>(requested: Option<T>, ceiling: T) -> T
where
    T: Ord + From<u8> + Copy,
{
    let ceiling = ceiling.max(T::from(1));
    requested.unwrap_or(ceiling).clamp(T::from(1), ceiling)
}

#[cfg(test)]
mod tests {
    use super::resolve_capped;

    #[test]
    fn capped_values_keep_host_floor_and_ceiling() {
        assert_eq!(resolve_capped::<usize>(None, 8), 8);
        assert_eq!(resolve_capped(Some(3_usize), 8), 3);
        assert_eq!(resolve_capped(Some(99_usize), 8), 8);
        assert_eq!(resolve_capped(Some(0_usize), 8), 1);
    }
}
