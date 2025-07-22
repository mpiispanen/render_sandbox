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
- Visual regression tests generate images using `cargo test` (specifically the `generate_visual_regression_images` test)
- Image generation AND comparison must happen on the same GPU-enabled runner to ensure generated images are available for comparison
- The visual-diff workflow runs entirely on self-hosted GPU instances: generates images, then immediately compares them against golden masters on the same machine
- Tests call the render_sandbox binary with appropriate parameters to generate test images
- Image comparison uses NVIDIA FLIP for high-fidelity perceptual comparison, generating diff images and statistics
- Results are uploaded as artifacts and committed to temporary branches for PR display

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