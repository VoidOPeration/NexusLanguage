// Meta-programming helpers
#[repr(C)]
pub struct NxType {
    pub size: usize,
    pub align: usize,
    pub name: *const i8,
}