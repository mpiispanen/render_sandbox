# Post-Commit Visual Regression Testing Specifications

This document provides comprehensive specifications and AI code assistant prompts for implementing post-commit visual regression testing in the upstream `mpiispanen/image-comparison-and-update` repository.

## Problem Statement

The current visual regression testing system only runs during pull requests, leaving the main branch without continuous visual validation. This creates several risks:

- **Golden master drift**: No detection when reference images become outdated or corrupted
- **Environment differences**: Visual outputs might differ between PR and main branch environments  
- **Missed regressions**: Visual changes that slip through PR review go undetected after merge
- **No continuous monitoring**: No ongoing assurance that main branch produces expected visual outputs

## Solution Requirements

Post-commit visual regression testing should complement the existing PR-based system by providing:

1. **Golden Master Validation**: Main branch outputs match expected golden masters
2. **Environment Consistency**: Visual outputs are consistent between PR and post-merge environments
3. **Continuous Monitoring**: Ongoing detection of visual regressions after merge
4. **Automated Issue Management**: GitHub issues for failures, automatic closure when resolved

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

## Upstream Repository Enhancement Specifications

### Core Requirements for Upstream Implementation

The upstream repository should implement a new reusable workflow that provides:

1. **Post-Commit Visual Validation Workflow**
   - Trigger: `workflow_call` for main branch pushes
   - Purpose: Validate visual outputs against golden masters
   - Output: GitHub issues instead of PR comments

2. **Issue Management System**
   - Create issues for visual regression failures
   - Update existing issues instead of creating duplicates
   - Automatically close issues when tests pass
   - Rich issue templates with actionable information

3. **Optional Auto-Update Functionality**
   - Automatic golden master updates with safety checks
   - Configurable thresholds and approval gates
   - Audit trail for all changes

4. **Enhanced Notification Support**
   - Multiple notification channels (Slack, Teams, email)
   - Configurable notification rules
   - Rich message formatting with visual previews

### Detailed Workflow Specifications

#### Input Parameters
```yaml
inputs:
  outputs_directory:
    description: 'Directory containing output images to compare'
    required: false
    default: 'outputs'
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
  notification_config:
    description: 'JSON configuration for notifications'
    required: false
    type: string
  require_approval:
    description: 'Require manual approval for golden master updates'
    required: false
    default: false
    type: boolean
```

#### Expected Workflow Behavior

1. **Image Comparison Phase**
   - Compare all images in outputs_directory against golden_directory
   - Use NVIDIA FLIP for perceptual difference analysis
   - Generate detailed comparison metrics (mean error, max error, percentiles)
   - Create visual diff images for failed comparisons

2. **Failure Handling Phase**
   - Create comprehensive GitHub issue with:
     - Summary of failed tests with metrics
     - Links to visual diff artifacts
     - Troubleshooting recommendations
     - Quick action buttons/links
   - Update existing issues instead of creating duplicates
   - Apply configurable labels for organization

3. **Auto-Update Phase (Optional)**
   - When enabled and differences are below threshold:
     - Validate new images (size, format, basic sanity checks)
     - Update golden masters with descriptive commit message
     - Log all changes for audit trail
   - When threshold exceeded or approval required:
     - Create issue with auto-update option
     - Wait for manual approval before proceeding

4. **Resolution Phase**
   - Monitor for successful test runs
   - Automatically close resolved issues
   - Add resolution comments with context

#### Issue Template Specification

```markdown
# {issue_title_prefix} Failures ({failed_count}/{total_count} tests failed)

**Test Run Details:**
- **Commit:** {commit_sha}
- **Branch:** {branch_name}
- **Timestamp:** {run_timestamp}
- **Workflow:** [View Details]({workflow_url})

## Summary

{failed_count} out of {total_count} visual regression tests failed with the following results:

| File | Status | FLIP Mean Error | Max Error | Action Required |
|------|--------|-----------------|-----------|-----------------|
{per_file_results_table}

## Failed Tests Details

{detailed_failure_analysis}

## Recommended Actions

### Immediate Steps
1. **Review Visual Differences:** Download the workflow artifacts to examine the visual changes
2. **Investigate Root Cause:** Determine if changes are intentional or indicate a bug
3. **Update Golden Masters:** If changes are expected, update the reference images

### Quick Fixes

**Option 1: Automatic Update (if changes are expected)**
```bash
# Run the workflow with auto-update enabled
gh workflow run post-commit-visual-regression.yml --ref main -f auto_update_golden=true
```

**Option 2: Manual Update**
```bash
# Download artifacts and manually update golden masters
# Then commit the updated golden masters
```

## Troubleshooting

**Common Causes of Visual Regression:**
- Graphics driver updates
- Dependency version changes
- Platform/environment differences
- Intentional visual changes not updated in golden masters

**Environment Information:**
- Runner: {runner_os}
- GPU: {gpu_info}
- Graphics Driver: {driver_version}

---
*This issue will be automatically closed when all visual regression tests pass.*
```

