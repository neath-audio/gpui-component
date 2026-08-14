//! Reusable visual language owned by the styled `gpui-neath` layer.
//!
//! `gpui-base` owns behavior, state, and required geometry. This module owns
//! Neath's reusable visual choices and must not depend on application crates.

pub mod recipes;
pub mod tokens;
pub mod typography;
