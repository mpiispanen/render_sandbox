# Post-Commit Visual Regression Testing

This document describes the post-commit visual regression testing implementation and recommendations for upstream workflow enhancements.

## Overview

The post-commit visual regression testing system provides continuous validation of visual outputs after code is merged to the main branch. This complements the existing pre-commit (PR) visual testing by ensuring:

1. **Golden Master Validation**: Main branch outputs match expected golden masters
2. **Environment Consistency**: Visual outputs are consistent between PR and post-merge environments
3. **Continuous Monitoring**: Ongoing detection of visual regressions after merge
4. **Automated Issue Management**: GitHub issues for failures, automatic closure when resolved

## Current Implementation

### Workflow: `.github/workflows/post-commit-visual-regression.yml`

**Triggers:**
- `push` to `main` branch (automatic)
- `workflow_dispatch` (manual with optional auto-update)

**Process:**
1. **Build and Test**: Builds render_sandbox and runs GPU tests to generate images
2. **Image Comparison**: Compares outputs against golden masters using NVIDIA FLIP
3. **Failure Handling**: Creates/updates GitHub issues for visual differences
4. **Auto-Update Option**: Can automatically update golden masters when run manually
5. **Issue Resolution**: Closes issues when visual tests pass again

**Key Features:**
- Runs on self-hosted GPU instances for consistent rendering
- Detailed FLIP analysis with mean error calculations
- Comprehensive reporting with file-by-file analysis
- Artifact uploads for manual review
- Automatic issue management lifecycle

### Differences from PR Workflow

| Aspect | PR Workflow | Post-Commit Workflow |
|--------|-------------|---------------------|
| **Trigger** | Pull request events | Push to main |
| **Purpose** | Compare changes, allow acceptance | Validate main branch |
| **Output** | PR comments with image embedding | GitHub issues |
| **Failure Action** | Accept/reject via comments | Investigation required |
| **Golden Master Updates** | Manual via `/accept-image` | Optional auto-update |
| **Cleanup** | Temporary branches for display | Artifact retention |

## Benefits

### 1. Golden Master Drift Detection
- Catches cases where golden masters become outdated
- Identifies corruption or accidental deletion of reference images
- Validates that accepted changes actually work in the main environment

### 2. Environment Validation
- Ensures visual outputs are consistent between PR and main branch environments
- Catches GPU driver updates or dependency changes that affect rendering
- Validates deployment environment consistency

### 3. Continuous Monitoring
- Provides ongoing assurance that visual outputs remain stable
- Early detection of regressions that might slip through PR review
- Historical tracking of visual stability

### 4. Automated Maintenance
- Optional automatic golden master updates for expected changes
- Automatic issue creation and closure based on test status
- Reduces manual maintenance overhead

## Usage

### Automatic Operation
The workflow runs automatically on every push to main and will:
- Generate visual test images
- Compare against golden masters
- Create GitHub issues for failures
- Close issues when tests pass

### Manual Operation
Run manually via GitHub Actions with options:
1. Go to Actions → Post-Commit Visual Regression
2. Click "Run workflow"
3. Optionally enable "Automatically update golden masters if tests fail"
4. Click "Run workflow"

### Handling Failures
When visual tests fail, a GitHub issue is created with:
- Detailed comparison results
- FLIP error analysis
- Recommended actions
- Quick fix instructions

## Upstream Workflow Recommendations

The current upstream `mpiispanen/image-comparison-and-update` repository provides excellent PR-based visual diff functionality but lacks post-commit support. Here are recommendations for upstream enhancements:

### 1. New Post-Commit Workflow

**Suggested Workflow: `post-commit-visual-validation.yml`**

**Purpose:** Handle visual regression testing for main branch pushes with different output handling than PR workflows.

**Key Differences from PR Workflow:**
- Issue creation instead of PR comments
- Support for automatic golden master updates
- Different notification mechanisms
- Branch-specific behavior

**Inputs:**
```yaml
inputs:
  outputs_directory:
    description: 'Directory containing output images to compare'
    required: false
    default: 'outputs'
    type: string
  auto_update_golden:
    description: 'Automatically update golden masters if differences found'
    required: false
    default: false
    type: boolean
  issue_labels:
    description: 'Labels to apply to created issues'
    required: false
    default: 'visual-regression,post-commit'
    type: string
  notification_method:
    description: 'How to notify about failures (issue, email, slack)'
    required: false
    default: 'issue'
    type: string
```

**Outputs:**
- GitHub issues for failures
- Automatic issue closure when resolved
- Artifact uploads for manual review
- Optional automatic golden master updates

### 2. Enhanced Issue Management

**Features:**
- Smart issue deduplication (update existing instead of creating new)
- Automatic issue closure when tests pass
- Rich issue templates with troubleshooting steps
- Label-based categorization and filtering

**Issue Template:**
```markdown
## Visual Regression Failure in Main Branch

**Summary:** {failed_tests}/{total_tests} visual tests failed
**Commit:** {commit_sha}
**Workflow:** [View Details]({workflow_url})

### Failed Tests
{list_of_failed_tests_with_flip_scores}

### Recommended Actions
1. Review visual differences in workflow artifacts
2. Investigate cause of changes
3. Update golden masters if changes are intentional

### Quick Fix
Use automatic golden master update: [Run Workflow]({workflow_dispatch_url})
```