## Complete AI Code Assistant Prompts for Upstream Implementation

The following prompts are ready-to-use specifications that can be provided to the upstream repository maintainers or AI coding assistants to implement the required functionality.

### Prompt 1: Core Post-Commit Visual Validation Workflow

```
# Implement Post-Commit Visual Regression Testing Workflow

## Problem Statement
The current image-comparison-and-update repository only supports pull request visual testing. Many projects need post-commit visual validation for their main branch to ensure golden master integrity and detect environment-specific issues after merge.

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
```

**Required Permissions:**
```yaml
permissions:
  contents: write
  issues: write
  actions: read
```

### Implementation Steps

1. **Image Comparison Logic**
   - Use existing NVIDIA FLIP comparison functionality
   - Compare all PNG files in outputs_directory against golden_directory
   - Calculate mean error, max error, and generate diff images
   - Store results in structured format (JSON/markdown)

2. **Issue Management**
   - Check for existing open issues with matching labels
   - Update existing issue if found, create new if not
   - Include comprehensive failure analysis with FLIP metrics
   - Add troubleshooting steps and quick action recommendations

3. **Auto-Update Functionality**
   - When auto_update_golden=true and errors below threshold:
     - Copy new images to golden_directory
     - Commit changes with descriptive message
     - Include audit information (commit SHA, error metrics)
   - When above threshold: create issue with manual update recommendation

4. **Artifact Management**
   - Upload comparison results and diff images
   - Include golden masters for reference
   - Retain artifacts for configurable period

5. **Success Handling**
   - Automatically close resolved issues when all tests pass
   - Add resolution comment with success confirmation

### Expected Usage in Consuming Repository
```yaml
name: Post-Commit Visual Regression
on:
  push:
    branches: [ main ]

jobs:
  generate-images:
    runs-on: [self-hosted, linux, x64]
    steps:
      # Build and run tests to generate images in outputs/
      
  post-commit-visual:
    needs: generate-images
    uses: mpiispanen/image-comparison-and-update/.github/workflows/post-commit-visual-validation.yml@main
    with:
      outputs_directory: outputs
      auto_update_golden: false
      issue_labels: 'visual-regression,post-commit,bug'
```

This workflow should provide the same high-quality image comparison as the existing PR workflow but with GitHub issue output instead of PR comments.
```

### Prompt 2: Enhanced Issue Management System

```
# Implement Smart Issue Management for Visual Regression

## Enhancement Goal
Extend the post-commit visual validation workflow with intelligent issue lifecycle management to reduce noise and improve team workflow.

## Required Features

### 1. Issue Deduplication
- Before creating new issues, search for existing open issues with matching labels
- Update existing issues instead of creating duplicates
- Use issue title pattern matching to identify related issues

### 2. Rich Issue Templates
Create comprehensive issue template with these sections:
- **Executive Summary**: Failed test count and key metrics
- **Detailed Results Table**: Per-file FLIP analysis with visual status
- **Troubleshooting Guide**: Common causes and investigation steps
- **Quick Actions**: One-click solutions for common scenarios
- **Environment Information**: Runner details, GPU info, driver versions

### 3. Automatic Issue Resolution
- Monitor subsequent workflow runs for the same repository
- Automatically close issues when all visual tests pass
- Add resolution comment with success confirmation and workflow link
- Update issue labels to indicate resolution

### 4. Issue Categorization
Support configurable labeling system:
- Severity labels based on FLIP error thresholds
- Component labels based on failed test patterns
- Priority labels based on failure frequency

