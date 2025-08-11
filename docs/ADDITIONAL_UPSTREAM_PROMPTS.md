# Additional Upstream Enhancement Recommendations

If the `post-commit-visual-validation.yml` workflow doesn't exist yet in the upstream repository, here are AI assistant prompts to request its implementation:

## Prompt 1: Post-Commit Visual Validation Workflow Implementation

```
# Create Post-Commit Visual Validation Workflow

## Problem
The current image-comparison-and-update repository only supports pull request visual testing. Projects need post-commit visual validation to ensure golden master integrity and detect environment-specific issues after merge to main branch.

## Required Implementation
Create a new reusable workflow file: `.github/workflows/post-commit-visual-validation.yml`

### Workflow Specification

**Trigger:** `workflow_call` (to be called by consuming repositories)

**Required Inputs:**
```yaml
inputs:
  outputs_directory:
    description: 'Directory containing output images to compare'
    required: false
    default: 'outputs'
    type: string
  artifact_name:
    description: 'Name of artifact containing output images'
    required: true
    type: string
  golden_directory:
    description: 'Directory containing golden master images'
    required: false
    default: 'golden'
    type: string
  auto_update_golden:
    description: 'Automatically update golden masters if differences found'
    required: false
    default: false
    type: boolean
  update_threshold:
    description: 'Maximum FLIP error for automatic updates (0.0-1.0)'
    required: false
    default: '0.01'
    type: string
  issue_labels:
    description: 'Comma-separated labels to apply to created issues'
    required: false
    default: 'visual-regression,post-commit'
    type: string
  issue_title_prefix:
    description: 'Prefix for created issue titles'
    required: false
    default: 'Post-Commit Visual Regression'
    type: string
```

**Key Differences from PR Workflow:**
1. **Output**: Create GitHub issues instead of PR comments
2. **Auto-closure**: Automatically close issues when tests pass
3. **Deduplication**: Update existing issues instead of creating duplicates
4. **Auto-update**: Optional automatic golden master updates

**Expected Workflow Steps:**
1. Download artifacts containing output images
2. Compare against golden masters using NVIDIA FLIP
3. If failures detected:
   - Check for existing open issues with matching labels
   - Create new issue or update existing with detailed failure analysis
   - Include FLIP metrics, visual diffs, and troubleshooting steps
4. If auto_update_golden=true and errors below threshold:
   - Update golden masters with validation
   - Commit changes with audit trail
5. If all tests pass:
   - Close any existing open issues
   - Add success confirmation comment

This workflow should provide comprehensive post-commit visual validation with GitHub issue management.
```

## Prompt 2: Enhanced Visual Diff Workflow Parameters

```
# Add Enhanced Parameters to Existing Visual Diff Workflow

## Enhancement Goal
Extend the existing visual-diff.yml workflow with additional configuration options for better customization and integration.

## Required New Input Parameters

Add these optional inputs to the existing workflow:

```yaml
inputs:
  golden_directory:
    description: 'Directory containing golden master images'
    required: false
    default: 'golden'
    type: string
  flip_threshold:
    description: 'FLIP error threshold for highlighting significant differences'
    required: false
    default: '0.01'
    type: string
  enable_metrics:
    description: 'Enable collection and reporting of comparison metrics'
    required: false
    default: true
    type: boolean
  post_commit_mode:
    description: 'Enable post-commit mode (creates issues instead of PR comments)'
    required: false
    default: false
    type: boolean
  create_issues:
    description: 'Create GitHub issues for failures (when post_commit_mode=true)'
    required: false
    default: false
    type: boolean
```

**Backward Compatibility:** All new parameters should be optional with sensible defaults to maintain compatibility with existing consumers.

**Enhanced Functionality:**
1. Configurable golden master directory location
2. Adjustable FLIP error thresholds for highlighting
3. Optional metrics collection and export
4. Mode switching between PR comments and GitHub issues
5. Conditional issue creation for post-commit scenarios

This enhancement would provide better flexibility while maintaining full backward compatibility.
```

## Prompt 3: Issue Management Enhancement

```
# Implement Smart Issue Management for Visual Regression

## Enhancement Goal
Add intelligent issue lifecycle management to reduce noise and improve team workflow efficiency.

## Required Features

### Issue Deduplication Logic
- Before creating new issues, search for existing open issues with matching labels
- Update existing issues instead of creating duplicates
- Use title pattern matching: `{issue_title_prefix}: {failed_count}/{total_count} tests failed`

### Rich Issue Content Template
```markdown
# {issue_title_prefix} Failures ({failed_count}/{total_count} tests failed)

**Test Run Details:**
- **Commit:** {commit_sha}
- **Workflow:** [View Details]({workflow_url})
- **Timestamp:** {iso_timestamp}

## Summary
{failed_count} out of {total_count} visual regression tests failed.

| File | Status | FLIP Mean Error | Max Error | Action |
|------|--------|-----------------|-----------|---------|
{per_file_results_table}

## Quick Actions
**If changes are expected:**
- 🔄 [Auto-update golden masters]({auto_update_dispatch_url})

**If changes are unexpected:**
- 🔍 [Download artifacts]({artifacts_url})
- 📋 [View troubleshooting guide](#troubleshooting)

## Troubleshooting
{troubleshooting_content}

---
*This issue will close automatically when visual tests pass*
```

### Automatic Resolution
- Monitor subsequent workflow runs
- Close issues when all tests pass
- Add resolution comment with success metrics

This system should provide clear, actionable feedback while minimizing manual overhead.
```