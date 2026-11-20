//! The glTF constants a model is read with: artkit's (which writes them), as wide as the reader's fields,
//! and those only reading needs.

use artkit::glb;

pub(crate) const FLOAT: u64 = glb::FLOAT as u64;
pub(crate) const BYTE: u64 = 5120;
pub(crate) const UBYTE: u64 = 5121;
pub(crate) const SHORT: u64 = 5122;
pub(crate) const USHORT: u64 = glb::USHORT as u64;
pub(crate) const UINT: u64 = glb::UINT as u64;
pub(crate) const ARRAY_BUFFER: u64 = glb::ARRAY_BUFFER as u64;
pub(crate) const ELEMENT_ARRAY_BUFFER: u64 = glb::ELEMENT_ARRAY_BUFFER as u64;
pub(crate) const MESHOPT: &str = "EXT_meshopt_compression";
pub(crate) const QUANTIZATION: &str = "KHR_mesh_quantization";