### Implementation Requirements

**Issue Title Pattern:**
`Post-Commit Visual Regression: {failed_count}/{total_count} tests failed (Run #{run_number})`

**Issue Body Template:**
```markdown
# Visual Regression Detected in Main Branch

**Test Summary:**
- 🔴 **Failed:** {failed_count} tests
- ✅ **Passed:** {passed_count} tests  
- 📊 **Total:** {total_count} tests
- ⚡ **Worst Error:** {max_flip_error}

**Environment:**
- **Commit:** [`{short_sha}`]({commit_url})
- **Workflow:** [#{run_number}]({workflow_url})
- **Timestamp:** {iso_timestamp}

## Failed Tests Analysis

| Test File | FLIP Error | Status | Visual Diff |
|-----------|------------|--------|-------------|
{test_results_table}

## Quick Actions

**If these changes are expected:**
- 🔄 [Auto-update golden masters]({auto_update_url})
- 📝 Manual update instructions below

**If these changes are unexpected:**
- 🔍 [Download artifacts for investigation]({artifacts_url})
- 📋 [View troubleshooting guide](#troubleshooting)

## Troubleshooting

<details>
<summary>🔧 Common Causes and Solutions</summary>

**Graphics Environment Changes:**
- GPU driver updates
- Different GPU hardware  
- Graphics library version changes

**Application Changes:**
- Rendering pipeline modifications
- Font/text rendering updates
- Color space or gamma changes

**Investigation Steps:**
1. Compare diff images in workflow artifacts
2. Check recent commits for rendering changes
3. Verify test environment consistency
4. Review golden master validity

</details>

---
*This issue will close automatically when visual tests pass*
```

**Auto-Close Logic:**
- Run after each successful post-commit workflow
- Query open issues with visual-regression labels
- Close issues when failure count drops to zero
- Add success comment with metrics

This system should provide clear, actionable feedback while minimizing manual issue management overhead.
```

### Prompt 3: Automatic Golden Master Update System

```
# Implement Safe Automatic Golden Master Updates

## Problem
Manual golden master updates create friction in the development workflow and can lead to outdated reference images when developers forget to update them.

## Solution Requirements
Implement an optional automatic update system with multiple safety mechanisms and audit capabilities.

### Core Features

#### 1. Threshold-Based Updates
- Only auto-update when FLIP error is below configurable threshold
- Default threshold: 0.01 (1% error)
- Support per-file thresholds for different test types
- Skip auto-update for errors above threshold (require manual review)

#### 2. Validation Pipeline
Before updating any golden master, validate:
- **Image integrity**: Valid PNG format, reasonable dimensions
- **Size bounds**: File size within expected range (prevent corruption)
- **Content validation**: Basic sanity checks (not solid colors, has expected features)
- **Metadata preservation**: Maintain creation date and other metadata

#### 3. Audit Trail System
- **Descriptive commits**: Include error metrics, test context, and change summary
- **Change documentation**: Log what changed and why in commit message
- **Rollback information**: Include previous golden master hash for easy reversion
- **Approval tracking**: Record manual approvals when required

#### 4. Safety Mechanisms
- **Batch limits**: Maximum number of files updated in single run
- **Review requirements**: Force manual approval for large changes
- **Rollback capability**: Easy reversion to previous golden masters
- **Approval gates**: Optional human approval before updates

### Implementation Specification

**Extended Input Parameters:**
```yaml
inputs:
  auto_update_golden:
    description: 'Enable automatic golden master updates'
    type: boolean
    default: false
  update_threshold:
    description: 'Maximum FLIP error for automatic updates'
    type: string
    default: '0.01'
  max_auto_updates:
    description: 'Maximum files to auto-update in single run'
    type: number
    default: 10
  require_approval:
    description: 'Require manual approval for updates above threshold'
    type: boolean
    default: false
  approval_threshold:
    description: 'FLIP error threshold requiring manual approval'
    type: string
    default: '0.05'
```

**Auto-Update Workflow Logic:**
1. **Categorize failures** by FLIP error threshold
2. **Auto-eligible**: Error < update_threshold → automatic update
3. **Review-required**: Error > approval_threshold → manual approval needed
4. **Middle-ground**: Between thresholds → create approval issue

