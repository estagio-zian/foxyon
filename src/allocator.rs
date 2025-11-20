#[cfg(all(feature = "mimalloc", feature = "snmalloc"))]
compile_error!("Cannot enable multiple allocators simultaneously");

#[cfg(feature = "snmalloc")]
use snmalloc_rs::SnMalloc;
#[cfg(feature = "snmalloc")]
#[global_allocator]
static GLOBAL: SnMalloc = SnMalloc;

#[cfg(feature = "mimalloc")]
use mimalloc::MiMalloc;
#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;