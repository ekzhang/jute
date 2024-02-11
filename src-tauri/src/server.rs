//! Code that interfaces directly with kernels to be jupyter-server compatible.
//!
//! This is currently unused while Jute relies on jupyter-server, but in the
//! future it could replace the Jupyter installation by directly invoking
//! kernels, or introduce new APIs for developer experience.

pub mod environment;
pub mod kernel;
