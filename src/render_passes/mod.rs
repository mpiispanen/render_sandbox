//! Render pass implementations for the render graph system
//!
//! This module contains all concrete render pass implementations that can be
//! added to the render graph for execution.

mod forward_pass;
mod placeholder_pass;

pub use forward_pass::ForwardRenderPass;
pub use placeholder_pass::PlaceholderPass;
