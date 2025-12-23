use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use std::borrow::Cow;
use tui_prompts::State as _;
use tui_prompts::TextState;

/// Data that can be selected in a picker
#[derive(Debug, Clone, PartialEq)]
pub enum PickerData {
    /// Revision (branch, commit, reference, etc.)
    Revision(String),
    /// Remote name
    Remote(String),
    /// Custom user input (literal text from input field)
    CustomInput(String),
}

impl PickerData {
    /// Get the display string for this data
    pub fn display(&self) -> &str {
        match self {
            PickerData::Revision(s) => s,
            PickerData::Remote(s) => s,
            PickerData::CustomInput(s) => s,
        }
    }
}

/// An item in the picker list
#[derive(Debug, Clone, PartialEq)]
pub struct PickerItem {
    /// The text to display and match against
    pub display: Cow<'static, str>,
    /// Associated data
    pub data: PickerData,
}

impl PickerItem {
    pub fn new(display: impl Into<Cow<'static, str>>, data: PickerData) -> Self {
        Self {
            display: display.into(),
            data,
        }
    }
}

/// Result of a fuzzy match with score
#[derive(Debug, Clone)]
struct MatchResult {
    index: usize,
    score: i64,
}

/// Picker status
#[derive(Debug, Clone, PartialEq)]
pub enum PickerStatus {
    /// Picker is active
    Active,
    /// User selected an item
    Done,
    /// User cancelled
    Cancelled,
}

/// State of the picker component
pub struct PickerState {
    /// All available items (excluding custom input)
    items: Vec<PickerItem>,
    /// Filtered and sorted indices based on current pattern
    filtered_indices: Vec<usize>,
    /// Current cursor position in filtered results
    cursor: usize,
    /// Current input pattern
    pub input_state: TextState<'static>,
    /// Fuzzy matcher
    matcher: SkimMatcherV2,
    /// Prompt text to display
    pub prompt_text: Cow<'static, str>,
    /// Current status
    status: PickerStatus,
    /// Allow user to input custom value not in the list
    allow_custom_input: bool,
    /// Custom input item (separate from items list)
    custom_input_item: Option<PickerItem>,
}

impl PickerState {
    /// Create a new picker with items
    pub fn new(
        prompt: impl Into<Cow<'static, str>>,
        items: Vec<PickerItem>,
        allow_custom_input: bool,
    ) -> Self {
        let mut state = Self {
            items: items.clone(),
            filtered_indices: Vec::new(),
            cursor: 0,
            input_state: TextState::default(),
            matcher: SkimMatcherV2::default(),
            prompt_text: prompt.into(),
            status: PickerStatus::Active,
            allow_custom_input,
            custom_input_item: None,
        };
        state.update_filter();
        state
    }

    /// Get current input pattern
    pub fn pattern(&self) -> &str {
        self.input_state.value()
    }

    /// Update the filter based on current input pattern
    pub fn update_filter(&mut self) {
        let pattern = self.pattern().to_string();

        if pattern.is_empty() {
            // Show all items when no pattern
            self.filtered_indices = (0..self.items.len()).collect();
            self.custom_input_item = None;
        } else {
            // Fuzzy match and sort by score
            let mut matches: Vec<MatchResult> = self
                .items
                .iter()
                .enumerate()
                .filter_map(|(i, item)| {
                    self.matcher
                        .fuzzy_match(&item.display, &pattern)
                        .map(|score| MatchResult { index: i, score })
                })
                .collect();

            // Sort by score (higher is better)
            matches.sort_by(|a, b| b.score.cmp(&a.score));

            self.filtered_indices = matches.into_iter().map(|m| m.index).collect();

            // Create custom input item if enabled
            if self.allow_custom_input {
                self.custom_input_item = Some(PickerItem::new(
                    pattern.clone(),
                    PickerData::CustomInput(pattern),
                ));
            } else {
                self.custom_input_item = None;
            }
        }

        // Reset cursor if out of bounds
        let total_count = self.filtered_indices.len()
            + if self.custom_input_item.is_some() {
                1
            } else {
                0
            };
        if self.cursor >= total_count {
            self.cursor = 0;
        }
    }

    /// Get the currently selected item, if any
    pub fn selected(&self) -> Option<&PickerItem> {
        // Check if cursor is on custom input item (always at the end)
        if self.cursor == self.filtered_indices.len() {
            return self.custom_input_item.as_ref();
        }

        self.filtered_indices
            .get(self.cursor)
            .and_then(|&i| self.items.get(i))
    }

