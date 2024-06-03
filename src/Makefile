obj-m := e1000_rs.o
e1000_rs-y := linux/e1000_main.o platform_linux.o 


clean:
	make LLVM=1 -C ../../linux M=$(PWD) clean