//! Manifest tracking system integration tests
//!
//! These tests verify the complete workflow of the manifest tracking system,
//! including tracker, reporter, and barrel file updates.

use std::fs;
use swagger_gen::manifest::{Manifest, ManifestTracker, generate_reports, update_manifest};
use swagger_gen::pipeline::{force_update_barrel, update_barrel_with_parents};
use tempfile::TempDir;

// ==================== Complete Workflow Tests ====================

#[test]
fn full_workflow_track_and_report() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path();
    let manifest_path = output_dir.join(".generated").join("manifest.json");

    // Step 1: Create tracker and track files
    let mut tracker = ManifestTracker::new("models");
    tracker.track("User", "User.ts");
    tracker.track("Post", "Post.ts");

    // Step 2: Calculate diff
    let entries = tracker.entries().clone();
    let diff = tracker.finish(&manifest_path);

    // Step 3: Generate reports
    let report_result = generate_reports(&diff, output_dir, ".generated");
    assert!(report_result.is_ok());

    // Step 4: Update manifest
    let update_result = update_manifest(
        &manifest_path,
        "models".to_string(),
        entries,
        "hash123",
        "3.0.0",
    );
    assert!(update_result.is_ok());

    // Verify manifest was created
    assert!(manifest_path.exists());

    // Verify reports were created
    assert!(output_dir.join(".generated/deletion-report.json").exists());
    assert!(output_dir.join(".generated/deletion-report.md").exists());
}

#[test]
fn manifest_persists_across_runs() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join(".generated").join("manifest.json");

    // First run: Create initial manifest
    let mut tracker1 = ManifestTracker::new("models");
    tracker1.track("User", "User.ts");
    tracker1.track("Post", "Post.ts");

    let entries1 = tracker1.entries().clone();
    let diff1 = tracker1.finish(&manifest_path);
    assert_eq!(diff1.added.len(), 2); // Both should be new

    update_manifest(&manifest_path, "models".to_string(), entries1, "", "").unwrap();

    // Second run: Load existing manifest and verify persistence
    let mut tracker2 = ManifestTracker::new("models");
    tracker2.track("User", "User.ts");
    tracker2.track("Post", "Post.ts");
    tracker2.track("Comment", "Comment.ts"); // New file

    let entries2 = tracker2.entries().clone();
    let diff2 = tracker2.finish(&manifest_path);

    assert_eq!(diff2.added.len(), 1); // Only Comment is new
    assert_eq!(diff2.unchanged.len(), 2); // User and Post are unchanged
    assert_eq!(diff2.deleted.len(), 0);

    update_manifest(&manifest_path, "models".to_string(), entries2, "", "").unwrap();

    // Verify manifest contains all three entries
    let manifest = Manifest::load(&manifest_path).unwrap();
    let model_entries = manifest.get_generator_entries("models").unwrap();
    assert_eq!(model_entries.len(), 3);
}

#[test]
fn multiple_generators_coexist() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join(".generated").join("manifest.json");

    // Run models generator
    let mut models_tracker = ManifestTracker::new("models");
    models_tracker.track("User", "User.ts");
    let models_entries = models_tracker.entries().clone();
    models_tracker.finish(&manifest_path);
    update_manifest(&manifest_path, "models".to_string(), models_entries, "", "").unwrap();

    // Run functions generator
    let mut functions_tracker = ManifestTracker::new("functions");
    functions_tracker.track("getUser", "getUser.ts");
    let functions_entries = functions_tracker.entries().clone();
    functions_tracker.finish(&manifest_path);
    update_manifest(
        &manifest_path,
        "functions".to_string(),
        functions_entries,
        "",
        "",
    )
    .unwrap();

    // Verify both generators exist in manifest
    let manifest = Manifest::load(&manifest_path).unwrap();
    assert!(manifest.get_generator_entries("models").is_some());
    assert!(manifest.get_generator_entries("functions").is_some());
}

