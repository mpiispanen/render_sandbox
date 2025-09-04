This is a Rust based repository using wgpu for graphics rendering, but the rendering interface should be agnostic of the rendering backend API for future expendability. The main purpose of the application is to serve as a flexible sandbox for graphics rendering research and testing. Please follow these guidelines when contributing:

## Code Standards

### Required Before Each Commit
- Run `cargo fmt` before committing any changes to ensure proper code formatting
- This will run cargo-fmt on all Rust files to maintain consistent style
- Run `cargo check` and `cargo clippy` and make sure we have no errors or undesired warnings left in our code before committing, or otherwise document why they are left and acceptable for committing

### Development Flow
- Build: `cargo build`
- Test: `cargo test`
- Make sure CI tests pass and fix errors before committing if they do not

## CI Infrastructure

### GPU Testing Requirements
- All tests that require GPU access should run on self-hosted GPU instances using our x64 Linux node
- Use self-hosted runner configuration: `runs-on: [self-hosted, linux, x64]` for GPU-dependent workflows
- Non-GPU tests (builds, linting, unit tests) should run on regular GitHub Actions runners: `runs-on: ubuntu-latest`
- Visual regression tests MUST run on GPU instances and should fail if GPU access is unavailable
- Do not implement synthetic/fallback image generation for GPU tests - real GPU rendering is required

### Test Organization
- **GPU-requiring tests**: Use `#[cfg(feature = "gpu-tests")]` attribute and run with `--features gpu-tests`
- **Standard CI tests**: Run without the `gpu-tests` feature on `ubuntu-latest` runners
- **GPU tests include**: Visual regression tests, rendering tests, GPU-dependent integration tests
- **Standard tests include**: Unit tests, CLI parsing, architecture tests, build validation
- The visual-diff workflow runs GPU tests on self-hosted instances with the `gpu-tests` feature enabled
- The standard CI workflow runs non-GPU tests on GitHub Actions standard runners

### Visual Regression Testing

The repository implements a comprehensive visual regression testing system with both pre-commit (PR) and post-commit (main branch) validation:

#### Pre-Commit Visual Testing (Pull Requests)
- GPU tests run via `cargo test --features gpu-tests` on self-hosted GPU instances, including visual regression tests that generate images
- The visual-diff workflow runs on pull requests targeting the main branch (`pull_request: branches: [ main ]`)
- The workflow separates GPU test execution from image comparison using upstream workflows:
  - **generate-images job**: Runs on self-hosted GPU instances (`runs-on: [self-hosted, linux, x64]`) to run all GPU tests and generate test images
  - **call-visual-diff job**: Calls the upstream `mpiispanen/image-comparison-and-update/.github/workflows/visual-diff.yml@main` workflow
- GPU tests include visual regression tests, render tests, and GLTF tests that require GPU access
- Visual regression tests call the render_sandbox binary with appropriate parameters to generate test images in the `outputs/` directory
- Image comparison uses the upstream workflow which handles NVIDIA FLIP comparison and PR reporting
- The upstream workflow handles image display, diff generation, and acceptance commands (`/accept-image filename.png`)
- Test images are uploaded as artifacts and passed to the upstream comparison workflow

#### Render Pass Testing Requirements
- **All render passes MUST have individual visual tests**: Each render pass (ForwardRenderPass, etc.) must have dedicated tests that verify image correctness
- **Render pass tests must generate specific images**: Tests should create distinct visual outputs that can be compared against golden masters
- **Individual pass isolation**: Tests should verify each render pass works correctly in isolation as well as in the full pipeline
- **Visual test naming**: Render pass tests should use descriptive names like `forward_pass_basic_triangle`, `forward_pass_antialiased`, etc.
- **Pipeline abstraction testing**: All render pass tests must verify the new pipeline abstraction system works correctly

#### Post-Commit Visual Testing (Main Branch Validation)
- Post-commit visual regression testing is implemented via `.github/workflows/post-commit-visual-regression.yml`
- Uses upstream `mpiispanen/image-comparison-and-update/.github/workflows/post-commit-visual-validation.yml@main`
- Triggered automatically on every push to main branch
- Creates GitHub issues for visual regression failures instead of PR comments
- Supports automatic golden master updates with configurable thresholds
- Automatically closes resolved issues when visual tests pass again
- **Failure Handling**: Creates detailed GitHub issues with FLIP error analysis and troubleshooting steps
- **Auto-Update Support**: Optional automatic golden master updates with safety checks and audit trails
- **Issue Management**: Smart deduplication and automatic closure of resolved issues

#### Workflow Separation
- **CI Workflow**: Handles standard build, unit tests, and linting for both PRs and main branch pushes
- **Visual-Diff Workflow**: Handles visual regression testing for pull requests with PR comment integration
- **Post-Commit Visual Regression**: Handles visual validation after merge to main with GitHub issue management
- **Critical**: All visual workflows are properly configured to ensure comprehensive visual regression coverage

### Self-Hosted Runner Configuration Guidelines
- GPU Instance Selection: Use self-hosted runner format `runs-on: [self-hosted, linux, x64]` for GPU-dependent workflows running visual regression tests
- Non-GPU workflows should continue using standard GitHub Actions runners: `runs-on: ubuntu-latest`
- Ensure proper resource allocation by separating GPU-dependent from CPU-only workflows
- Self-hosted runners provide dedicated GPU access for rendering tests that require real hardware

## Key Guidelines
1. Follow Rust best practices and idiomatic patterns
2. Maintain existing code structure and organization
4. Write unit tests for new functionality.
5. Write integration tests to make sure the overall application and rendering logic works as expected.
6. Document public APIs and complex logic.
7. Avoid adding trivial comments to code. Comment on the why rather than the what unless the code is very complex. 
8. Variable and function naming should make code readable so that in most cases trivial comments are unnecessary.