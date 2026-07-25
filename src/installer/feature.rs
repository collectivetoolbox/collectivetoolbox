//! Shared feature tree types for installer components.
//!
//! This module provides a unified `Feature` type used by both the GUI and TUI
//! installers to represent installable features/components. Features can be
//! loaded from a `ReleaseManifest` or created programmatically.

#[allow(unused_imports, clippy::wildcard_imports, reason = "Standard workspace module prelude")]
use crate::utilities::*;

use std::collections::{HashMap, HashSet};

use crate::manifest::ReleaseManifest;

/// A feature/component that can be installed.
///
/// Features form a tree structure where each feature can have child features.
/// When a feature is selected, its size contributes to the total installation
/// size.
#[derive(Debug, Clone)]
pub struct Feature {
    /// Unique feature ID.
    pub id: String,
    /// Display name (may be localized).
    pub name: String,
    /// Whether this feature is selected for installation.
    pub selected: bool,
    /// Whether this feature is required (cannot be deselected).
    pub required: bool,
    /// Whether this feature is currently unavailable (cannot be selected).
    pub unavailable: bool,
    /// Total size in bytes when installed (sum of all files for this feature).
    pub size_bytes: u64,
    /// Child features.
    pub children: Vec<Feature>,
    /// IDs of features this depends on.
    pub depends_on: Vec<String>,
    /// Whether this node is expanded in a tree view (GUI only).
    pub expanded: bool,
}

impl Feature {
    /// Creates a new feature with the given ID, name, and size.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        size_bytes: u64,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            selected: true,
            required: false,
            unavailable: false,
            size_bytes,
            children: Vec::new(),
            depends_on: Vec::new(),
            expanded: true,
        }
    }

    /// Marks this feature as required.
    #[must_use]
    pub fn required(mut self) -> Self {
        self.required = true;
        self.selected = true;
        self
    }

    /// Marks this feature as unavailable.
    #[must_use]
    pub fn unavailable(mut self) -> Self {
        self.unavailable = true;
        self.selected = false;
        self
    }

    /// Adds a child feature.
    #[must_use]
    pub fn with_child(mut self, child: Feature) -> Self {
        self.children.push(child);
        self
    }

    /// Adds a dependency on another feature.
    #[must_use]
    pub fn depends_on(mut self, feature_id: impl Into<String>) -> Self {
        self.depends_on.push(feature_id.into());
        self
    }

    /// Calculates the total selected size including children.
    pub fn selected_size(&self) -> u64 {
        let mut total = if self.selected { self.size_bytes } else { 0 };
        for child in &self.children {
            total = total.saturating_add(child.selected_size());
        }
        total
    }

    /// Calculates the total size including all children (regardless of
    /// selection).
    pub fn total_size(&self) -> u64 {
        let mut total = self.size_bytes;
        for child in &self.children {
            total = total.saturating_add(child.total_size());
        }
        total
    }

    /// Collects all selected feature IDs into the given set.
    pub fn collect_selected(&self, out: &mut HashSet<String>) {
        if self.selected {
            out.insert(self.id.clone());
        }
        for child in &self.children {
            child.collect_selected(out);
        }
    }

    /// Recursively sets the selection state of this feature and all children.
    ///
    /// Required features are not affected.
    pub fn set_selection_recursive(&mut self, selected: bool) {
        if !self.required {
            self.selected = selected;
        }
        for child in &mut self.children {
            child.set_selection_recursive(selected);
        }
    }

    /// Counts the total number of features in this tree (including self).
    pub fn count_features(&self) -> usize {
        let mut count: usize = 1;
        for child in &self.children {
            count = count.saturating_add(child.count_features());
        }
        count
    }

    /// Counts the number of selected features in this tree (including self).
    pub fn count_selected(&self) -> usize {
        let mut count = usize::from(self.selected);
        for child in &self.children {
            count = count.saturating_add(child.count_selected());
        }
        count
    }
}

/// Placeholder feature tree for testing or when no manifest is available.
///
/// This creates a minimal feature tree with a single required "core" feature.
/// It serves as a fallback when the installer cannot load a manifest from
/// the server.
///
/// **Note:** This should be replaced with features loaded from a manifest
/// in production use. The placeholder exists to allow UI development and
/// testing without a running server.
pub fn placeholder_feature_tree() -> Vec<Feature> {
    vec![
        Feature::new(
            "core",
            "Core Application (placeholder - manifest not loaded)",
            15 * 1024 * 1024,
        )
        .required()
        .with_child(
            Feature::new("cli", "Command Line Interface", 2 * 1024 * 1024)
                .required(),
        ),
    ]
}

