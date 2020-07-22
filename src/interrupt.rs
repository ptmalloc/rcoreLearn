use riscv::register::{
    scause::{
        self,
        Trap,
        Exception,
        Interrupt
    },
    sepc,
    stvec,
    sscratch,
    sstatus
};
use crate::timer::{
    TICKS,
    clock_set_next_event
};


use crate::context::TrapFrame;

global_asm!(include_str!("trap/trap.asm"));

pub fn init() {
    unsafe {
        extern "C" {
            fn __alltraps();
        }        
        sscratch::write(0);
        stvec::write(__alltraps as usize, stvec::TrapMode::Direct);
        sstatus::set_sie();
    }
    println!("++++ setup interrupt! ++++");
}

#[no_mangle]
pub fn rust_trap(tf: &mut TrapFrame) {
    // 根据中断原因分类讨论
    match tf.scause.cause() {
        // 断点中断
        Trap::Exception(Exception::Breakpoint) => breakpoint(&mut tf.sepc),
        // S态时钟中断
        Trap::Interrupt(Interrupt::SupervisorTimer) => super_timer(),
        _ => panic!("undefined trap!")
    }
}

// 断点中断处理：输出断点地址并改变中断返回地址防止死循环
fn breakpoint(sepc: &mut usize) {
    println!("a breakpoint set @0x{:x}", sepc);
    *sepc += 2;
}

// S态时钟中断处理
fn super_timer() {
    // 设置下一次时钟中断触发时间
    clock_set_next_event();
    unsafe {
        // 更新时钟中断触发计数
        // 注意由于 TICKS 是 static mut 的
        // 后面会提到，多个线程都能访问这个变量
        // 如果同时进行 +1 操作，会造成计数错误或更多严重bug
        // 因此这是 unsafe 的，不过目前先不用管这个
        TICKS += 1;
        // 每触发 100 次时钟中断将计数清零并输出
        if (TICKS == 100) {
            TICKS = 0;
            println!("* 100 ticks *");
        }
    }
    // 发生外界中断时，epc 指向的指令没有完成执行，因此这里不需要修改 epc
}
