use crate::picker::{PickerData, PickerItem, PickerState};
use tui_prompts::State as _;

#[test]
fn picker_basic() {
    let items = vec![
        PickerItem::new(
            "feature/auth",
            PickerData::Revision("feature/auth".to_string()),
        ),
        PickerItem::new("feature/ui", PickerData::Revision("feature/ui".to_string())),
        PickerItem::new("main", PickerData::Revision("main".to_string())),
        PickerItem::new("develop", PickerData::Revision("develop".to_string())),
    ];

    let picker = PickerState::new("Select branch", items, false);

    assert_eq!(picker.total_items(), 4);
    assert_eq!(picker.filtered_count(), 4);
    assert_eq!(picker.cursor(), 0);
}

#[test]
fn picker_fuzzy_match() {
    let items = vec![
        PickerItem::new(
            "feature/auth",
            PickerData::Revision("feature/auth".to_string()),
        ),
        PickerItem::new("feature/ui", PickerData::Revision("feature/ui".to_string())),
        PickerItem::new("main", PickerData::Revision("main".to_string())),
        PickerItem::new("develop", PickerData::Revision("develop".to_string())),
    ];

    let mut picker = PickerState::new("Select branch", items, false);

    // Simulate typing "fea"
    picker.input_state.value_mut().push_str("fea");
    picker.update_filter();

    // Should match "feature/auth" and "feature/ui"
    assert_eq!(picker.filtered_count(), 2);
}

#[test]
fn picker_navigation() {
    let items = vec![
        PickerItem::new("item1", PickerData::Revision("item1".to_string())),
        PickerItem::new("item2", PickerData::Revision("item2".to_string())),
        PickerItem::new("item3", PickerData::Revision("item3".to_string())),
    ];

    let mut picker = PickerState::new("Select item", items, false);

    assert_eq!(picker.cursor(), 0);

    picker.next();
    assert_eq!(picker.cursor(), 1);

    picker.next();
    assert_eq!(picker.cursor(), 2);

    // Wrap around
    picker.next();
    assert_eq!(picker.cursor(), 0);

    // Go back
    picker.previous();
    assert_eq!(picker.cursor(), 2);
}

#[test]
fn picker_selection() {
    let items = vec![
        PickerItem::new("item1", PickerData::Revision("item1".to_string())),
        PickerItem::new("item2", PickerData::Revision("item2".to_string())),
    ];

    let mut picker = PickerState::new("Select item", items, false);

    let selected = picker.selected().unwrap();
    assert_eq!(selected.display, "item1");

    picker.next();
    let selected = picker.selected().unwrap();
    assert_eq!(selected.display, "item2");
}

#[test]
fn picker_empty_pattern() {
    let items = vec![
        PickerItem::new("item1", PickerData::Revision("item1".to_string())),
        PickerItem::new("item2", PickerData::Revision("item2".to_string())),
    ];

    let picker = PickerState::new("Select item", items, false);

    // Empty pattern should show all items
    assert_eq!(picker.pattern(), "");
    assert_eq!(picker.filtered_count(), 2);
}

#[test]
fn picker_no_matches() {
    let items = vec![
        PickerItem::new("item1", PickerData::Revision("item1".to_string())),
        PickerItem::new("item2", PickerData::Revision("item2".to_string())),
    ];

    let mut picker = PickerState::new("Select item", items, false);

    picker.input_state.value_mut().push_str("xyz");
    picker.update_filter();

    assert_eq!(picker.filtered_count(), 0);
    assert!(picker.selected().is_none());
}

#[test]
fn picker_case_insensitive() {
    let items = vec![
        PickerItem::new("Feature", PickerData::Revision("Feature".to_string())),
        PickerItem::new("feature", PickerData::Revision("feature".to_string())),
        PickerItem::new("FEATURE", PickerData::Revision("FEATURE".to_string())),
    ];

    let mut picker = PickerState::new("Select item", items, false);

    picker.input_state.value_mut().push_str("fea");
    picker.update_filter();

    // fuzzy-matcher is case-insensitive by default
    assert_eq!(picker.filtered_count(), 3);
}

#[test]
fn picker_custom_input_disabled() {
    let items = vec![
        PickerItem::new("item1", PickerData::Revision("item1".to_string())),
        PickerItem::new("item2", PickerData::Revision("item2".to_string())),
    ];

    let mut picker = PickerState::new("Select item", items, false);

    picker.input_state.value_mut().push_str("custom");
    picker.update_filter();

    // Should have no matches, and no custom input item
    assert_eq!(picker.filtered_count(), 0);
}

#[test]
fn picker_custom_input_enabled() {
    let items = vec![
        PickerItem::new("item1", PickerData::Revision("item1".to_string())),
        PickerItem::new("item2", PickerData::Revision("item2".to_string())),
    ];

    let mut picker = PickerState::new("Select item", items, true);

    picker.input_state.value_mut().push_str("custom");
    picker.update_filter();

    // Should have 0 filtered items (no matches)
    assert_eq!(picker.filtered_count(), 0);

    let selected = picker.selected().unwrap();
    assert_eq!(selected.display, "custom");
    match &selected.data {
        PickerData::CustomInput(s) => assert_eq!(s, "custom"),
        _ => panic!("Expected CustomInput"),
    }
}

#[test]
fn picker_custom_input_with_matches() {
    let items = vec![
        PickerItem::new(
            "feature/auth",
            PickerData::Revision("feature/auth".to_string()),
        ),
        PickerItem::new("feature/ui", PickerData::Revision("feature/ui".to_string())),
    ];

    let mut picker = PickerState::new("Select branch", items, true);

    picker.input_state.value_mut().push_str("feat");
    picker.update_filter();

    // Should have 2 filtered items (custom input not counted)
    assert_eq!(picker.filtered_count(), 2);

    // Navigate to last item (custom input)
    picker.next();
    picker.next();

    let selected = picker.selected().unwrap();
    assert_eq!(selected.display, "feat");
    match &selected.data {
        PickerData::CustomInput(s) => assert_eq!(s, "feat"),
        _ => panic!("Expected CustomInput"),
    }
}

#[test]
fn picker_custom_input_empty_pattern() {
    let items = vec![PickerItem::new(
        "item1",
        PickerData::Revision("item1".to_string()),
    )];

    let picker = PickerState::new("Select item", items, true);

    // Empty pattern should not add custom input
    assert_eq!(picker.pattern(), "");
    assert_eq!(picker.filtered_count(), 1);

    let selected = picker.selected().unwrap();
    match &selected.data {
        PickerData::Revision(_) => {}
        _ => panic!("Expected Revision, not CustomInput"),
    }
}