#[test]
fn incremental_generation_detects_changes() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join(".generated").join("manifest.json");

    // First generation
    let mut tracker1 = ManifestTracker::new("models");
    tracker1.track("User", "User.ts");
    tracker1.track("Post", "Post.ts");
    tracker1.track("Comment", "Comment.ts");
    let entries1 = tracker1.entries().clone();
    tracker1.finish(&manifest_path);
    update_manifest(&manifest_path, "models".to_string(), entries1, "", "").unwrap();

    // Second generation: Remove Post, keep User, add Author
    let mut tracker2 = ManifestTracker::new("models");
    tracker2.track("User", "User.ts");
    tracker2.track("Author", "Author.ts"); // New
    // Comment is unchanged
    tracker2.track("Comment", "Comment.ts");
    let diff = tracker2.finish(&manifest_path);

    // Verify diff
    assert_eq!(diff.added.len(), 1);
    assert!(diff.added.iter().any(|(n, _)| n == "Author"));
    assert_eq!(diff.deleted.len(), 1);
    assert!(diff.deleted.iter().any(|(n, _)| n == "Post"));
    assert_eq!(diff.unchanged.len(), 2);

    // Generate reports and verify
    let report = generate_reports(&diff, temp_dir.path(), ".generated").unwrap();
    assert_eq!(report.summary.added_count, 1);
    assert_eq!(report.summary.deleted_count, 1);
    assert_eq!(report.summary.unchanged_count, 2);
}

#[test]
fn report_generation_after_tracker_finish() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join(".generated").join("manifest.json");

    // Create some initial state
    let mut tracker1 = ManifestTracker::new("models");
    tracker1.track("OldModel", "OldModel.ts");
    let entries1 = tracker1.entries().clone();
    tracker1.finish(&manifest_path);
    update_manifest(&manifest_path, "models".to_string(), entries1, "", "").unwrap();

    // Generate new state with changes
    let mut tracker2 = ManifestTracker::new("models");
    tracker2.track("NewModel", "NewModel.ts");
    let diff = tracker2.finish(&manifest_path);

    // Generate report
    let report = generate_reports(&diff, temp_dir.path(), ".generated").unwrap();

    // Verify report content
    assert_eq!(report.generator_id, "models");
    assert_eq!(report.deleted.len(), 1);
    assert_eq!(report.deleted[0].name, "OldModel");
    assert_eq!(report.added.len(), 1);
    assert!(report.added.contains(&"NewModel".to_string()));
}

// ==================== End-to-End Scenario Tests ====================

#[test]
fn e2e_first_generation() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join(".generated").join("manifest.json");

    // First generation with no existing manifest
    let mut tracker = ManifestTracker::new("models");
    tracker.track("User", "User.ts");
    tracker.track("Post", "Post.ts");

    let entries = tracker.entries().clone();
    let diff = tracker.finish(&manifest_path);

    // All files should be marked as added
    assert_eq!(diff.added.len(), 2);
    assert_eq!(diff.deleted.len(), 0);
    assert_eq!(diff.unchanged.len(), 0);

    // Update manifest
    update_manifest(&manifest_path, "models".to_string(), entries, "", "").unwrap();

    // Verify manifest was created
    assert!(manifest_path.exists());
}

#[test]
fn e2e_unchanged_generation() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join(".generated").join("manifest.json");

    // First generation
    let mut tracker1 = ManifestTracker::new("models");
    tracker1.track("User", "User.ts");
    tracker1.track("Post", "Post.ts");
    let entries1 = tracker1.entries().clone();
    tracker1.finish(&manifest_path);
    update_manifest(&manifest_path, "models".to_string(), entries1, "", "").unwrap();

    // Second generation with same files
    let mut tracker2 = ManifestTracker::new("models");
    tracker2.track("User", "User.ts");
    tracker2.track("Post", "Post.ts");
    let diff = tracker2.finish(&manifest_path);

    // All files should be unchanged
    assert_eq!(diff.added.len(), 0);
    assert_eq!(diff.deleted.len(), 0);
    assert_eq!(diff.unchanged.len(), 2);
    assert!(!diff.has_changes());
}