**Commit Message Template:**
```
Auto-update golden masters: {update_count} files updated

Visual regression test run: #{workflow_run}
Commit: {commit_sha}
Timestamp: {iso_timestamp}

Updated files:
{file_list_with_metrics}

Validation results:
- All images passed integrity checks
- Total FLIP error reduced from {old_error} to {new_error}
- No files exceeded safety thresholds

Auto-update criteria:
- Threshold: {threshold}
- Max files: {max_files}
- Approval required: {approval_setting}

Rollback: To revert these changes, run:
git revert {commit_hash}
```

**Approval Workflow (when required):**
- Create separate issue for approval with preview images
- Include side-by-side comparisons and error analysis
- Provide approval/rejection buttons via GitHub Actions
- Block auto-update until manual approval received

**Safety Validations:**
```bash
# Image integrity validation
validate_image() {
  local file="$1"
  
  # Check file format
  if ! file "$file" | grep -q "PNG image data"; then
    echo "ERROR: Invalid PNG format"
    return 1
  fi
  
  # Check dimensions (reasonable bounds)
  local dimensions=$(identify -format "%w %h" "$file")
  local width=$(echo $dimensions | cut -d' ' -f1)
  local height=$(echo $dimensions | cut -d' ' -f2)
  
  if [ $width -lt 10 ] || [ $width -gt 4096 ] || [ $height -lt 10 ] || [ $height -gt 4096 ]; then
    echo "ERROR: Suspicious dimensions: ${width}x${height}"
    return 1
  fi
  
  # Check file size (reasonable bounds)
  local size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file")
  if [ $size -lt 100 ] || [ $size -gt 10485760 ]; then  # 100B to 10MB
    echo "ERROR: Suspicious file size: $size bytes"
    return 1
  fi
  
  return 0
}
```

This system should balance automation with safety, ensuring golden masters stay current while preventing accidental corruption or inappropriate updates.
```

### Prompt 4: Multi-Channel Notification System

```
# Implement Multi-Channel Notification Support

## Problem
While GitHub issues provide good tracking, teams need immediate notifications through their preferred communication channels (Slack, Teams, email) for rapid response to visual regressions.

## Solution Requirements
Add configurable notification support to all visual regression workflows with rich formatting and actionable buttons.

### Supported Notification Channels

#### 1. Slack Integration
- Webhook-based notifications with rich blocks
- Channel-specific routing
- Thread support for related notifications
- Interactive buttons for quick actions

#### 2. Microsoft Teams
- Webhook cards with visual previews
- Mention support for team alerts
- Action buttons for workflow triggers
- Adaptive card formatting

#### 3. Email Notifications
- SMTP support for direct email alerts
- HTML formatting with embedded images
- Multiple recipient support
- Escalation rules for repeated failures

#### 4. Discord Integration
- Bot-based notifications for community projects
- Embed support with visual previews
- Role mention capabilities
- Custom emoji reactions

### Configuration System

**Input Parameter:**
```yaml
inputs:
  notification_config:
    description: 'JSON configuration for notifications'
    required: false
    type: string
    # Example: '{"slack": {"webhook": "...", "channel": "#visual-tests"}, "email": {"smtp": "...", "to": ["team@example.com"]}}'
```

**Configuration Schema:**
```json
{
  "slack": {
    "webhook_url": "https://hooks.slack.com/...",
    "channel": "#visual-regression",
    "username": "Visual Regression Bot",
    "mention_on_failure": ["@channel", "@visual-team"],
    "thread_replies": true
  },
  "teams": {
    "webhook_url": "https://outlook.office.com/webhook/...",
    "mention_users": ["john@company.com"],
    "card_theme": "attention"
  },
  "email": {
    "smtp_server": "smtp.gmail.com",
    "smtp_port": 587,
    "username": "notifications@company.com",
    "password_secret": "EMAIL_PASSWORD",
    "to": ["team@company.com", "qa@company.com"],
    "from": "Visual Regression <notifications@company.com>"
  },
  "discord": {
    "webhook_url": "https://discord.com/api/webhooks/...",
    "username": "Visual Bot",
    "avatar_url": "https://example.com/bot-avatar.png",
    "mention_roles": ["@Visual Team"]
  }
}
```

