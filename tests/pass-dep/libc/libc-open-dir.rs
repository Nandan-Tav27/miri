//@ignore-target: windows # no libc on Windows targets
//@compile-flags: -Zmiri-disable-isolation

// Reproducer for <https://github.com/rust-lang/miri/issues/5084>.
//
// On Unix, opening a directory with `O_RDONLY` succeeds and returns a valid fd;
// the error only shows up later when you try to `read` from it, which fails with
// `EISDIR`. Miri delegates `open` to the host `std`, so on a *Windows host* the
// open itself currently fails with `PermissionDenied` instead, because Windows
// won't hand out a directory handle without `FILE_FLAG_BACKUP_SEMANTICS`.
//
// This test exercises the expected Unix behaviour so it passes on Unix hosts and
// (once the shim is fixed) on Windows hosts too.

use std::ffi::CString;
use std::fs::create_dir;

#[path = "../../utils/mod.rs"]
mod utils;

#[path = "../../utils/libc.rs"]
mod libc_utils;

use libc_utils::errno_result;

fn main() {
    let dir_path = utils::prepare_dir("miri_test_open_dir_5084");
    create_dir(&dir_path).unwrap();
    let dir_name = CString::new(dir_path.into_os_string().into_encoded_bytes()).unwrap();

    // Opening a directory read-only should succeed.
    let fd = errno_result(unsafe { libc::open(dir_name.as_ptr(), libc::O_RDONLY) }).unwrap();

    // Reading from the directory fd should then fail with EISDIR.
    let mut buf = [0u8; 4];
    let err =
        errno_result(unsafe { libc::read(fd, buf.as_mut_ptr().cast(), buf.len()) }).unwrap_err();
    assert_eq!(err.raw_os_error().unwrap(), libc::EISDIR, "unexpected errno: {err}");

    libc_utils::errno_check(unsafe { libc::close(fd) });
}