    /// Get all filtered items with their original indices
    /// Custom input item (if present) is always last with index usize::MAX
    pub fn filtered_items(&self) -> impl Iterator<Item = (usize, &PickerItem)> {
        self.filtered_indices
            .iter()
            .filter_map(|&i| self.items.get(i).map(|item| (i, item)))
            .chain(
                self.custom_input_item
                    .as_ref()
                    .map(|item| (usize::MAX, item)),
            )
    }

    /// Move cursor to next item
    pub fn next(&mut self) {
        let total_count = self.filtered_indices.len()
            + if self.custom_input_item.is_some() {
                1
            } else {
                0
            };
        if total_count > 0 {
            self.cursor = (self.cursor + 1) % total_count;
        }
    }

    /// Move cursor to previous item
    pub fn previous(&mut self) {
        let total_count = self.filtered_indices.len()
            + if self.custom_input_item.is_some() {
                1
            } else {
                0
            };
        if total_count > 0 {
            self.cursor = if self.cursor == 0 {
                total_count - 1
            } else {
                self.cursor - 1
            };
        }
    }

    /// Get current cursor position
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Get total number of items
    pub fn total_items(&self) -> usize {
        self.items.len()
    }

    /// Get number of filtered items
    pub fn filtered_count(&self) -> usize {
        self.filtered_indices.len()
    }

    /// Get the fuzzy match positions for a given item (for highlighting)
    pub fn match_indices(&self, item_index: usize) -> Option<Vec<usize>> {
        let pattern = self.pattern();
        if pattern.is_empty() {
            return None;
        }

        // Don't highlight custom input items (marked with usize::MAX)
        if item_index == usize::MAX {
            return None;
        }

        self.items
            .get(item_index)
            .and_then(|item| self.matcher.fuzzy_indices(&item.display, pattern))
            .map(|(_, indices)| indices)
    }

    /// Get current status
    pub fn status(&self) -> &PickerStatus {
        &self.status
    }

    /// Mark picker as done (user selected an item)
    pub fn done(&mut self) {
        self.status = PickerStatus::Done;
    }

    /// Mark picker as cancelled (user cancelled)
    pub fn cancel(&mut self) {
        self.status = PickerStatus::Cancelled;
    }

    /// Check if picker is done
    pub fn is_done(&self) -> bool {
        self.status == PickerStatus::Done
    }

    /// Check if picker is cancelled
    pub fn is_cancelled(&self) -> bool {
        self.status == PickerStatus::Cancelled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn create_test_items() -> Vec<PickerItem> {
        vec![
            PickerItem::new("main", PickerData::Revision("main".to_string())),
            PickerItem::new("develop", PickerData::Revision("develop".to_string())),
            PickerItem::new(
                "feature/test",
                PickerData::Revision("feature/test".to_string()),
            ),
            PickerItem::new(
                "feature/new",
                PickerData::Revision("feature/new".to_string()),
            ),
            PickerItem::new("bugfix/123", PickerData::Revision("bugfix/123".to_string())),
        ]
    }

    #[test]
    fn test_picker_data_display() {
        let revision = PickerData::Revision("main".to_string());
        assert_eq!(revision.display(), "main");

        let custom = PickerData::CustomInput("custom".to_string());
        assert_eq!(custom.display(), "custom");
    }

    #[test]
    fn test_picker_item_new() {
        let item = PickerItem::new("main", PickerData::Revision("main".to_string()));
        assert_eq!(item.display.as_ref(), "main");
        assert_eq!(item.data.display(), "main");
    }

    #[test]
    fn test_picker_state_new_without_custom_input() {
        let items = create_test_items();
        let state = PickerState::new("Select branch", items.clone(), false);

        assert_eq!(state.prompt_text.as_ref(), "Select branch");
        assert_eq!(state.total_items(), 5);
        assert_eq!(state.filtered_count(), 5);
        assert_eq!(state.cursor(), 0);
        assert_eq!(state.pattern(), "");
        assert_eq!(state.status(), &PickerStatus::Active);
        assert!(!state.allow_custom_input);
    }

    #[test]
    fn test_picker_state_new_with_custom_input() {
        let items = create_test_items();
        let state = PickerState::new("Select branch", items, true);

        assert!(state.allow_custom_input);
        assert_eq!(state.custom_input_item, None); // No custom item when pattern is empty
    }

    #[test]
    fn test_empty_pattern_shows_all_items() {
        let items = create_test_items();
        let state = PickerState::new("Select", items, false);

        assert_eq!(state.filtered_count(), 5);
        assert_eq!(state.filtered_indices, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_fuzzy_filtering() {
        let items = create_test_items();
        let mut state = PickerState::new("Select", items, false);

        state
            .input_state
            .handle_key_event(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::empty()));
        state.update_filter();

        // Should match "feature/test", "feature/new", and "bugfix/123" (fuzzy match)
        assert_eq!(state.filtered_count(), 3);

        let filtered: Vec<_> = state.filtered_items().collect();
        assert!(
            filtered
                .iter()
                .any(|(_, item)| item.display == "feature/test")
        );
        assert!(
            filtered
                .iter()
                .any(|(_, item)| item.display == "feature/new")
        );
        assert!(
            filtered
                .iter()
                .any(|(_, item)| item.display == "bugfix/123")
        );
    }

    #[test]
    fn test_fuzzy_filtering_sorts_by_score() {
        // Create items with varying match quality for pattern "feat"
        let items = vec![
            PickerItem::new("feat", PickerData::Revision("feat".to_string())), // Exact match
            PickerItem::new("feature", PickerData::Revision("feature".to_string())), // Prefix match
            PickerItem::new(
                "feature/test",
                PickerData::Revision("feature/test".to_string()),
            ), // Prefix with more chars
            PickerItem::new(
                "fix-eat-bug",
                PickerData::Revision("fix-eat-bug".to_string()),
            ), // Scattered match
        ];
        let mut state = PickerState::new("Select", items, false);

        // Type "feat" pattern
        state
            .input_state
            .handle_key_event(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::empty()));
        state
            .input_state
            .handle_key_event(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::empty()));
        state
            .input_state
            .handle_key_event(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()));
        state
            .input_state
            .handle_key_event(KeyEvent::new(KeyCode::Char('t'), KeyModifiers::empty()));
        state.update_filter();