### Notification Templates

#### Slack Failure Notification
```json
{
  "blocks": [
    {
      "type": "header",
      "text": {
        "type": "plain_text",
        "text": "🔴 Visual Regression Detected"
      }
    },
    {
      "type": "section",
      "fields": [
        {
          "type": "mrkdwn",
          "text": "*Repository:* {repo_name}"
        },
        {
          "type": "mrkdwn",
          "text": "*Branch:* {branch_name}"
        },
        {
          "type": "mrkdwn",
          "text": "*Failed Tests:* {failed_count}/{total_count}"
        },
        {
          "type": "mrkdwn",
          "text": "*Worst Error:* {max_flip_error}"
        }
      ]
    },
    {
      "type": "section",
      "text": {
        "type": "mrkdwn",
        "text": "*Failed Tests:*\n{failed_tests_list}"
      }
    },
    {
      "type": "actions",
      "elements": [
        {
          "type": "button",
          "text": {
            "type": "plain_text",
            "text": "View Details"
          },
          "url": "{workflow_url}",
          "style": "primary"
        },
        {
          "type": "button",
          "text": {
            "type": "plain_text",
            "text": "Auto-Update Golden Masters"
          },
          "url": "{auto_update_url}",
          "style": "danger"
        },
        {
          "type": "button",
          "text": {
            "type": "plain_text",
            "text": "Download Artifacts"
          },
          "url": "{artifacts_url}"
        }
      ]
    }
  ]
}
```

#### Teams Adaptive Card
```json
{
  "$schema": "http://adaptivecards.io/schemas/adaptive-card.json",
  "type": "AdaptiveCard",
  "version": "1.2",
  "body": [
    {
      "type": "TextBlock",
      "size": "Medium",
      "weight": "Bolder",
      "text": "Visual Regression Detected",
      "color": "Attention"
    },
    {
      "type": "FactSet",
      "facts": [
        {
          "title": "Repository:",
          "value": "{repo_name}"
        },
        {
          "title": "Failed Tests:",
          "value": "{failed_count}/{total_count}"
        },
        {
          "title": "Commit:",
          "value": "{short_sha}"
        }
      ]
    },
    {
      "type": "TextBlock",
      "text": "**Failed Tests:**\n{failed_tests_summary}",
      "wrap": true
    }
  ],
  "actions": [
    {
      "type": "Action.OpenUrl",
      "title": "View Workflow",
      "url": "{workflow_url}"
    },
    {
      "type": "Action.OpenUrl",
      "title": "Auto-Update",
      "url": "{auto_update_url}"
    }
  ]
}
```

#### Email HTML Template
```html
<!DOCTYPE html>
<html>
<head>
  <style>
    .container { font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; }
    .header { background-color: #dc3545; color: white; padding: 20px; text-align: center; }
    .content { padding: 20px; }
    .test-results { border-collapse: collapse; width: 100%; }
    .test-results th, .test-results td { border: 1px solid #ddd; padding: 8px; text-align: left; }
    .failed { background-color: #ffe6e6; }
    .actions { text-align: center; margin: 20px 0; }
    .button { display: inline-block; padding: 10px 20px; margin: 5px; background-color: #007bff; color: white; text-decoration: none; border-radius: 5px; }
  </style>
</head>
<body>
  <div class="container">
    <div class="header">
      <h1>Visual Regression Detected</h1>
      <p>{failed_count} out of {total_count} tests failed</p>
    </div>
    <div class="content">
      <p><strong>Repository:</strong> {repo_name}</p>
      <p><strong>Commit:</strong> {commit_sha}</p>
      <p><strong>Timestamp:</strong> {timestamp}</p>
      
      <h3>Failed Tests</h3>
      <table class="test-results">
        <tr>
          <th>Test File</th>
          <th>FLIP Error</th>
          <th>Status</th>
        </tr>
        {failed_tests_table_rows}
      </table>
      
      <div class="actions">
        <a href="{workflow_url}" class="button">View Details</a>
        <a href="{auto_update_url}" class="button">Auto-Update</a>
        <a href="{artifacts_url}" class="button">Download Artifacts</a>
      </div>
    </div>
  </div>
</body>
</html>
```

### Implementation Requirements

