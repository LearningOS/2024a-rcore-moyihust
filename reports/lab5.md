### 实现的功能

实现了死锁检测，创建了 `DeadlockDetect` 结构体，并实现了银行家算法。在系统调用中初始化系统资源和进程资源，实现了资源的申请和释放，并实现了死锁的检测。

### 问答

#### 1. 当主线程 (即 0 号线程) 退出时，视为整个进程退出，此时需要结束该进程管理的所有线程并回收其资源。

- **需要回收的资源有哪些？**
  - 内存资源：包括堆内存、栈内存、内存映射区域等。
  - 文件描述符：关闭所有打开的文件描述符。
  - 线程控制块（TaskControlBlock）：释放所有线程的控制块。
  - 互斥锁、信号量等同步原语：释放所有分配的同步原语。
  - 其他系统资源：如定时器、网络连接等。

- **其他线程的 TaskControlBlock 可能在哪些位置被引用，分别是否需要回收，为什么？**
  - **任务队列**：在任务调度器的任务队列中引用，需要回收，因为这些线程已经不再需要调度。
  - **同步原语的等待队列**：在互斥锁、信号量等同步原语的等待队列中引用，需要回收，因为这些线程已经不再需要等待。
  - **进程控制块（ProcessControlBlock）**：在进程的线程列表中引用，需要回收，因为整个进程已经退出。
  - **其他线程的引用**：如果其他线程持有对这些线程的引用，需要回收，因为这些线程已经不再有效。

#### 2. 对比以下两种 `Mutex.unlock` 的实现，二者有什么区别？这些区别可能会导致什么问题？

```rust
impl Mutex for Mutex1 {
    fn unlock(&self) {
        let mut mutex_inner = self.inner.exclusive_access();
        assert!(mutex_inner.locked);
        mutex_inner.locked = false;
        if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
            add_task(waking_task);
        }
    }
}

impl Mutex for Mutex2 {
    fn unlock(&self) {
        let mut mutex_inner = self.inner.exclusive_access();
        assert!(mutex_inner.locked);
        if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
            add_task(waking_task);
        } else {
            mutex_inner.locked = false;
        }
    }
}
```

- **区别**：
  - `Mutex1`：在解锁时总是将 `locked` 设为 `false`，然后检查等待队列是否有任务需要唤醒。如果有任务需要唤醒，则将其添加到任务队列中。
  - `Mutex2`：在解锁时首先检查等待队列是否有任务需要唤醒。如果有任务需要唤醒，则将其添加到任务队列中。如果没有任务需要唤醒，则将 `locked` 设为 `false`。

- **可能导致的问题**：
  - `Mutex1`：在解锁时总是将 `locked` 设为 `false`，这可能导致在有任务需要唤醒时，锁的状态不一致（即锁已经被释放，但任务还没有被唤醒）。
  - `Mutex2`：在解锁时只有在没有任务需要唤醒时才将 `locked` 设为 `false`，这可以确保锁的状态一致（即只有在没有任务需要唤醒时，锁才被释放）。

### 荣耀准则

1. 在完成本次实验的过程中，我曾分别与以下各位就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

2. 此外，我也参考了以下资料，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：
   - https://sazikk.github.io/posts/%E7%AC%94%E8%AE%B0-rCoreLab%E5%AE%9E%E9%AA%8C%E7%AC%94%E8%AE%B0

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。

Similar code found with 3 license types