        // All should match
        assert_eq!(state.filtered_count(), 4);

        // Verify sorted by score: exact match > prefix match > scattered match
        let filtered: Vec<_> = state
            .filtered_items()
            .map(|(_, item)| item.display.as_ref())
            .collect();

        // "feat" (exact) should be first, "fix-eat-bug" (scattered) should be last
        assert_eq!(filtered[0], "feat");
        assert_eq!(filtered[filtered.len() - 1], "fix-eat-bug");
    }

    #[test]
    fn test_case_insensitive_matching() {
        let items = vec![
            PickerItem::new("Feature", PickerData::Revision("Feature".to_string())),
            PickerItem::new("feature", PickerData::Revision("feature".to_string())),
            PickerItem::new("FEATURE", PickerData::Revision("FEATURE".to_string())),
        ];

        let mut state = PickerState::new("Select item", items, false);

        state
            .input_state
            .handle_key_event(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::empty()));
        state
            .input_state
            .handle_key_event(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::empty()));
        state
            .input_state
            .handle_key_event(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()));
        state.update_filter();

        // fuzzy-matcher is case-insensitive by default
        assert_eq!(state.filtered_count(), 3);
    }

    #[test]
    fn test_no_matches() {
        let items = create_test_items();
        let mut state = PickerState::new("Select", items, false);

        state
            .input_state
            .handle_key_event(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::empty()));
        state
            .input_state
            .handle_key_event(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::empty()));
        state
            .input_state
            .handle_key_event(KeyEvent::new(KeyCode::Char('z'), KeyModifiers::empty()));
        state.update_filter();

        assert_eq!(state.filtered_count(), 0);
        assert!(state.selected().is_none());
    }

    #[test]
    fn test_custom_input_creation() {
        let items = create_test_items();
        let mut state = PickerState::new("Select", items, true);

        // Use pattern "fea" which matches feature/test and feature/new
        state
            .input_state
            .handle_key_event(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::empty()));
        state
            .input_state
            .handle_key_event(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::empty()));
        state
            .input_state
            .handle_key_event(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()));
        state.update_filter();

        // Custom input item should be created
        assert!(state.custom_input_item.is_some());

        // Should have multiple regular matches + custom input
        let filtered: Vec<_> = state.filtered_items().collect();
        assert!(filtered.len() >= 3); // At least 2 feature items + bugfix + custom input

        // Custom input should be last in filtered items
        let last = filtered.last().unwrap();
        assert_eq!(last.0, usize::MAX);
        assert_eq!(last.1.display.as_ref(), "fea");
    }

    #[test]
    fn test_custom_input_not_created_when_disabled() {
        let items = create_test_items();
        let mut state = PickerState::new("Select", items, false);

        state
            .input_state
            .handle_key_event(KeyEvent::new(KeyCode::Char('t'), KeyModifiers::empty()));
        state
            .input_state
            .handle_key_event(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::empty()));
        state.update_filter();

        assert!(state.custom_input_item.is_none());
    }

    #[test]
    fn test_custom_input_not_created_on_empty_pattern() {
        let items = create_test_items();
        let mut state = PickerState::new("Select", items, true);

        // Start with no input
        assert!(state.custom_input_item.is_none());

        // Add then remove input
        state
            .input_state
            .handle_key_event(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()));
        state.update_filter();
        assert!(state.custom_input_item.is_some());

        state
            .input_state
            .handle_key_event(KeyEvent::new(KeyCode::Backspace, KeyModifiers::empty()));
        state.update_filter();
        assert!(state.custom_input_item.is_none());
    }

    #[test]
    fn test_cursor_next() {
        let items = create_test_items();
        let mut state = PickerState::new("Select", items, false);

        assert_eq!(state.cursor(), 0);

        state.next();
        assert_eq!(state.cursor(), 1);

        state.next();
        assert_eq!(state.cursor(), 2);
    }

    #[test]
    fn test_cursor_next_wraps_around() {
        let items = create_test_items();
        let mut state = PickerState::new("Select", items, false);

        // Move to end
        for _ in 0..5 {
            state.next();
        }

        // Should wrap to 0
        assert_eq!(state.cursor(), 0);
    }

    #[test]
    fn test_cursor_previous() {
        let items = create_test_items();
        let mut state = PickerState::new("Select", items, false);

        state.next();
        state.next();
        assert_eq!(state.cursor(), 2);

        state.previous();
        assert_eq!(state.cursor(), 1);

        state.previous();
        assert_eq!(state.cursor(), 0);
    }

    #[test]
    fn test_cursor_previous_wraps_around() {
        let items = create_test_items();
        let mut state = PickerState::new("Select", items, false);

        assert_eq!(state.cursor(), 0);

        state.previous();
        // Should wrap to last item
        assert_eq!(state.cursor(), 4);
    }

    #[test]
    fn test_cursor_with_custom_input() {
        let items = create_test_items();
        let mut state = PickerState::new("Select", items, true);

        state
            .input_state
            .handle_key_event(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::empty()));
        state.update_filter();

        // 3 matched items (feature/test, feature/new, bugfix/123) + 1 custom input = 4 total
        assert_eq!(state.cursor(), 0);

        state.next();
        assert_eq!(state.cursor(), 1);

        state.next();
        assert_eq!(state.cursor(), 2);

        state.next();
        assert_eq!(state.cursor(), 3); // Custom input position

        state.next();
        assert_eq!(state.cursor(), 0); // Wrapped around forward

        state.previous();
        assert_eq!(state.cursor(), 3); // Wrapped around backward to custom input
    }

    #[test]
    fn test_cursor_resets_when_filter_reduces_items() {
        let items = create_test_items();
        let mut state = PickerState::new("Select", items, false);

        // Move cursor to position 4
        for _ in 0..4 {
            state.next();
        }
        assert_eq!(state.cursor(), 4);

        // Filter to only 2 items
        state
            .input_state
            .handle_key_event(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::empty()));
        state.update_filter();

        // Cursor should reset to 0
        assert_eq!(state.cursor(), 0);
    }

    #[test]
    fn test_selected_returns_correct_item() {
        let items = create_test_items();
        let mut state = PickerState::new("Select", items, false);

        let selected = state.selected().unwrap();
        assert_eq!(selected.display.as_ref(), "main");

        state.next();
        state.next();
        let selected = state.selected().unwrap();
        assert_eq!(selected.display.as_ref(), "feature/test");
    }

    #[test]
    fn test_selected_with_filter() {
        let items = create_test_items();
        let mut state = PickerState::new("Select", items, false);

        state
            .input_state
            .handle_key_event(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::empty()));
        state.update_filter();

        // Should match "bugfix/123"
        let selected = state.selected().unwrap();
        assert_eq!(selected.display.as_ref(), "bugfix/123");
    }

    #[test]
    fn test_selected_returns_custom_input() {
        let items = create_test_items();
        let mut state = PickerState::new("Select", items, true);

        // Use a pattern that doesn't match any items
        state
            .input_state
            .handle_key_event(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty()));
        state
            .input_state
            .handle_key_event(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty()));
        state.update_filter();

        // No matches, only custom input at cursor 0
        assert_eq!(state.cursor(), 0);
        assert_eq!(state.selected().unwrap().display.as_ref(), "qq");
    }

    #[test]
    fn test_filtered_items_order() {
        let items = create_test_items();
        let mut state = PickerState::new("Select", items, true);

        state
            .input_state
            .handle_key_event(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::empty()));
        state.update_filter();

        let indices: Vec<_> = state.filtered_items().map(|(idx, _)| idx).collect();

        // Verify normal item indices followed by custom input
        assert_eq!(indices[0], 2); // feature/test
        assert_eq!(indices[1], 3); // feature/new
        assert_eq!(indices[2], 4); // bugfix/123
        assert_eq!(indices[3], usize::MAX); // custom input
    }

    #[test]
    fn test_match_indices_with_pattern() {
        let items = create_test_items();
        let mut state = PickerState::new("Select", items, false);

        state
            .input_state
            .handle_key_event(KeyEvent::new(KeyCode::Char('m'), KeyModifiers::empty()));
        state
            .input_state
            .handle_key_event(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()));
        state
            .input_state
            .handle_key_event(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::empty()));
        state.update_filter();

        // Get the index of "main" in original items
        let indices = state.match_indices(0);
        assert!(indices.is_some());

        let indices = indices.unwrap();
        assert_eq!(indices.len(), 3); // 'm', 'a', 'i' should match
    }

    #[test]
    fn test_match_indices_empty_pattern() {
        let items = create_test_items();
        let state = PickerState::new("Select", items, false);

        let indices = state.match_indices(0);
        assert!(indices.is_none());
    }

    #[test]
    fn test_match_indices_custom_input_returns_none() {
        let items = create_test_items();
        let mut state = PickerState::new("Select", items, true);

        state
            .input_state
            .handle_key_event(KeyEvent::new(KeyCode::Char('t'), KeyModifiers::empty()));
        state.update_filter();

        // usize::MAX is used for custom input items
        let indices = state.match_indices(usize::MAX);
        assert!(indices.is_none());
    }

    #[test]
    fn test_status_transitions() {
        let items = create_test_items();
        let mut state = PickerState::new("Select", items, false);

        assert_eq!(state.status(), &PickerStatus::Active);
        assert!(!state.is_done());
        assert!(!state.is_cancelled());

        state.done();
        assert_eq!(state.status(), &PickerStatus::Done);
        assert!(state.is_done());
        assert!(!state.is_cancelled());
    }

    #[test]
    fn test_status_cancelled() {
        let items = create_test_items();
        let mut state = PickerState::new("Select", items, false);

        state.cancel();
        assert_eq!(state.status(), &PickerStatus::Cancelled);
        assert!(!state.is_done());
        assert!(state.is_cancelled());
    }

    #[test]
    fn test_empty_items_list() {
        let state = PickerState::new("Select", vec![], false);

        assert_eq!(state.total_items(), 0);
        assert_eq!(state.filtered_count(), 0);
        assert_eq!(state.cursor(), 0);
        assert!(state.selected().is_none());
    }

    #[test]
    fn test_empty_items_with_custom_input() {
        let mut state = PickerState::new("Select", vec![], true);

        state
            .input_state
            .handle_key_event(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty()));
        state.update_filter();

        assert_eq!(state.total_items(), 0);
        assert_eq!(state.filtered_count(), 0);
        assert!(state.custom_input_item.is_some());

        assert_eq!(state.selected().unwrap().display.as_ref(), "a");
    }

    #[test]
    fn test_cursor_navigation_empty_list() {
        let mut state = PickerState::new("Select", vec![], false);

        state.next();
        assert_eq!(state.cursor(), 0);

        state.previous();
        assert_eq!(state.cursor(), 0);
    }

    #[test]
    fn test_single_item_navigation() {
        let items = vec![PickerItem::new(
            "only",
            PickerData::Revision("only".to_string()),
        )];
        let mut state = PickerState::new("Select", items, false);

        assert_eq!(state.cursor(), 0);

        state.next();
        assert_eq!(state.cursor(), 0); // Wraps to same item

        state.previous();
        assert_eq!(state.cursor(), 0); // Wraps to same item
    }
}