**Notification Trigger Points:**
1. **Failure Detection**: Send alert when visual regressions detected
2. **Resolution**: Send success notification when issues resolved
3. **Auto-Update**: Confirm when golden masters automatically updated
4. **Manual Actions**: Notify when manual intervention required

**Error Handling:**
- Graceful fallback when notification services unavailable
- Retry logic for transient failures
- Validation of notification configuration
- Logging of notification delivery status

**Security Considerations:**
- Secure storage of webhook URLs and credentials
- Input validation for all configuration parameters
- Rate limiting to prevent notification spam
- Audit logging of notification activities

This system should provide immediate awareness of visual regressions while maintaining security and reliability.
```

### Prompt 5: Advanced Analytics and Reporting

```
# Implement Advanced Visual Regression Analytics

## Problem
Teams need better insights into visual regression patterns, trends, and performance to improve their testing strategy and identify systemic issues.

## Required Features

#### 1. Trend Analysis System
- **Visual Stability Metrics**: Track FLIP error trends over time
- **Failure Pattern Analysis**: Identify recurring failure patterns
- **Golden Master Age Tracking**: Monitor how long reference images remain valid
- **Environment Impact Analysis**: Correlate failures with environment changes

#### 2. Performance Monitoring
- **Comparison Speed Metrics**: Track FLIP processing time
- **Resource Usage**: Monitor memory and CPU usage during comparisons
- **Throughput Analysis**: Images processed per minute
- **Bottleneck Identification**: Identify slow comparison operations

#### 3. Dashboard-Ready Metrics
Export metrics in formats suitable for monitoring dashboards:
- **Prometheus metrics** for Grafana dashboards
- **JSON APIs** for custom integrations
- **CSV exports** for analysis tools
- **Webhook payloads** for real-time monitoring

#### 4. Historical Reporting
- **Weekly/Monthly summaries** of visual regression activity
- **Top failing tests** identification
- **Golden master update frequency** tracking
- **Team performance metrics** (resolution time, etc.)

### Implementation Specification

**Extended Workflow Outputs:**
```yaml
outputs:
  metrics_json:
    description: 'JSON metrics for dashboard integration'
    value: ${{ steps.metrics.outputs.json }}
  trend_data:
    description: 'Historical trend data'
    value: ${{ steps.metrics.outputs.trends }}
  performance_stats:
    description: 'Performance metrics'
    value: ${{ steps.metrics.outputs.performance }}
```

**Metrics Collection Logic:**
```bash
# Collect comprehensive metrics
collect_metrics() {
  local start_time=$(date +%s)
  local total_images=0
  local total_errors=0
  local max_error=0
  local processing_times=()
  
  # Process each image and collect timing data
  for image in outputs/*.png; do
    local img_start=$(date +%s.%N)
    
    # Run FLIP comparison
    flip_result=$(flip -r "golden/$(basename "$image")" -t "$image" -v 2)
    
    local img_end=$(date +%s.%N)
    local img_duration=$(echo "$img_end - $img_start" | bc -l)
    
    processing_times+=($img_duration)
    total_images=$((total_images + 1))
    
    # Extract error metrics
    local mean_error=$(echo "$flip_result" | grep "Mean:" | awk '{print $2}')
    total_errors=$(echo "$total_errors + $mean_error" | bc -l)
    
    if (( $(echo "$mean_error > $max_error" | bc -l) )); then
      max_error=$mean_error
    fi
  done
  
  local end_time=$(date +%s)
  local total_duration=$((end_time - start_time))
  
  # Calculate statistics
  local avg_error=$(echo "scale=6; $total_errors / $total_images" | bc -l)
  local throughput=$(echo "scale=2; $total_images / $total_duration" | bc -l)
  
  # Export metrics in JSON format
  cat > metrics.json << EOF
{
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "workflow_run": "$GITHUB_RUN_NUMBER",
  "commit_sha": "$GITHUB_SHA",
  "performance": {
    "total_images": $total_images,
    "total_duration": $total_duration,
    "throughput_images_per_second": $throughput,
    "average_processing_time": $(echo "${processing_times[@]}" | tr ' ' '\n' | awk '{sum+=$1} END {print sum/NR}')
  },
  "quality": {
    "average_flip_error": $avg_error,
    "maximum_flip_error": $max_error,
    "total_error_sum": $total_errors
  },
  "results": {
    "total_tests": $total_images,
    "passed_tests": $passed_count,
    "failed_tests": $failed_count,
    "success_rate": $(echo "scale=4; $passed_count / $total_images * 100" | bc -l)
  }
}
EOF
}
```

