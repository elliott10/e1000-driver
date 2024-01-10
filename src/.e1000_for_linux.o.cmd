cmd_/root/kernel_dev/drivers/e1000-driver/src/linux/../e1000_for_linux.o := RUST_MODFILE=/root/kernel_dev/drivers/e1000-driver/src/linux/../e1000_for_linux rustc --edition=2021 -Zbinary_dep_depinfo=y -Dunsafe_op_in_unsafe_fn -Drust_2018_idioms -Dunreachable_pub -Dnon_ascii_idents -Wmissing_docs -Drustdoc::missing_crate_level_docs -Dclippy::correctness -Dclippy::style -Dclippy::suspicious -Dclippy::complexity -Dclippy::perf -Dclippy::let_unit_value -Dclippy::mut_mut -Dclippy::needless_bitwise_bool -Dclippy::needless_continue -Wclippy::dbg_macro --target=./rust/target.json -Cpanic=abort -Cembed-bitcode=n -Clto=n -Cforce-unwind-tables=n -Ccodegen-units=1 -Csymbol-mangling-version=v0 -Crelocation-model=static -Zfunction-sections=n -Dclippy::float_arithmetic -Ctarget-feature=-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-avx,-avx2 -Ztune-cpu=generic -Cno-redzone=y -Ccode-model=kernel -Copt-level=2 -Cdebug-assertions=n -Coverflow-checks=y -Cdebuginfo=2  --cfg MODULE  @./include/generated/rustc_cfg -Zallow-features=allocator_api,const_refs_to_cell -Zcrate-attr=no_std -Zcrate-attr='feature(allocator_api,const_refs_to_cell)' --extern alloc --extern kernel --crate-type rlib --out-dir /root/kernel_dev/drivers/e1000-driver/src/linux/.. -L ./rust/ --crate-name e1000_for_linux --emit=dep-info,obj /root/kernel_dev/drivers/e1000-driver/src/linux/../e1000_for_linux.rs; mv /root/kernel_dev/drivers/e1000-driver/src/linux/../e1000_for_linux.d /root/kernel_dev/drivers/e1000-driver/src/linux/../.e1000_for_linux.o.d; sed -i '/^$(pound)/d' /root/kernel_dev/drivers/e1000-driver/src/linux/../.e1000_for_linux.o.d

source_/root/kernel_dev/drivers/e1000-driver/src/linux/../e1000_for_linux.o := /root/kernel_dev/drivers/e1000-driver/src/linux/../e1000_for_linux.rs

deps_/root/kernel_dev/drivers/e1000-driver/src/linux/../e1000_for_linux.o := \
  /root/kernel_dev/drivers/e1000-driver/src/linux/../linux/mod.rs \
  /root/kernel_dev/drivers/e1000-driver/src/linux/../linux/volatile.rs \
  /root/kernel_dev/drivers/e1000-driver/src/linux/../linux/print.rs \
  /root/kernel_dev/drivers/e1000-driver/src/linux/../e1000/mod.rs \
  /root/kernel_dev/drivers/e1000-driver/src/linux/../e1000/e1000.rs \
  /root/kernel_dev/drivers/e1000-driver/src/linux/../e1000/e1000_const.rs \
  /root/kernel_dev/drivers/e1000-driver/src/linux/../utils.rs \
  /root/kernel_dev/linux-rust-v6.1.66/rust/libcore.rmeta \
  /root/kernel_dev/linux-rust-v6.1.66/rust/libcompiler_builtins.rmeta \
  /root/kernel_dev/linux-rust-v6.1.66/rust/libkernel.rmeta \
  /root/kernel_dev/linux-rust-v6.1.66/rust/liballoc.rmeta \
  /root/kernel_dev/linux-rust-v6.1.66/rust/libmacros.so \
  /root/kernel_dev/linux-rust-v6.1.66/rust/libbindings.rmeta \
  /root/kernel_dev/linux-rust-v6.1.66/rust/libbuild_error.rmeta \
  /root/kernel_dev/drivers/e1000-driver/src/linux/../linux/mod.rs \
  /root/kernel_dev/drivers/e1000-driver/src/linux/../linux/volatile.rs \
  /root/kernel_dev/drivers/e1000-driver/src/linux/../linux/print.rs \
  /root/kernel_dev/drivers/e1000-driver/src/linux/../e1000/mod.rs \
  /root/kernel_dev/drivers/e1000-driver/src/linux/../e1000/e1000.rs \
  /root/kernel_dev/drivers/e1000-driver/src/linux/../e1000/e1000_const.rs \
  /root/kernel_dev/drivers/e1000-driver/src/linux/../utils.rs \
  /root/kernel_dev/linux-rust-v6.1.66/rust/libcore.rmeta \
  /root/kernel_dev/linux-rust-v6.1.66/rust/libcompiler_builtins.rmeta \
  /root/kernel_dev/linux-rust-v6.1.66/rust/libkernel.rmeta \
  /root/kernel_dev/linux-rust-v6.1.66/rust/liballoc.rmeta \
  /root/kernel_dev/linux-rust-v6.1.66/rust/libmacros.so \
  /root/kernel_dev/linux-rust-v6.1.66/rust/libbindings.rmeta \
  /root/kernel_dev/linux-rust-v6.1.66/rust/libbuild_error.rmeta \

/root/kernel_dev/drivers/e1000-driver/src/linux/../e1000_for_linux.o: $(deps_/root/kernel_dev/drivers/e1000-driver/src/linux/../e1000_for_linux.o)

$(deps_/root/kernel_dev/drivers/e1000-driver/src/linux/../e1000_for_linux.o):