/// Converts a `ReleaseManifest` into a feature tree.
///
/// This groups files by their `feature_id` and builds a tree structure based
/// on the `requires` relationships between features. The size of each feature
/// is the sum of all file sizes for that feature.
///
/// # Arguments
/// * `manifest` - The release manifest to convert
/// * `lang_code` - The language code for localized feature names (e.g., "en")
#[allow(clippy::items_after_statements, reason = "helper helper structs defined after logic block")]
pub fn features_from_manifest(
    manifest: &ReleaseManifest,
    lang_code: &str,
) -> Vec<Feature> {
    // Group files by feature_id and collect feature info
    let mut feature_map: HashMap<String, FeatureBuilder> = HashMap::new();

    for file in &manifest.files {
        let builder = feature_map
            .entry(file.feature_id.clone())
            .or_insert_with(|| FeatureBuilder {
                id: file.feature_id.clone(),
                name: file
                    .get_feature_name(lang_code)
                    .unwrap_or(&file.feature_id)
                    .to_string(),
                size_bytes: 0,
                requires: file.requires.clone(),
                required: file.required,
                unavailable: file.unavailable,
            });

        builder.size_bytes = builder.size_bytes.saturating_add(file.file_size);
        // If any file in this feature is required, the feature is required
        if file.required {
            builder.required = true;
        }
        // If any file in this feature is unavailable, the feature is unavailable
        if file.unavailable {
            builder.unavailable = true;
        }
    }

    // Build the dependency graph and find root features (those with no parents)
    let mut children_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut has_parent: HashSet<String> = HashSet::new();

    for builder in feature_map.values() {
        for req in &builder.requires {
            children_map
                .entry(req.clone())
                .or_default()
                .push(builder.id.clone());
            has_parent.insert(builder.id.clone());
        }
    }

    // Build feature tree recursively
    fn build_feature(
        id: &str,
        feature_map: &HashMap<String, FeatureBuilder>,
        children_map: &HashMap<String, Vec<String>>,
    ) -> Option<Feature> {
        let builder = feature_map.get(id)?;

        let mut feature =
            Feature::new(&builder.id, &builder.name, builder.size_bytes);
        if builder.required {
            feature = feature.required();
        }
        if builder.unavailable {
            feature = feature.unavailable();
        }
        for req in &builder.requires {
            feature = feature.depends_on(req.clone());
        }

        // Add children
        if let Some(child_ids) = children_map.get(id) {
            for child_id in child_ids {
                if let Some(child) =
                    build_feature(child_id, feature_map, children_map)
                {
                    feature = feature.with_child(child);
                }
            }
        }

        Some(feature)
    }

    // Start with root features (those with no parents)
    let mut roots: Vec<Feature> = Vec::new();
    for id in feature_map.keys() {
        if !has_parent.contains(id) {
            if let Some(feature) =
                build_feature(id, &feature_map, &children_map)
            {
                roots.push(feature);
            }
        }
    }

    // If no roots found (possibly due to circular dependencies), just return
    // all features as roots
    if roots.is_empty() {
        roots = feature_map
            .values()
            .map(|b| {
                let mut f = Feature::new(&b.id, &b.name, b.size_bytes);
                if b.required {
                    f = f.required();
                }
                if b.unavailable {
                    f = f.unavailable();
                }
                f
            })
            .collect();
    }

    // Sort by name for consistent display
    roots.sort_by(|a, b| a.name.cmp(&b.name));

    roots
}

/// Helper struct for building features from manifest.
struct FeatureBuilder {
    id: String,
    name: String,
    size_bytes: u64,
    requires: Vec<String>,
    required: bool,
    unavailable: bool,
}

/// Toggles a feature by its 1-based index in a flat traversal.
///
/// Returns `true` if a feature was toggled, `false` if the index was not found
/// or the feature is required.
pub fn toggle_feature_by_index(
    feature: &mut Feature,
    target_index: usize,
    current: &mut usize,
) -> bool {
    if *current == target_index {
        if !feature.required {
            feature.selected = !feature.selected;
            return true;
        }
        return false;
    }
    *current = current.saturating_add(1);

    for child in &mut feature.children {
        if toggle_feature_by_index(child, target_index, current) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[crate::ctb_test]
    fn test_feature_size_calculation() {
        let feature = Feature::new("test", "Test", 1000)
            .with_child(Feature::new("child1", "Child 1", 500))
            .with_child(Feature::new("child2", "Child 2", 300));

        assert_eq!(feature.selected_size(), 1800);
        assert_eq!(feature.total_size(), 1800);
    }

    #[crate::ctb_test]
    fn test_feature_selection() {
        let mut feature = Feature::new("parent", "Parent", 100)
            .with_child(Feature::new("child1", "Child 1", 50))
            .with_child(Feature::new("child2", "Child 2", 50).required());

        feature.set_selection_recursive(false);

        assert!(!feature.selected);
        assert!(!feature.children[0].selected);
        // Required child should still be selected
        assert!(feature.children[1].selected);
    }

    #[crate::ctb_test]
    fn test_collect_selected() {
        let mut feature = Feature::new("parent", "Parent", 100)
            .with_child(Feature::new("child1", "Child 1", 50));

        feature.children[0].selected = false;

        let mut selected = HashSet::new();
        feature.collect_selected(&mut selected);

        assert!(selected.contains("parent"));
        assert!(!selected.contains("child1"));
    }

    #[crate::ctb_test]
    fn test_toggle_by_index() {
        let mut features = vec![
            Feature::new("a", "A", 100)
                .with_child(Feature::new("a1", "A1", 50))
                .with_child(Feature::new("a2", "A2", 50)),
            Feature::new("b", "B", 100),
        ];

        // Toggle feature at index 2 (a1)
        let mut idx = 1;
        toggle_feature_by_index(&mut features[0], 2, &mut idx);

        assert!(!features[0].children[0].selected);
    }

    #[crate::ctb_test]
    fn test_placeholder_tree() {
        let tree = placeholder_feature_tree();
        assert!(!tree.is_empty());
        assert!(tree[0].required);
        assert!(tree[0].name.contains("placeholder"));
    }
}