**Dashboard Integration Example:**
```yaml
# Add to workflow to export Prometheus metrics
- name: Export Prometheus metrics
  run: |
    cat >> metrics.prom << EOF
    # HELP visual_regression_tests_total Total number of visual regression tests
    # TYPE visual_regression_tests_total counter
    visual_regression_tests_total{repository="${GITHUB_REPOSITORY}",branch="${GITHUB_REF_NAME}"} ${{ steps.metrics.outputs.total_tests }}
    
    # HELP visual_regression_failures_total Total number of failed visual regression tests  
    # TYPE visual_regression_failures_total counter
    visual_regression_failures_total{repository="${GITHUB_REPOSITORY}",branch="${GITHUB_REF_NAME}"} ${{ steps.metrics.outputs.failed_tests }}
    
    # HELP visual_regression_flip_error_max Maximum FLIP error detected
    # TYPE visual_regression_flip_error_max gauge
    visual_regression_flip_error_max{repository="${GITHUB_REPOSITORY}",branch="${GITHUB_REF_NAME}"} ${{ steps.metrics.outputs.max_error }}
    
    # HELP visual_regression_processing_duration_seconds Time spent processing images
    # TYPE visual_regression_processing_duration_seconds gauge  
    visual_regression_processing_duration_seconds{repository="${GITHUB_REPOSITORY}",branch="${GITHUB_REF_NAME}"} ${{ steps.metrics.outputs.duration }}
    EOF
    
    # Upload to monitoring system
    curl -X POST "$PROMETHEUS_GATEWAY/metrics/job/visual_regression" --data-binary @metrics.prom
```

This analytics system should provide deep insights into visual regression patterns and help teams optimize their testing strategies.
```

## Summary for Upstream Implementation

These specifications provide a complete roadmap for implementing comprehensive post-commit visual regression testing in the upstream `mpiispanen/image-comparison-and-update` repository. The implementation would include:

### Core Components
1. **Post-Commit Visual Validation Workflow** - Main workflow for comparing images and managing issues
2. **Smart Issue Management System** - Intelligent GitHub issue lifecycle management
3. **Automatic Golden Master Updates** - Safe, threshold-based automatic updates with audit trails
4. **Multi-Channel Notifications** - Slack, Teams, email, and Discord integration
5. **Advanced Analytics and Reporting** - Comprehensive metrics and trend analysis

### Key Benefits
- **Complete Coverage**: Both PR and post-commit visual regression testing
- **Reduced Maintenance**: Automated issue management and golden master updates
- **Better Team Communication**: Multi-channel notifications with rich formatting
- **Data-Driven Insights**: Analytics for improving visual testing strategies
- **Enterprise Ready**: Security, audit trails, and approval workflows

### Integration Example
Once implemented upstream, consuming repositories would use it like this:

```yaml
name: Post-Commit Visual Regression
on:
  push:
    branches: [ main ]

jobs:
  generate-images:
    runs-on: [self-hosted, linux, x64]
    steps:
      - uses: actions/checkout@v4
      - name: Build and test
        run: |
          cargo build --release
          cargo test --features gpu-tests --release
      - name: Upload outputs
        uses: actions/upload-artifact@v4
        with:
          name: test-outputs
          path: outputs/

  post-commit-visual:
    needs: generate-images
    uses: mpiispanen/image-comparison-and-update/.github/workflows/post-commit-visual-validation.yml@main
    with:
      outputs_directory: outputs
      auto_update_golden: false
      update_threshold: '0.01'
      issue_labels: 'visual-regression,post-commit,bug'
      notification_config: |
        {
          "slack": {
            "webhook_url": "${{ secrets.SLACK_WEBHOOK }}",
            "channel": "#visual-tests",
            "mention_on_failure": ["@visual-team"]
          }
        }
```

This would provide a robust, enterprise-grade visual regression testing solution with minimal setup overhead for consuming repositories.