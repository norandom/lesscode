//! Sizing spike: a Rust port of the model-free part of the annotation-driver
//! prototype (`validator.py`, `engine.py` without selection, the model-free
//! checks of `pack_check.py`, and the control derivation of `run_controls.py`).

pub mod controls;
pub mod engine;
pub mod pack;
pub mod pack_check;
pub mod scope;
pub mod tree;
pub mod validator;
