### 实现的功能

在本次实验中，我们将当前运行程序的处理从 `taskmanager` 中分离出来，交由 `processor` 处理，并使用修改后的接口完成 `ch3-4` 的系统调用。我们仿照 `exec` 和 `fork` 实现了 `spawn` 流程，并创建了一个新的地址空间。此外，我们在 PCB 中添加了 `priority` 和 `stride` 字段，并在选择时遍历选择 `stride` 最小的进程。

### 问答

#### Stride 算法深入

Stride 算法的原理非常简单，但存在一个较大的问题。例如，两个 `pass = 10` 的进程，使用 8 位无符号整形储存 `stride`，假设 `p1.stride = 255`，`p2.stride = 250`，在 `p2` 执行一个时间片后，理论上下一次应该是 `p1` 执行。

**实际情况是轮到 `p1` 执行吗？为什么？**

实际情况是不会轮到 `p1` 执行。因为 `p1.stride` 和 `p2.stride` 都接近最大值 255，当 `p2` 执行一个时间片后，其 `stride` 值会溢出并变为较小的值，从而导致 `p2` 继续执行。

我们之前要求进程优先级 `>= 2` 其实就是为了解决这个问题。可以证明，在不考虑溢出的情况下，在进程优先级全部 `>= 2` 的情况下，如果严格按照算法执行，那么 `STRIDE_MAX – STRIDE_MIN <= BigStride / 2`。

**为什么？尝试简单说明（不要求严格证明）。**

这是因为在优先级 `>= 2` 的情况下，每个进程的 `stride` 增长速度相对较慢，且不同进程之间的 `stride` 差距不会超过 `BigStride / 2`。这样可以避免溢出导致的错误调度。

已知以上结论，考虑溢出的情况下，可以为 `Stride` 设计特别的比较器，让 `BinaryHeap<Stride>` 的 `pop` 方法能返回真正最小的 `Stride`。补全下列代码中的 `partial_cmp` 函数，假设两个 `Stride` 永远不会相等。

```rust
use core::cmp::Ordering;

struct Stride(u64);

impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let half_max = u64::MAX / 2;
        if self.0 < half_max && other.0 > half_max {
            Some(Ordering::Less)
        } else if self.0 > half_max && other.0 < half_max {
            Some(Ordering::Greater)
        } else {
            self.0.partial_cmp(&other.0)
        }
    }
}

impl PartialEq for Stride {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}
```

### 解释

1. **`partial_cmp` 函数**：
   - 计算 `half_max`，即 `u64::MAX / 2`。
   - 如果 `self.0` 小于 `half_max` 且 `other.0` 大于 `half_max`，则返回 `Ordering::Less`。
   - 如果 `self.0` 大于 `half_max` 且 `other.0` 小于 `half_max`，则返回 `Ordering::Greater`。
   - 否则，使用默认的 `partial_cmp` 比较。

2. **`eq` 函数**：
   - 假设两个 `Stride` 永远不会相等，因此返回 `false`。

### 荣耀准则

1. 在完成本次实验的过程中，我曾分别与以下各位就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

2. 此外，我也参考了以下资料，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。