### 3. Auto-Update Functionality

**Smart Golden Master Updates:**
- Validation checks before updating
- Commit message with context
- Optional approval workflow for sensitive changes
- Rollback capability

**Safety Features:**
- Maximum change threshold protection
- Review requirement for large changes
- Audit trail for all updates
- Optional human approval gates

### 4. Notification Integrations

**Multiple Notification Methods:**
- GitHub issues (current)
- Slack integration for team notifications
- Email alerts for critical failures
- Microsoft Teams webhooks
- Discord notifications

**Configurable Notification Rules:**
- Immediate alerts for failures
- Daily/weekly summary reports
- Escalation for repeated failures
- Success confirmations

### 5. Enhanced Reporting

**Advanced Analytics:**
- Trend analysis of visual stability
- Performance metrics for image comparison
- Historical failure patterns
- Golden master update frequency

**Reporting Formats:**
- Detailed HTML reports
- CSV exports for analysis
- JSON APIs for integration
- Dashboard-ready metrics

## Implementation Prompts for Upstream Repository

Here are specific prompts that could be provided to the upstream repository maintainers:

### Prompt 1: Basic Post-Commit Workflow

```
# Add Post-Commit Visual Regression Testing Support

## Problem
The current visual-diff.yml workflow only supports pull request contexts. Many projects need visual regression testing for their main branch after merge to ensure:
- Golden masters remain valid post-merge
- Environment consistency between PR and main branch
- Continuous monitoring of visual stability
- Automatic issue creation for failures

## Requested Enhancement
Create a new workflow `post-commit-visual-validation.yml` that:

1. **Triggers:** workflow_call for main branch pushes
2. **Outputs:** GitHub issues instead of PR comments
3. **Features:**
   - Same FLIP-based image comparison
   - Issue creation/update for failures
   - Automatic issue closure when resolved
   - Optional automatic golden master updates
   - Configurable issue labels and templates

## Use Case
```yaml
# In consuming repository
jobs:
  post-commit-visual:
    uses: mpiispanen/image-comparison-and-update/.github/workflows/post-commit-visual-validation.yml@main
    with:
      outputs_directory: outputs
      auto_update_golden: false
      issue_labels: 'visual-regression,post-commit,bug'
```

This would provide comprehensive visual regression coverage for both pre-commit (PR) and post-commit (main) scenarios.
```

### Prompt 2: Auto-Update Enhancement

```
# Add Automatic Golden Master Update Support

## Enhancement Request
Extend the visual comparison workflows to support automatic golden master updates with safety checks:

## Features Needed
1. **Smart Updates:** Only update when changes are below a threshold
2. **Safety Checks:** Validate images before updating
3. **Audit Trail:** Detailed commit messages with change context
4. **Rollback Support:** Easy reversion of automatic updates
5. **Approval Gates:** Optional human approval for large changes

## Configuration Options
```yaml
inputs:
  auto_update_golden:
    description: 'Enable automatic golden master updates'
    type: boolean
  update_threshold:
    description: 'Maximum FLIP error for automatic updates'
    default: '0.01'
    type: string
  require_approval:
    description: 'Require manual approval for updates'
    type: boolean
```

This would reduce maintenance overhead while maintaining quality controls.
```

### Prompt 3: Enhanced Notification System

```
# Add Multi-Channel Notification Support

## Problem
GitHub issues work well for tracking but teams often need immediate notifications via their preferred channels (Slack, email, Teams).

## Requested Enhancement
Add configurable notification support to visual regression workflows:

## Notification Channels
1. **Slack:** Webhook integration with rich formatting
2. **Email:** SMTP support for email alerts
3. **Microsoft Teams:** Webhook cards with visual previews
4. **Discord:** Bot integration for community projects

## Configuration
```yaml
inputs:
  notifications:
    description: 'Notification configuration (JSON)'
    type: string
    # Example: '{"slack": {"webhook": "...", "channel": "#visual-tests"}, "email": {"to": "team@example.com"}}'
```

## Message Templates
- Failure alerts with quick action buttons
- Success confirmations
- Weekly/daily summary reports
- Escalation for repeated failures

This would improve team awareness and response times for visual regression issues.
```

## Testing the Implementation

To test the new post-commit visual regression workflow:

1. **Enable the workflow** by merging these changes
2. **Make a visual change** in a PR and merge it
3. **Verify the workflow runs** on the main branch
4. **Check issue creation** if differences are detected
5. **Test auto-update** via manual workflow dispatch
6. **Verify issue closure** when tests pass

## Future Enhancements

Potential improvements for the post-commit system:
1. **Slack/Teams integration** for immediate failure notifications
2. **Trend analysis** of visual stability over time
3. **Performance metrics** for image comparison
4. **Machine learning** to predict when golden masters need updates
5. **Advanced filtering** of acceptable vs problematic changes