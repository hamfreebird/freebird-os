构建并运行：
```shell
cargo bootimage
qemu-system-x86_64 -drive format=raw,file=target/x86_64-freebird-os/debug/bootimage-freebird-os.bin
```