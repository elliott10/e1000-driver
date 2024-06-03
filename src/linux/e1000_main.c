#include<linux/init.h>
#include<linux/module.h>
#include "e1000.h"

static size_t __ptr  =0;
 
int e1000_init(void)
{
    printk("my_dev_init\n");
    __ptr = e1000_new();   
    
    printk("dev ptr: %d \n", __ptr);
    e1000_probe(__ptr);

    return 0;
}
 
void e1000_exit(void)
{
    printk("my_dev_exit : %d \n", 0);


    return;
}
 
module_init(e1000_init);
module_exit(e1000_exit);


MODULE_AUTHOR("zhourui <zhourui818@126.com>");
MODULE_DESCRIPTION("e1000 extensions support");
MODULE_LICENSE("GPL");