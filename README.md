# rCore

An Unix like Operating System kernel written with Rust based on RISC-V.

## Run the OS

```shell
cd os/ && make run
```

## Dev Environment

- Ubuntu 22.04 LTS
- Rust toolchain (nightly)
- QEMU emulator: qemu-system-riscv64 (version 7.0.0)
- GDB: riscv64-unknown-elf-gdb (GNU gdb (SiFive GDB 8.3.0-2020.04.1) 8.3)

Get more information about development environment from [setup-devel-env](http://rcore-os.cn/rCore-Tutorial-Book-v3/chapter0/5setup-devel-env.html#)(Simplified Chinese)

## Working in progress

- [x] LibOS: Make the application isolated from the hardware, simplify the difficulty and complexity of the application to access the hardware.
- [x] BatchOS： Isolate applications from the operating system to strengthen system security and improve execution efficiency.
- [x] Multiprog & Time-sharing OS: Share CPU resource between multiply applications.
- [ ] Address Space OS: Isolate the memory address space accessed by the application, limit the mutual interference between the application, and improve security.
- [ ] Process OS: Allows application dynamically creates new process and enhance process management and resource management capabilities.
- [ ] Filesystem OS：Allows applications to store data persistently.
- [ ] IPC OS：Allows multiple apps to interact with data and event notifications between processes.
- [ ] Thread & Coroutine OS：Implement threading and coroutine applications to simplify switching and data sharing.
- [ ] SyncMutex OS：Supports synchronous and exclusive access to shared resources in multithreaded applications.
- [ ] Device OS：Improve the I/O efficiency and human-computer interaction ability of the application, and support serial ports/block devices/keyboards/mice/display devices based on peripheral interrupts.

Run `git checkout chx` to view these Operating System.
| Operating System            | Git branch |
| --------------------------- | :--------: |
| LibOS                       |    ch1     |
| BatchOS                     |    ch2     |
| Multiprog & Time-sharing OS |    ch3     |
| Address Space OS            |            |
| Process OS                  |            |
| Filesystem OS               |            |
| IPC OS                      |            |
| Thread & Coroutine OS       |            |
| SyncMutex OS                |            |
| Device OS                   |            |

## Reference

- [rCore-Tutorial-Book 第三版](https://rcore-os.github.io/rCore-Tutorial-Book-v3/index.html)(Simplified Chinese) and its [Github repo](https://github.com/rcore-os/rCore-Tutorial-v3)
- [rCore-Tutorial-Guide 2022 春季学期](https://learningos.github.io/rCore-Tutorial-Guide-2022S/index.html)(Simplified Chinese)
- [some resource to learning RISC-V](https://github.com/rcore-os/rCore/wiki/os-tutorial-summer-of-code-2021#step-1-%E8%87%AA%E5%AD%A6risc-v%E7%B3%BB%E7%BB%9F%E7%BB%93%E6%9E%84%E5%A4%A7%E7%BA%A67%E5%A4%A9)(Simplified Chinese)
