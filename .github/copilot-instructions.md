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
- Visual regression tests run on GPU instances but include fallback image generation when GPU access is unavailable
- Fallback images are synthetic test images that maintain visual regression testing workflow functionality in environments without GPU

### Test Organization
- **GPU-requiring tests**: Use `#[cfg(feature = "gpu-tests")]` attribute and run with `--features gpu-tests`
- **Standard CI tests**: Run without the `gpu-tests` feature on `ubuntu-latest` runners
- **GPU tests include**: Visual regression tests, rendering tests, GPU-dependent integration tests
- **Standard tests include**: Unit tests, CLI parsing, architecture tests, build validation
- The visual-diff workflow runs GPU tests on self-hosted instances with the `gpu-tests` feature enabled
- The standard CI workflow runs non-GPU tests on GitHub Actions standard runners

### Visual Regression Testing
- GPU tests run via `cargo test --features gpu-tests` on self-hosted GPU instances, including visual regression tests that generate images
- Visual regression tests prefer real GPU rendering but fall back to synthetic images when GPU access is unavailable
- The visual-diff workflow runs on pull requests targeting the main branch (`pull_request: branches: [ main ]`)
- The workflow separates GPU test execution from image comparison using upstream workflows:
  - **generate-images job**: Runs on self-hosted GPU instances (`runs-on: [self-hosted, linux, x64]`) to run all GPU tests and generate test images (real or fallback)
  - **call-visual-diff job**: Calls the upstream `mpiispanen/image-comparison-and-update/.github/workflows/visual-diff.yml@main` workflow
- GPU tests include visual regression tests, render tests, and GLTF tests that require GPU access
- Visual regression tests call the render_sandbox binary with appropriate parameters to generate test images in the `outputs/` directory
- When GPU is unavailable, fallback synthetic images are generated to ensure CI workflow completion
- Image comparison uses the upstream workflow which handles NVIDIA FLIP comparison and PR reporting
- The upstream workflow handles image display, diff generation, and acceptance commands (`/accept-image filename.png`)
- Test images are uploaded as artifacts and passed to the upstream comparison workflow
- **Critical**: The visual-diff workflow must be configured to trigger on pull requests targeting main to ensure all GPU tests run as part of the PR flow

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