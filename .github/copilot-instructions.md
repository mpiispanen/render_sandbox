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
- All tests that require GPU access should run on RunOn AWS GPU instances using the best value GPU node
- Use RunOn label format: `runs-on: [runson, gpu-4x-large]` for GPU-dependent workflows
- Non-GPU tests (builds, linting, unit tests) should run on regular GitHub Actions runners: `runs-on: ubuntu-latest`
- Visual regression tests MUST run on GPU instances and should fail if GPU access is unavailable
- Do not implement synthetic/fallback image generation for GPU tests - real GPU rendering is required

### Visual Regression Testing
- Visual regression tests generate images using `cargo test` (specifically the `generate_visual_regression_images` test)
- The CI workflow accepts these pre-generated images and performs comparison against golden masters
- Tests call the render_sandbox binary with appropriate parameters to generate test images
- This follows the upstream pattern where external applications generate images and CI performs comparison

### RunOn Configuration Guidelines
- GPU Instance Selection: Use `gpu-4x-large` for the best value when running visual regression tests
- Non-GPU workflows should continue using standard GitHub Actions runners
- Ensure proper resource allocation by separating GPU-dependent from CPU-only workflows

## Key Guidelines
1. Follow Rust best practices and idiomatic patterns
2. Maintain existing code structure and organization
4. Write unit tests for new functionality.
5. Write integration tests to make sure the overall application and rendering logic works as expected.
6. Document public APIs and complex logic.
7. Avoid adding trivial comments to code. Comment on the why rather than the what unless the code is very complex. 
8. Variable and function naming should make code readable so that in most cases trivial comments are unnecessary.