#[test]
fn e2e_partial_change() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join(".generated").join("manifest.json");

    // First generation
    let mut tracker1 = ManifestTracker::new("models");
    tracker1.track("User", "User.ts");
    tracker1.track("Post", "Post.ts");
    tracker1.track("Comment", "Comment.ts");
    let entries1 = tracker1.entries().clone();
    tracker1.finish(&manifest_path);
    update_manifest(&manifest_path, "models".to_string(), entries1, "", "").unwrap();

    // Second generation: 1 deleted, 1 unchanged, 1 added
    let mut tracker2 = ManifestTracker::new("models");
    tracker2.track("User", "User.ts"); // Unchanged
    tracker2.track("Comment", "Comment.ts"); // Unchanged
    tracker2.track("Author", "Author.ts"); // Added
    // Post is deleted
    let diff = tracker2.finish(&manifest_path);

    assert_eq!(diff.added.len(), 1);
    assert_eq!(diff.deleted.len(), 1);
    assert_eq!(diff.unchanged.len(), 2);
    assert!(diff.has_changes());
}

#[test]
fn e2e_complete_replacement() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join(".generated").join("manifest.json");

    // First generation
    let mut tracker1 = ManifestTracker::new("models");
    tracker1.track("OldUser", "OldUser.ts");
    tracker1.track("OldPost", "OldPost.ts");
    let entries1 = tracker1.entries().clone();
    tracker1.finish(&manifest_path);
    update_manifest(&manifest_path, "models".to_string(), entries1, "", "").unwrap();

    // Second generation: completely different files
    let mut tracker2 = ManifestTracker::new("models");
    tracker2.track("NewUser", "NewUser.ts");
    tracker2.track("NewPost", "NewPost.ts");
    tracker2.track("NewComment", "NewComment.ts");
    let diff = tracker2.finish(&manifest_path);

    // All old files deleted, all new files added
    assert_eq!(diff.added.len(), 3);
    assert_eq!(diff.deleted.len(), 2);
    assert_eq!(diff.unchanged.len(), 0);
}

#[test]
fn e2e_barrel_update_integration() {
    let temp_dir = TempDir::new().unwrap();
    let models_dir = temp_dir.path().join("models");

    // Create directory structure
    fs::create_dir_all(&models_dir).unwrap();
    fs::write(models_dir.join("User.ts"), "export type User = {};").unwrap();
    fs::write(models_dir.join("Post.ts"), "export type Post = {};").unwrap();

    // Update barrel files
    let result = update_barrel_with_parents("models", temp_dir.path());

    assert!(result.is_ok());

    // Verify barrel file was created
    let index_path = models_dir.join("index.ts");
    assert!(index_path.exists());

    let content = fs::read_to_string(&index_path).unwrap();
    assert!(content.contains("export * from './Post';"));
    assert!(content.contains("export * from './User';"));
}

#[test]
fn e2e_manifest_and_barrel_together() {
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path();
    let models_dir = output_dir.join("models");
    let manifest_path = output_dir.join(".generated").join("manifest.json");

    // Create directory structure
    fs::create_dir_all(&models_dir).unwrap();
    fs::write(models_dir.join("User.ts"), "export type User = {};").unwrap();
    fs::write(models_dir.join("Post.ts"), "export type Post = {};").unwrap();

    // Track files
    let mut tracker = ManifestTracker::new("models");
    tracker.track("User", "models/User.ts");
    tracker.track("Post", "models/Post.ts");

    let entries = tracker.entries().clone();
    let diff = tracker.finish(&manifest_path);

    // Generate reports
    generate_reports(&diff, output_dir, ".generated").unwrap();

    // Update manifest
    update_manifest(&manifest_path, "models".to_string(), entries, "", "").unwrap();

    // Update barrel files
    force_update_barrel("models", output_dir).unwrap();

    // Verify everything was created
    assert!(manifest_path.exists());
    assert!(output_dir.join(".generated/deletion-report.json").exists());
    assert!(output_dir.join(".generated/deletion-report.md").exists());
    assert!(models_dir.join("index.ts").exists());
}
