# Visual Regression Testing Guide

This document provides a comprehensive guide to the visual regression testing system implemented in the render_sandbox project.

## Overview

The visual regression testing system automatically detects visual changes in rendered output by comparing new images against reference "golden" images. It uses NVIDIA FLIP for high-fidelity pixel-level comparison and provides an interactive workflow for reviewing and accepting changes through GitHub PRs.

## Features

- **High-fidelity comparison**: Uses NVIDIA FLIP for precise image comparison with perceptual weighting
- **Interactive workflow**: Accept/reject changes via PR comments like `/accept-image filename.png`
- **LFS-aware storage**: Efficiently handles large image files using Git LFS
- **Consolidated reporting**: Single PR comment with all visual changes and statistics
- **Security**: Permission checks ensure only authorized users can accept images
- **Fallback support**: Works in CI environments without GPU access

## Workflow Overview

### 1. Automatic Testing (On PR)

When you open or update a pull request, the system:

1. Builds the render_sandbox project
2. Runs `scripts/generate_test_images.py` to generate test images
3. Compares each output against its golden reference using NVIDIA FLIP
4. Generates visual diff images showing differences
5. Posts a consolidated report as a PR comment
6. Uploads artifacts for the acceptance workflow

### 2. Review and Accept Changes

If visual differences are detected, the PR comment will show:

- Summary table of all images with their status
- Detailed comparison for changed images with FLIP statistics
- Side-by-side view of differences and new outputs
- Instructions for accepting changes

Example PR comment:

```markdown
# Visual Regression Test Results

## Summary

| File | Status | FLIP Mean Error | Result |
|------|--------|-----------------|--------|
| `basic_render_800x600.png` | 🔄 Changed | 0.025 | Needs review |
| `square_512x512.png` | ✅ Passed | 0.000 | No changes |

## Detailed Results

### 🔄 **Changed Image:** `basic_render_800x600.png`

**FLIP Statistics:**
- **Mean Error:** 0.025
- **Weighted Median:** 0.018

| Difference | New Output |
|------------|------------|
| ![Difference](./diffs/diff_basic_render_800x600.png) | ![New Output](./outputs/basic_render_800x600.png) |

To accept this change, comment: `/accept-image basic_render_800x600.png`
```

### 3. Accepting Changes

To accept a visual change, comment on the PR:

```
/accept-image basic_render_800x600.png
```

This will:
- Validate you have write permissions to the repository
- Download the artifacts from the visual diff workflow
- Move the new image to the `golden/` directory
- Commit and push the change using Git LFS
- Confirm acceptance with a PR comment

## Test Configuration

### Current Test Cases

The system includes several pre-configured test cases:

1. **basic_render_800x600** - Standard resolution rendering test
2. **high_res_1920x1080** - High resolution rendering test  
3. **square_512x512** - Square aspect ratio test
4. **antialiased_4x** - Anti-aliased rendering with 4x MSAA
5. **minimal_400x300** - Minimal resolution test

### Adding New Tests

To add a new visual test case, edit `scripts/generate_test_images.py`:

```python
test_cases = [
    # ... existing test cases ...
    {
        'name': 'my_new_test_case',
        'width': 1024,
        'height': 768,
        'samples': 1,
        'description': 'Description of what this test validates'
    }
]
```

The script will attempt to run render_sandbox with these parameters, and fall back to generating synthetic images if GPU access is unavailable (e.g., in CI).

## Directory Structure

```
├── .gitattributes          # Git LFS configuration
├── .github/workflows/
│   ├── visual-diff.yml     # Main comparison workflow
│   └── accept-image.yml    # Image acceptance workflow
├── scripts/
│   └── generate_test_images.py  # Test image generation
├── outputs/                # Generated test images (temporary)
├── diffs/                  # Visual diff images (temporary)
└── golden/                 # Reference images (LFS tracked)
```

## Git LFS Configuration

The project automatically tracks images in specific directories using Git LFS:

```gitattributes
# Track all images in the golden reference directory
golden/**/*.png filter=lfs diff=lfs merge=lfs -text
golden/**/*.jpg filter=lfs diff=lfs merge=lfs -text
golden/**/*.jpeg filter=lfs diff=lfs merge=lfs -text
golden/**/*.bmp filter=lfs diff=lfs merge=lfs -text
```

## FLIP Image Comparison

The system uses NVIDIA FLIP (FLipper Image comParsion) for high-fidelity image comparison. FLIP provides:

- **Perceptual weighting**: Accounts for human visual perception
- **Statistical analysis**: Mean, median, quartile error measurements
- **Visual diff generation**: Creates heatmap images showing differences
- **Configurable thresholds**: Adjustable sensitivity for difference detection

### FLIP Statistics Explained

- **Mean Error**: Average perceptual difference across all pixels
- **Weighted Median**: Median error weighted by perceptual importance
- **Quartiles**: Distribution of errors (1st and 3rd quartile values)
- **Threshold**: Images with mean error > 0.001 are considered "different"

## Troubleshooting

### Common Issues

1. **Images not loading in PR comments**
   - GitHub may need time to process LFS files
   - Wait a moment and refresh the PR

2. **Permission denied on accept**
   - Ensure the user has write access to the repository
   - Only collaborators with write permissions can accept images

3. **Workflow not triggering**
   - Check that the PR has changes that generate new output images
   - Verify `scripts/generate_test_images.py` runs successfully
   - Ensure artifacts are being uploaded correctly

4. **FLIP comparison failures**
   - Usually indicates file format or corruption issues
   - Check that both reference and test images are valid PNG files

### Manual Testing

You can run the test image generation locally:

```bash
# Generate test images
python3 scripts/generate_test_images.py

# Check generated images
ls -la outputs/

# Manual FLIP comparison (if you have FLIP installed)
flip -r golden/basic_render_800x600.png -t outputs/basic_render_800x600.png -d diffs -b diff_basic_render
```

## Security Considerations

- Only users with write permissions can accept images
- All operations are logged and attributed to specific users
- Git LFS ensures large files don't bloat repository history
- Artifacts are automatically cleaned up after 30 days

## Contributing

When contributing visual changes:

1. Make your code changes
2. Run tests locally to verify behavior
3. Open a PR and wait for visual regression results
4. Review any visual differences carefully
5. Accept expected changes using `/accept-image <filename>`
6. Address any unexpected differences before merging

Remember that accepting a visual change updates the reference for all future tests, so review carefully!