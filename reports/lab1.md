## 实现的功能
在TCB中加入start_time和syscall_times字段，并在trap.rs中更新syscall_times，在taskManager的run_first_task和run_next_task中维护start_time字段，最后在syscall中通过get_current_task_control_block实现返回task_info


## 问答题
1. 正确进入 U 态后，程序的特征还应有：使用 S 态特权指令，访问 S 态寄存器后会报错。 请同学们可以自行测试这些内容（运行 三个 bad 测例 (ch2b_bad_*.rs) ）， 描述程序出错行为，同时注意注明你使用的 sbi 及其版本。

    使用的SBI版本RustSBI version 0.3.0-alpha.4, adapting to RISC-V SBI v1.0.0。
    ch2b_bad_address.rs导致发生页错误/
    ```
    [kernel] Loading app_0
    [kernel] PageFault in application, kernel killed it.
    ```
    ch2b_bad_instructions.rs导致非法指令错误
    ch2b_bad_register.rs试图访问特权寄存器导致非法指令错误
    ```
    [kernel] Loading app_1
    [kernel] IllegalInstruction in application, kernel killed it.
    [kernel] Loading app_2
    [kernel] IllegalInstruction in application, kernel killed it.
    ```

2. 深入理解 trap.S 中两个函数 __alltraps 和 __restore 的作用，并回答如下问题:

    1. L40：刚进入 __restore 时，a0 代表了什么值。请指出 __restore 的两种使用情景。

        - 刚进入 __restore 时，a0 代表的是用户态的栈指针。__restore 有两种使用情景：一是从 S 态返回 U 态，二是从中断或异常处理返回用户态。

    2. L43-L48：这几行汇编代码特殊处理了哪些寄存器？这些寄存器的的值对于进入用户态有何意义？请分别解释。

        ``` asm
        ld t0, 32*8(sp)
        ld t1, 33*8(sp)
        ld t2, 2*8(sp)
        csrw sstatus, t0
        csrw sepc, t1
        csrw sscratch, t2
        ```
        - 这些代码处理了 sstatus、sepc 和 sscratch 寄存器：
        - sstatus：保存了当前的状态信息，包括中断使能位等。
        - sepc：保存了异常或中断发生时的程序计数器值，返回用户态时需要恢复。
        - sscratch：保存了临时数据，通常用于保存内核栈指针。
    3. L50-L56：为何跳过了 x2 和 x4？

        ````
        ld x1, 1*8(sp)
        ld x3, 3*8(sp)
        .set n, 5
        .rept 27
        LOAD_GP %n
        .set n, n+1
        .endr
        ````
        - 跳过 x2 和 x4 是因为 x2 是栈指针（sp），已经在其他地方处理，而 x4 通常是保留寄存器，不需要恢复。
    4. L60：该指令之后，sp 和 sscratch 中的值分别有什么意义？
        ```
        csrrw sp, sscratch, sp
        ```
        - 该指令之后，sp 和 sscratch 的值互换。sp 恢复为用户态的栈指针，而 sscratch 保存了内核态的栈指针。
    5. __restore：中发生状态切换在哪一条指令？为何该指令执行之后会进入用户态？
        - 状态切换发生在 sret 指令。执行 sret 后，处理器会根据 sstatus 寄存器中的状态位切换到用户态，并跳转到 sepc 指定的地址继续执行

    6. L13：该指令之后，sp 和 sscratch 中的值分别有什么意义？

        ```
        csrrw sp, sscratch, sp
        ```
        - 该指令之后，sp 和 sscratch 的值互换。sp 恢复为用户态的栈指针，而 sscratch 保存了内核态的栈指针。
    7. 从 U 态进入 S 态是哪一条指令发生的？
        - 从 U 态进入 S 态通常是通过触发异常或中断来实现的，具体的指令是 ecall 

## 荣誉准则
1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

    陈志鹏：队员 关于test sys_time部分交流

2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

    https://learningos.cn/rCore-Camp-Guide-2024A/honorcode.html
    https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter3/6answer.html


3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。