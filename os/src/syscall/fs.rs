//! File and filesystem-related syscalls

const FD_STDOUT: usize = 1;

/// write buf of length `len`  to a file with `fd`
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    if !crate::batch::checkmem(buf as usize, len) {
        return -1;
    }

    match fd {
        FD_STDOUT => {
            // println!("FD_STDOUT");
            let slice = unsafe { core::slice::from_raw_parts(buf, len) };
            let str = core::str::from_utf8(slice).unwrap();
            print!("{}", str);
            len as isize
        }
        _ => {
            println!("Unsupported fd: [{:}] in sys_write!", fd);
            -1
        }
    }
}
