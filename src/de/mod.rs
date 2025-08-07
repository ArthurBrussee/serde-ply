mod ply_file;

// Binary deserialization functions. For performance we want to these to be specialized, rather than each
// row having to decide on the ply format (binary vs ascii).
pub mod ascii;
pub mod binary;
