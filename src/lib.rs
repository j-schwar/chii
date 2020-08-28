#![feature(bindings_after_at)]

pub mod bit;
pub mod comp;
pub mod data;
pub mod int;
pub mod math;
pub mod schema;
pub mod vie;

mod encode;

pub use encode::encode;
