# Octoscript

[English](README.md) | 简体中文

> **要开发 OctoSense 应用？** 你不需要这个仓库：它是独立的 OctoScript 语言运行时（VM、能力、工作流、LSP）。应用的 `main.splash` 运行在应用工具为你准备好的 Makepad 运行时上，应用可用的语言与 API 见 [SCRIPT-API](https://github.com/OctoSense-org/OctoScript-App-Design-Flow/blob/main/docs/SCRIPT-API.md)。请按 [OctoSense 组织主页](https://github.com/OctoSense-org)给出的顺序阅读：[OctoScript-App-Design-Flow `AGENTS.md`](https://github.com/OctoSense-org/OctoScript-App-Design-Flow/blob/main/AGENTS.md) → [`flows/README.md`](https://github.com/OctoSense-org/OctoScript-App-Design-Flow/blob/main/flows/README.md) → [`docs/QUICKSTART.md`](https://github.com/OctoSense-org/OctoScript-App-Design-Flow/blob/main/docs/QUICKSTART.md)。

Octoscript 是一个以能力（capability）为先的脚本运行时，面向动态工作流、工具编排和数据转换。它以 Makepad Octoscript VM 为起点，把 UI 支持作为可选项，而不是让 UI 成为语言的边界。

## 当前基线

- 一个保留上游来源的独立 VM 与解析器，以固定版本的 git 依赖形式引用 `OctoSense-org/makepad` 的 `octoscript` 分支。
- 一个无副作用、有界的规范语言预检，为生成的源码和编辑器工具提供结构化诊断；并为继承下来的 VM 提供按 token 感知的降级处理，把规范的换行语句边界转换过去。
- 一个无副作用的规范格式化器，在规范化合法 Octoscript 源码的同时保留注释和字面量写法，服务于 LLM 和编辑器工作流。
- 一个有界、感知语法的词法符号索引，覆盖导入、函数、局部绑定、参数和循环绑定，无需对源码求值。
- 在表达式标识符处提供有界的同文档词法补全：候选项感知作用域，编辑为精确 token 替换；对精确可见的 `use mod.tool` 绑定给出固定的 `mod.tool` 成员建议；可选的有界、可刷新的建议性工具目录投影，用于直接的工具名字面量；可选的可刷新模块接口投影，用于直接导入路径和有界的链式导入模块成员；此外还支持有界的直接字面量记录字段补全、悬停和跳转定义，经由精确的直接子字面量和有界的别名路径实现，不做运行时类型推断。
- 一个无副作用的逐步工作流审查，在宿主签发有序的能力租约之前，把语法状态与直接工具调用提示配对呈现。
- 一个有界、纯数据的 workflow-draft JSON 格式和 CLI 审查路径，用于在宿主创建受信计划或授予权限之前审查 LLM 生成的计划。
- 与审批绑定的有界 JSON 工作流数据流：宿主输入和已完成步骤的输出仅作为数据注入，始终受租约约束，且从不复制到工作流遥测中。
- 可选的宿主自有数据流 schema 契约，在后续步骤的权限生效之前校验输入和每一个已完成步骤的输出，并把其摘要绑定进感知契约的数据流检查点。
- 一个仅供宿主使用的 stdio 语言服务器，提供规范语法诊断、全文档格式化编辑、顶层声明符号、同文档的词法定义、引用、绑定类型悬停和符号高亮，词法补全（包括固定的 `mod.tool` API 以及可选的建议性工具和模块元数据），有界的固定/建议性能力签名帮助，一层有界的直接模块输出对象子级，以及与版本绑定、带防护的重命名；全程不读取文件，也不对代码求值。
- 默认的运行时与能力宿主求值会在任何工具运行之前拒绝非规范的 Makepad 兼容语法。
- 独立运行时初始化会在源码求值之前屏蔽继承自 Makepad 的 UI/调试入口和无界原生入口，使 Octoscript 只能访问文档中列出的核心以及受信宿主安装的模块。
- 冻结且不带任何权限的 `mod.std.math` 标量辅助函数、`mod.std.json` 有界 JSON 辅助函数、`mod.std.text` 有界文本辅助函数、`mod.std.array` 有界浅层数组辅助函数，以及 `mod.std.object` 有界自有字段记录辅助函数，满足常见数据流需求，既不恢复 Makepad 更宽泛、面向着色器的 `mod.math` 接口，也不授予宿主权限。
- 一个有界求值器，限制源码大小、单个字符串大小、受追踪的 Octoscript 自有驻留堆、VM 操作数栈、活动调用帧、指令数和截止时间。这些 VM 上限并不是操作系统层面的进程内存配额，也不包括不透明的受信 Rust 适配器所做的分配。
- 通过 `.parse_json()`/`.to_json()` 直接进行 `Runtime` JSON 转换，提供有界的 `string.to_bytes()` 桥接，以及冻结的 `mod.std.json`：输入严格且受字节数和深度限制，输出感知循环引用且受字节数和深度限制，出错时返回普通脚本错误，而不是执行无界的 VM 工作。
- 可恢复的 `try ... catch ...` 控制流，可跨越 Octoscript 函数调用；硬性资源中止不可被捕获，也不会隐式回滚副作用。
- 默认拒绝的工具宿主：脚本只能通过 `mod.tool` 调用显式注册的工具。
- 一个有界、面向 LLM 的工具目录，除了每个工具的元数据和 schema 限制外，还对描述符总数和序列化字节总数设限。
- 游标安全的有界能力审计导出和工作流事件视图，带显式淘汰计数器；另有可选启用的认证持久化能力审计日志和认证工作流事件日志，供宿主自有的运维/审计回放使用，并与工作流权限保持分离。
- 按宿主接收顺序、有界的跨流遥测，覆盖具名的能力审计和工作流事件源分段，包括一个内存聚合器和一个认证持久化聚合日志，具备精确的源游标和聚合游标、显式的丢失检测，且不具备恢复或能力权限。
- 带审计的工具调用，限制输入/输出和调用次数。
- 用于结构化工具输入和输出的有界可执行 JSON 契约。
- 为经过审查的 Rust 输入和输出类型提供强制要求 schema 的 Serde 桥接。
- 有界、由宿主驱动的延迟工具 promise，适用于协作式的移动端和嵌入式事件循环。
- 一个封闭的静态目录移动端与嵌入式 profile，面向经过审查的本地 Rust 适配器，并为脚本可见的结构化数据提供可执行 JSON 契约。
- 一个有界的宿主自有固定文件目录适配器，面向经过审查的普通 UTF-8 文件，只通过不透明标识符寻址，并在初始化时固定，而不是由脚本选择文件系统路径。
- 一个特性开关控制的宿主自有固定 HTTP 端点目录和精确源（exact-origin）策略目录，用于经过审查的 JSON GET 和 POST 调用。固定端点只接受不透明 ID；源策略只有在 scheme、host 和实际端口精确匹配后才放行有界的脚本 URL。两者默认使用 HTTPS，限制请求/响应数据，禁用代理和重定向，并由宿主控制方法、请求头和凭据绑定。宿主可以把一个已解析的凭据注入到某一个固定 HTTPS 端点，也可以有意地注入到同一精确 HTTPS 源下的所有已接受路由。一个可选的原生解析器在 macOS、iOS 和 Windows 上执行只读的精确凭据查找，不提供 mock 兜底；这属于 API 层面的中介，而不是出站流量隔离，也不是通用的密钥 API。
- 一个可选、仅限 Linux 的私有 Unix socket HTTP 代理，服务于隔离的 Bubblewrap worker。它把清单中不透明的 `network_origin` ID 精确绑定到一个经过审查的端点或源目录，保留 Bubblewrap 隔离的网络命名空间，并添加一个以描述符固定、只包含一个 socket 的私有目录。它按 worker 会话整体生效，仅支持 HTTP，不是可移植的防火墙，不是原始网络 API，不是按工具划分的进程边界，也不是持久化副作用协议。
- 一个有界的 worker 侧能力密钥代理契约，面向经过审查的 Rust 适配器：宿主自有的 provider 只能把一个会自动清零的二进制密钥释放给某一个精确预配置的 `(tool, secret-id)` 绑定，且该绑定当前生效的 worker 授权必须携带同一个不透明的 `Secret` 资源。它不提供任何 Octoscript 查找或枚举 API，本身也不是平台凭据存储或操作系统级的密钥边界。
- 一个封闭的移动端与嵌入式工作流 profile，暴露纯数据草稿、有界 JSON 数据流和 schema 契约、宿主自有计划、具名的逐步策略、检查点和执行，包括仅在初始化时配置的固定文件和固定端点/源目录适配器以及直接能力模块，但不暴露可变的能力注册。
- 仅延迟执行的外部工具，由宿主认领、完成或取消，无需安装进程内处理器。
- 按工具设置的延迟截止时间，由宿主驱动过期，超时结果可审计。
- 仅供宿主使用的外部工具有界重试，并为每个延迟操作提供稳定、不具授权作用的幂等键。当操作系统熵不可用时，外部注册以关闭方式失败，除非宿主提供一个有界的会话 nonce 并注明其唯一性范围。
- 有界、可选脱敏的外部输出分块，只释放给受信宿主，从不直接交给 Octoscript 源码。
- 带密钥、区分方向、经过重放检查的 worker 协议帧，以及针对进行中外部操作的认证对账。
- 认证的持久化操作分发帧和有界 worker 日志，使幂等性在 worker 重启后仍然重放安全。
- 一个按能力划定范围的 worker 运行时，只分发显式注册的 Rust 适配器，并强制持久化操作的顺序。
- 经宿主批准、按当前策略和产品动作重新校验的持久化补偿意图，每个成功的操作对应一个逆向副作用，并支持重放安全的 worker 恢复。
- 与审批绑定、带目录指纹的能力租约，在 `await` 和恢复执行之间对动态工作流工具调用进行权限收窄，包括为每个受信工作流步骤签发一个最小权限租约。
- 宿主自有、有序的逐步能力策略，在签发这些租约之前把具名的受信步骤绑定到授权；它们是配置，而不是序列化的或脚本可见的权限。
- 有界、纯数据的工作流检查点：重启后要运行计划的剩余部分，必须重新获得宿主批准；数据流检查点只保留上下文摘要，而不保留原始输入或先前的输出。
- 可恢复的进行中外部工作流步骤，在宿主完成操作期间，或在两阶段协作式适配器取消请求与确认期间，保留已批准的能力租约。
- 与计划绑定的持久化外部操作账本，包含输入指纹、派生的 worker 密钥、修订水位钩子，以及一个用于挂起的外部工作流步骤的两阶段 prepare/persist/exact-claim 桥接。
- 仅供宿主使用的认证存储信封，支持密钥轮换，并要求严格防回滚的 compare-and-swap 后端契约。
- 可选的 SQLite 载荷存储，搭配一个显式的受信回滚锚点，包括持久化的修订和 fencing 承诺。
- 有界的事务性回滚锚点服务协议，附带可嵌入的服务端分发器、可选的精确调用方/操作/记录授权关卡，以及可选的固定 HTTPS 客户端传输。它拒绝格式错误或版本倒退的协议数据，并禁用客户端重定向和代理，但单独部署的服务仍然是防回滚的 CAS 权威。
- 带 fencing 的认证 worker 日志存储，把持久化 worker 状态绑定到宿主选定的记录、修订和当前写入者租约。
- 受特性开关控制、经过认证的进程内 worker 传输层，供应用提供的移动端和嵌入式适配器使用；它保留普通调用的分帧方式，但并不是操作系统级隔离。
- 受特性开关控制、有界的 JSON-line worker 通道和经过认证的传输层，用于宿主提供的受隔离 worker 管道；进程创建、截止时间和隔离仍由宿主策略决定。
- 受特性开关控制的多路复用 JSON-line 传输层和 worker 驱动，用于单次经过认证的普通调用，支持与请求精确绑定的协作式取消、显式选择启用的可取消适配器，并且在进程终止时不会给出虚假确认。
- 与会话绑定的 supervisor 桥接层：先解决 watchdog 竞态，再把 worker 的完成或取消应用到 `CapabilityRuntime`；另有一个工作流适配器，通过 `WorkflowEngine` 推进被挂起的步骤。
- 受特性开关控制、一次性、经过认证的持久化操作传输层，用于全新的受隔离 worker 会话；它校验一次派发、对账或补偿的结果，但不会自动执行恢复策略。
- 受特性开关控制的 Bubblewrap 停止后恢复协调器：要求提供与会话绑定的回收证明，以不同密钥启动一个最小权限的受隔离 worker，执行一次受 watchdog 约束的对账，并通过带 fencing、经过认证的 compare-and-swap 存储提交观察结果。
- Linux Bubblewrap worker 策略编译器和启动器，面向固定的、由宿主选定的 worker 和由清单选定的文件根目录，另可选一条精确的、经代理（brokered）的 HTTP `network_origin` 路径；它会拒绝可执行文件和密钥选择器，以及所有没有该代理的 network-origin 授权，而不是声称支持并不支持的策略；它会拒绝持久化的可写宿主根目录，除非该目录带有经过验证、按描述符固定的 Linux 项目配额（配置了总量层面的字节和 inode 硬上限），并强制锁定后续用户命名空间，或者宿主代码显式选择较弱的外部配额逃生舱；在 worker 执行前，它会丢弃所有 Linux capability。
- 可选的 Linux 按描述符固定的可执行文件身份，适用于固定的 Bubblewrap、worker、pre-exec 运行器以及显式的 Landlock 可执行目标，不提供按路径启动的回退。它要求按描述符固定的运行时根目录，但不能替代不可变的运行时所有权，也不能替代完整的代码执行管控。
- 面向 Linux Bubblewrap worker 的一次性、带版本的私有管道会话引导，它与编译后命令所保留的精确清单绑定，先于 JSON worker 帧发送，且不会通过 argv 或环境变量暴露密钥。
- 可选的有界私有 `/tmp` 容量，以及一个宿主生命周期句柄：它可以强制终止并回收 Bubblewrap worker，同时不把终止当作适配器副作用的结果。
- 由清单选定、有界的临时 `file_root` 挂载，挂载在宿主选定的 worker 路径上。处于活动状态的、由宿主支撑的持久化可写根目录默认失败关闭，除非它们在精确的、按描述符固定的根目录上使用经过验证的 Linux 通用项目配额，配置了总量层面的字节和 inode 上限，并强制锁定后续用户命名空间；或者宿主代码显式确认存在独立执行的配额。可选启用的更严格策略会拒绝未经验证的持久化根目录和无上限的私有 `/tmp`，要求锁定后续用户命名空间，并把基础命名空间的文件系统重新挂载为只读。每个临时根目录都有自己的 `tmpfs` 分配上限，宿主可以在启动前拒绝超出所配置总潜在容量的方案。这仍然是按挂载点独立计算的 tmpfs 用量，而不是共享的 tmpfs 运行时配额；它不单独限制 inode，也不是持久化存储、`noexec` 保证或可移植的宿主文件系统配额。一个 worker 方案默认最多包含 64 个互不相同的活动 `file_root` 选择；宿主可以调低这个上限（包括降到零），也可以显式调高，但最多只能到固定的 256 个根目录上限。这个上限约束的是挂载方案的规模，而不是磁盘用量。
- 可选的 Linux cgroup-v2 worker 会话，支持由宿主委派的 CPU 带宽、内存、swap、任务数以及按设备的 I/O 限制；一个固定的运行器会在 Bubblewrap 启动前加入该 cgroup，受管的生命周期拆除会杀掉整个 worker 进程树。
- 可选的 Linux Bubblewrap seccomp 配置：一个面向兼容性的固定拒绝集合，以及一个由宿主选定、有界的严格系统调用允许列表，未列出的系统调用会直接杀死进程。配合 Landlock 可执行文件运行器时，严格过滤会在 Landlock 设置完成之后、紧接着固定内层 exec 之前生效。两者都不管控可执行文件路径或 capability 授权。
- 可选的 Linux Landlock 基于文件系统的可执行文件允许列表，针对 worker 可见的精确文件，由固定的 pre-exec 运行器安装，内核不支持时没有回退。它不是完整的代码加载、网络、密钥或 capability 边界。
- 可选的 Bubblewrap watchdog 和通用的有界 worker 传输层，支持由宿主选定的单次调用和整个会话的挂钟截止时间；超时或宿主终止会使会话失效，其结果保持不确定。
- 一个小型 `octoscript` CLI，用于本地求值和运行工作流示例。

默认情况下，源码无法访问任何环境文件系统、子进程、原始 socket、HTTP 客户端/服务端，或 Makepad 平台/调试模块。继承来的 VM 引导过程在内部保留了兼容性对象，但 `Runtime` 会在规范模式或兼容模式求值之前屏蔽它们在源码中的入口。可选的固定文件目录和固定端点目录是显式、有界的工具，而不是通用的文件系统或网络 API。VM 中的 capability 检查并不是操作系统沙箱；执行本地工具或需要出站网络隔离的适配器，必须运行在合适的、针对目标平台的隔离边界之后，才适合处理不可信的工作负载。

对于普通的数值数据流，`use mod.std.math` 提供一个小型、冻结的、由 Octoscript 自有的标量库。它独立于被屏蔽的 Makepad `mod.math` 着色器模块，无法访问文件、进程、网络、时钟、熵源或 Rust crate。

对于严格的本地 JSON 转换，`use mod.std.json` 只提供 `json.parse(document)` 和 `json.stringify(value)`。它们复用与 `.parse_json()` 和 `.to_json()` 相同的、在字节数/深度/循环引用上有界的边界，不具备宿主、适配器、文件系统、进程、网络、时钟、熵源或 crate 访问能力。

对于本地文本处理，`use mod.std.text` 提供 `trim`、`lower`、`upper`、按 Unicode 标量计数的 `len`、按 Unicode 标量切片的 `slice`、按 Unicode 标量字面匹配的 `index_of` 和 `last_index_of`、字面匹配谓词、字面匹配的 `replace_all`、按字面匹配的 `split`，以及 `join`。`slice` 使用左闭右开的标量区间，要求 `0 <= start <= end <= text.len(value)`。`index_of` 返回字面匹配的第一个标量位置，没有匹配时返回 `-1`；空的查找串返回 `0`。`last_index_of` 返回最后一个标量位置或 `-1`；空的查找串返回 `text.len(value)`。`split` 按字面匹配非空分隔符，保留空字段，最多返回 4,096 段。`join` 接受最多 4,096 个字符串组成的数组，保持其顺序，允许使用空字符串作为分隔符。结果受 Octoscript 所配置的字符串长度上限约束；该模块不暴露正则表达式、宿主状态、文件系统、进程、网络、时钟、熵源或 crate 访问。

对于本地集合处理，`use mod.std.array` 提供 `array.len(value)`、`array.has_index(value, index)`、`array.get(value, index, fallback)`、`array.contains(value, item)`、`array.index_of(value, item)`、`array.slice(value, start, end)`、`array.range(start, end)`、`array.concat(left, right)`、`array.compact(value)`、`array.unique(value)`、`array.reverse(value)`、`array.flatten(value)` 和 `array.push(value, item)`。`has_index` 能区分值为 `nil` 的有效下标和不存在的下标，而 `get` 仅在下标不存在时返回 fallback。两者都不遍历数组。`contains` 和 `index_of` 最多扫描 4,096 项，使用直接相等比较：标量按值比较，数组和记录只按引用匹配；`index_of` 返回第一个匹配位置或 `-1`。`slice` 使用左闭右开区间，下标为非负整数。`range` 为带下标的循环构建一个全新的左闭右开 `[start, end)` 数组；其非负端点必须是不超过 `2^53` 的精确 Octoscript 标量整数，`start` 不能大于 `end`，最多返回 4,096 项。规范 Octoscript 有意不提供区间运算符。`compact` 返回一个去掉 `nil` 项的全新浅拷贝数组，同时保留 `false`、零、空字符串以及原有顺序。`unique` 按首次出现的顺序返回一个全新的浅拷贝数组，移除后续与之直接相等（与 `contains` 和 `index_of` 的相等规则相同）的值。`flatten` 只展开一层：每个外层元素都必须是数组，且在复制之前就会拒绝任何超过 4,096 项的源数组或结果。`push` 会修改传入的数组，返回 `nil`，并拒绝超过 4,096 项的结果。这些变换类辅助函数不使用回调，且都是浅层操作；`len`、`has_index` 和 `get` 是常数时间且不设上限。该模块不暴露宿主状态、文件系统、进程、网络、时钟、熵源或 crate 访问。

对于有界的记录处理，`use mod.std.object` 提供 `object.len(value)`、`object.has(value, key)`、`object.get(value, key, fallback)`、`object.pick(value, keys)`、`object.omit(value, keys)`、`object.from_entries(entries)`、`object.with(value, key, item)`、`object.keys(value)`、`object.entries(value)`、`object.values(value)` 和 `object.merge(left, right)`。接受记录输入的辅助函数只接受普通记录或 JSON 对象数据，从不沿原型链查找，也从不调用回调。`has` 能区分值为 `nil` 的自有文本字段和不存在的字段；`get` 仅在该自有文本字段不存在时返回 fallback。`has`、`get` 和 `pick` 不遍历源字段。`pick` 最多接受 4,096 个字符串键，按键数组的顺序返回一个由已存在的请求字段组成的全新浅拷贝记录；缺失的字段会被忽略。`from_entries` 最多接受 4,096 个精确的 `[string, value]` 键值对，保留每个键首次出现的位置，并采用后出现的重复值。`with` 返回一个全新的浅拷贝记录，其中一个字符串键被更新或追加；已存在的键保持原位置，而当源记录已有 4,096 个字段时，新键会被拒绝。`omit` 最多接受 4,096 个字符串键，按源字段顺序返回一个全新的浅拷贝记录，其中去掉了匹配的自有文本字段；重复和缺失的键会被忽略。它是本地的黑名单式变换，而不是边界白名单：当必须排除未知字段时，请在工具边界之前使用 `pick`。`omit`、`keys`、`entries`、`values` 和 `merge` 最多浅层处理 4,096 个以文本为键的自有字段；`entries` 按存储的字段顺序返回全新的 `[text_key, value]` 键值对，`merge` 还会拒绝合并后源字段总数超过该上限的情况。`len` 是常数时间且不设上限。该模块不暴露宿主状态、文件系统、进程、网络、时钟、熵源或 crate 访问。

## 示例

```octoscript
use mod.tool

let summary = tool.call("text.echo", "plan the release")
summary
```

`text.echo` 是否存在、能访问什么，由宿主决定，而不是由脚本决定。

对于需要让出控制权、回到宿主事件循环的工作，使用显式的 promise。宿主每次调用 `pump()` 时最多运行一个已授权的工具（或用 `pump_up_to` 运行有上限的一批）。

```octoscript
use mod.tool

let summary = tool.start("text.echo", "plan the release").await()
summary
```

需要可安全恢复的回退时，使用规范的 `try/catch`。恢复不会退还这次调用，也不意味着适配器已产生的效果被回滚。

```octoscript
use mod.tool

let summary = try {
    tool.start("text.echo", "plan the release").await()
} catch {
    "summary unavailable"
}
summary
```

Rust 应用通过为每种效果注册一个范围收窄、受策略约束的适配器，来接入其现有的 crate 生态。Octoscript 不会直接导入 crate，也不会直接使用环境中的操作系统 API。

JSON 能力使用对象或数组信封。Rust 适配器可以接收并返回 `serde_json::Value`，也可以对经过审查的结构体使用必须附带 schema 的类型化 Serde 桥接；Octoscript 通过 `tool.call_json` 或 `tool.start_json` 把记录和数组转换为 JSON。

```octoscript
use mod.tool
use mod.std.assert

let response_json = tool.call_json("math.add", {left: 20, right: 22})
let response = response_json.parse_json()
assert(response.total == 42)
```

对于固定且经过审查的适配器，Rust 宿主也可以改为注册一个有界的直接能力模块，这样生成的源码就能直接使用解码后的数据：

```octoscript
use mod.arithmetic
use mod.std.assert

let math = arithmetic
let response = math.add({left: 20, right: 22})
assert(response.total == 42)
```

这是由宿主配置、建立在同一个受契约约束的能力之上的语法，而不是通用的模块加载，也不是直接访问 crate。它保留了目标工具的策略、审计和能力租约检查。参见 [Host Tool Catalog](docs/tool-catalog.md)。

开发用 CLI 会把这个经过审查的门面与其 `math.add` 演示能力一起注册：

```sh
cargo run -p octoscript-cli -- run --allow-json-add examples/direct_module_workflow.octoscript
cargo run -p octoscript-cli -- module-catalog --allow-json-add
cargo run -p octoscript-cli -- tool-calls --allow-json-add examples/direct_module_workflow.octoscript
cargo run -p octoscript-cli -- workflow-review --allow-json-add examples/direct_module_workflow_draft.json
cargo run -p octoscript-cli -- workflow-run --allow-json-add --grant calculate:math.add:1 examples/direct_module_workflow_draft.json
```

目录把 `arithmetic.add` 门面映射到 `math.add`；工作流策略和租约授予的始终是底层的 `math.add` 能力，而不是门面名称。审查可以通过有界的精确本地根别名（例如 `let math = arithmetic`）保留这一映射；它从不把别名当作新的模块、目标工具或授权。宿主还可以在经过审查的 JSON 工具之上注册一个 `with_deferred_method` 门面；其显式的 `mode: "deferred"` 返回现有的有界 promise，`await()` 产出解码后的 JSON，同时保留同一个底层授权。配置了经过审查的模块目录后，`tool-calls` 和 `workflow-review` 会添加参考性的 `direct_module_calls` 条目，把这一映射展示给 LLM 和运维人员审查。这些条目不会授予任何工具，也不会取代上面显式的 `calculate:math.add:1` 工作流策略。

```sh
cargo run -p octoscript-cli -- run --allow-echo examples/tool_workflow.octoscript
```

延迟执行示例可以这样运行：

```sh
cargo run -p octoscript-cli -- run --allow-echo examples/deferred_tool_workflow.octoscript
```

JSON 数据流示例可以这样运行：

```sh
cargo run -p octoscript-cli -- run --allow-json-add examples/json_tool_workflow.octoscript
```

## Makepad UI 兼容性

[`examples/makepad_ui_counter.octoscript`](examples/makepad_ui_counter.octoscript) 是一段当前风格的小型 Makepad UI 主体，作为解析器兼容性测试样例保留。它有意不能通过 `octoscript-cli` 运行：独立运行时不会安装 Makepad 控件模块、事件循环或 `ui` 句柄。规范的工作流配置仍会拒绝它；受信任的 UI 宿主可以在安装自己的绑定之前使用 `octoscript_core::check_vm_compatibility_named`，它会强制执行源码、token 和分隔符嵌套的上限。关于当前上游示例的区别和确切边界，参见 [Makepad UI compatibility](docs/makepad-ui-compatibility.md)。

查看提供给 LLM 宿主的确切演示工具目录：

```sh
cargo run -p octoscript-cli -- catalog --allow-echo --allow-json-add
```

在生成源码之前，LLM 宿主可以查询带版本的规范语言契约，无需创建运行时或注册工具：

```sh
cargo run -p octoscript-cli -- profile
```

JSON 响应会标明配置和语法文件路径，报告当前生效的默认上限，并说明工具与工作流的权限边界。它不是工具目录，不是能力授权，也不能替代规范性的 [Octoscript Grammar v0.2](docs/grammar.md)。提出有副作用的调用之前，请查询宿主单独提供的目录。

对于 LLM 生成的有序工作流，在编写其 JSON 信封之前，先查询有界的草稿生成 schema：

```sh
cargo run -p octoscript-cli -- workflow-schema
```

该 schema 只描述 `format_version` 和有序的 `id`/`source` 步骤，以及解码器的限制。它有意不包含能力、审批、契约、检查点、结果或外部操作句柄等字段；在宿主做任何规划或审批之前，请先用 `octoscript workflow-review` 审查生成的文件。

对照规范的 Octoscript v0.2 配置验证生成的源码，无需创建能力宿主，也不运行任何字节码：

```sh
cargo run -p octoscript-cli -- check examples/deferred_tool_workflow.octoscript
```

该命令输出 JSON 诊断信息，遇到无效源码（包括可移植契约之外的 Makepad 兼容语法）时以非零状态退出。可移植源码契约见 [Octoscript Grammar v0.2](docs/grammar.md)。

在不求值源码、不构造能力宿主的情况下，查看有效的顶层声明：

```sh
cargo run -p octoscript-cli -- outline examples/json_tool_workflow.octoscript
```

该命令输出包含 `function` 和 `let` 声明的 JSON，并附带每个声明及标识符的 UTF-8 字节范围。对于无效源码，它仍会输出结构化的语法诊断，并以非零状态退出，声明列表为空。

在申请审批之前，查看源码中直接出现的 `tool.call`、`tool.start`、`tool.call_json` 和 `tool.start_json` 调用位置：

```sh
cargo run -p octoscript-cli -- tool-calls examples/json_tool_workflow.octoscript
```

该命令输出 JSON 位置信息；当第一个参数直接写成字符串时，还会给出字面量工具名。它从不求值源码，也不创建能力宿主。它只是审查辅助：别名、遮蔽、控制流和计算得出的名称都不会被解析，因此宿主仍须签发租约，运行时仍须对每一次实际调用进行授权。输出最多保留 1,024 个直接调用位置，若省略了后面的位置，会设置 `tool_calls_truncated`。宿主还可以额外公开其经过审查的直接模块映射；开发演示仅在带有 `--allow-json-add` 时这样做，并输出单独的参考性 `direct_module_calls` 列表。该映射可以追踪可见直接导入的有界精确本地根别名，但从不追踪计算得出的接收者、成员别名或源自源码的权限。

在 LLM 生成的多步草稿成为宿主持有的计划之前，先进行审查：

```sh
cargo run -p octoscript-cli -- workflow-review examples/release_workflow_draft.json
```

带版本的 JSON 草稿只包含步骤 ID 和源码。审查输出包括每个步骤的语法状态和直接工具调用提示，从不包含授权或审批。某个步骤的直接调用审查被截断时，该步骤会报告 `tool_calls_truncated`；整个工作流在所有步骤中最多保留 4,096 条提示。其上限和宿主生命周期参见 [workflow drafts](docs/workflow-drafts.md)。在显式配置了宿主模块目录时，另有一个参考性的 `direct_module_calls` 列表，可以把直接门面调用（包括有界的精确本地根别名）映射到其底层工具；它同样受整个工作流 4,096 条的上限约束，并且从不选择授权。

只在显式由宿主选择的逐步授权下，运行有界的本地演示目录：

```sh
cargo run -p octoscript-cli -- workflow-run --allow-echo --allow-json-add \
  --grant prepare:text.echo:1 --grant calculate:math.add:1 \
  examples/local_workflow_draft.json
```

使用显式输入和一个经过审查的授权，运行有界的数据流示例：

```sh
cargo run -p octoscript-cli -- workflow-run --allow-json-add \
  --input examples/dataflow_input.json \
  --grant prepare:math.add:1 \
  examples/dataflow_workflow_draft.json
```

`prepare` 步骤用 `object.pick` 把宿主输入显式收窄为经过审查的 `math.add` 信封。随后，纯计算的 `summarize` 步骤使用了：带回退的动态自有字段查找、带空输入回退的有界数组下标查找、有界的数组变换和循环、文本规范化、自有字段记录合并，以及有界的 JSON 往返转换。它不获得任何工具授权；只有 `prepare` 能发起经过审查的效果。

`workflow-run` 只接受这两个需显式启用的本地演示适配器，并打印结构化的执行/审计摘要。它从不根据源码提示推导授权，不开放文件系统、网络或进程权限，也不支持外部 worker。生产宿主必须构造自己经过审查的目录和策略。使用 `--input` 时，直接结果还会包含原始的数据流输入和输出，便于本地检查；审计视图和工作流事件视图则从不包含这些内容。

对于生产环境的数据流，宿主还可以通过 `WorkflowDataContract` 绑定编译好的输入 schema 和逐步输出 schema。这些 schema 是受信任的应用配置，而不是草稿或检查点中的字段；输出契约校验失败时，工作流会在后续已授权步骤运行之前停止。使用配套的、感知契约的检查点/ 恢复 API，可在重启后保留该策略。参见 [workflow drafts](docs/workflow-drafts.md) 和 [workflow checkpoints](docs/workflow-checkpoints.md)。

格式化有效的规范源码，无需创建能力宿主，也不会改写输入文件：

```sh
cargo run -p octoscript-cli -- format examples/deferred_tool_workflow.octoscript
```

在编辑器或 CI 工作流中使用 `--check`，要求源码符合规范格式化结果，但不打印输出：

```sh
cargo run -p octoscript-cli -- format --check examples/deferred_tool_workflow.octoscript
```

在兼容 LSP 的编辑器中运行语言服务器：

```sh
cargo run -p octoscript-lsp
```

它接受客户端提供的已打开文档文本，以及可选的、有上限的参考性初始化元数据和配置刷新；最多保留 128 个文档状态，且不保留超过标准 256 KiB 源码上限的文档文本。它提供全量同步诊断、整篇文档格式化、顶层声明符号，以及有界的同文档词法定义/引用请求、绑定类型悬停提示、符号高亮、词法补全和带防护的重命名。普通词法补全只在光标位于表达式位置标识符之内或末尾时提供。它返回在该 token 处可见的全部已保留绑定，交由客户端过滤，并为该标识符提供精确的替换编辑。对于无效源码，只有在第一个语法诊断之前结束的位置才能获得补全。候选出现位置和补全位置各有独立的 4,096 条上限；任一被截断都会把 LSP 结果标记为不完整。截断的位置列表仍可服务已保留的位置，但截断的符号集不会返回任何候选，因为被省略的内层定义可能遮蔽已保留的外层绑定。只有当客户端支持带版本的 `documentChanges` 时才会声明支持重命名；每次编辑都绑定到确切的已打开文档版本。它会拒绝截断的索引、导入路径的变更、无效标识符，以及会改变完整索引词法绑定报告的改写。它从不读取文档 URI、不求值源码、不加载或解析任意导入模块、不创建能力宿主，也不加载 Rust 适配器。对于精确且词法可见的 `use mod.tool` 绑定，它还会在直接的 `tool.` 成员位置提示固定的 `call`、`call_json`、`start` 和 `start_json` 方法。对于精确可见的 `use mod.std.math` 绑定，它还会在直接的 `math.` 成员位置补全文档中列出的固定标量函数以及 `pi`/`e` 常量，并提供纯文本悬停提示和函数签名帮助。对于精确可见的 `use mod.std.assert` 绑定，它还为 `assert(condition)` 提供固定的纯文本悬停提示和签名帮助；`use mod.std` 在直接的 `std.assert(...)` 处支持同样的固定签名。这些固定能力面不查询工具目录或适配器，不追踪本地别名，也不意味着任何能力授权。对于精确可见的 `use mod.std.json` 绑定，它会补全 `parse` 和 `stringify`，并提供固定的纯文本悬停提示和签名帮助。对于精确可见的 `use mod.std.text` 绑定，它会补全固定的文本函数，包括按 Unicode 标量计算的 `slice`、`index_of` 和 `last_index_of`、按字面量分割的 `split`，以及拼接字符串数组的 `join`，并提供纯文本悬停提示和签名帮助。对于精确可见的 `use mod.std.array` 绑定，它会补全 `len`、`has_index`、`get`、`contains`、`index_of`、`slice`、`range`、`concat`、`compact`、`reverse`、`unique`、`flatten` 和 `push`，并提供同样固定的纯文本悬停提示和签名帮助。对于精确可见的 `use mod.std.object` 绑定，它会补全 `len`、`has`、`get`、`pick`、`omit`、`from_entries`、`with`、`keys`、`entries`、`values` 和 `merge`，并提供同样固定的纯文本悬停提示和签名帮助。在语句位置的 `use mod.` 路径处，同一静态投影会补全 `std`；在 `use mod.std.` 之下，它会补全 `array`、`assert`、`json`、`math`、`object` 和 `text`。冻结的 `mod.std` 子树不能被参考性目录元数据扩展。集成方还可以通过 `initializationOptions.octoscript.toolCatalog` 或之后的 `workspace/didChangeConfiguration` 更新，额外提供一个参考性的工具目录投影；它接受宿主目录 JSON 中的 `name`、`format` 和 `description` 字段。对于精确可见的 `mod.tool` 绑定，LSP 会在直接的 `call`/`start` 调用中用文本条目、在直接的 `call_json`/`start_json` 调用中用 JSON 条目补全第一个字符串字面量。它从不连接能力运行时，不读取目录文件，也不从这些元数据推导授权。该投影的上限为 128 条、名称和描述合计保留 512 KiB、名称 128 字节、描述 4 KiB；格式错误、重复或超出上限的输入会被整体丢弃，并把该次补全结果标记为不完整。词法服务还能识别精确可见的直接 `let binding = { ... }` 初始化器。在 `binding.field`、直接的两级字面量路径（如 `binding.child.grandchild.field`），或者通过精确的 `let alias = binding`、`let alias = binding.child`、`let alias = binding.child.grandchild` 链（最多 16 跳，总共最多两次别名子项选择，无论是在一条边上还是分散在整条链中）访问时，它会提供字面量字段名，并支持悬停提示和跳转到字段键的定义。别名目标在其源码位置解析，因此词法遮蔽保持不变。这些元数据的上限为 1,024 个结构、4,096 个字段和 1,024 个直接别名。省略任何一条别名边都会使已保留的记录补全为空且不完整，并禁用静态字段悬停提示和定义跳转。如果之前通过根绑定、或通过解析到它的任何已保留根/子项/孙项别名，发生过直接写入，或存在可能产生修改的成员、下标、调用或逃逸路径，LSP 会屏蔽该结构。它不推断带括号或计算得出的别名、带括号或计算得出的子项值、超出两级预算的别名或成员路径、赋值、控制流、函数返回值、导入的值或运行时数据。除此之外它保持保守：不推断前向引用、一般类型、任意记录字段、内置项、任意目录数据或运行时才能得出的导入模块导出项。

编辑器还可以通过 `initializationOptions.octoscript.moduleCatalog` 或之后的 `workspace/didChangeConfiguration` 更新，提供一个独立的参考性模块接口投影。它会在语句位置的直接 `use mod.*` 路径中补全当前段，在直接可见的导入模块绑定或稳定的精确本地根别名链之后补全有界的目录路径，并为精确的目录叶子提供纯文本参考性悬停提示。显式同时声明了模式和 `single_json` 调用形态的精确叶子，还会获得一个有界的单值签名；服务器从不仅凭模式推断签名。它不加载源文件、不解析模块、不检查运行时导出，也不覆盖固定的 `mod.tool` API。工具目录键和模块目录键各自独立刷新：省略的键保留之前的值，JSON `null` 显式清除它，格式错误或超出上限的键值只会使对应的那个目录不可用。格式错误的 `settings` 值或非对象的 `settings.octoscript` 会清除所有参考性目录。两种投影都不会为源码授权。模块别名必须是精确的 `let alias = binding` 链，最多 16 跳，具有完整的源码元数据，并且在其解析到的组内没有写入、成员提取、带括号/计算得出的边或其他值逃逸；否则目录元数据会以失败关闭的方式处理。这并不扩展固定的 `mod.tool` API，其编辑器支持仍仅限直接导入。其确切格式和上限参见 [editor module-interface projection](docs/module-catalog.md)。截断的词法索引仍可提供已保留且可靠的定义和悬停提示，但需要穷尽结果的引用、高亮和重命名请求会直接失败，而不是返回不完整的集合。

对于宿主管理的数据流编写会话，编辑器还可以提供一个独立的 `initializationOptions.octoscript.workflowDataCatalog` 投影。它补全直接且未被遮蔽的 `workflow.input.*` 和 `workflow.outputs.<stepId>.*` 路径，并为已知字段元数据提供悬停提示。使用 `octoscript-workflow` 的宿主可以从挂起的、绑定契约的延续或检查点生成经过验证的当前前缀更新；LSP 本身仍不加载 schema 或运行时状态。它不校验数据、不批准工作流、不签发租约，也不会让适配器变得可调用；缺少元数据时不会创建 `workflow` 命名空间，格式错误的输入会以失败关闭的方式处理。宿主可以提供 `workflowDataStepContext`，在结构上绑定一个被投影的当前步骤及其之前被投影的输出前缀，用于过滤输出补全和悬停提示。之后宿主可以通过 `workspace/didChangeConfiguration` 替换完整的目录/上下文对；相关的格式错误或不完整刷新会使工作流元数据不可用，而不是保留过时的投影。终止或不可用的运行时状态可以用 JSON `null` 原子地清除这两个键。参见 [editor workflow-data projection](docs/workflow-data-catalog.md)。

## 工作区

- `octoscript-core`：有界的 VM 封装与诊断。
- `octoscript-capabilities`：显式工具策略、游标安全的有界审计导出（带有按 feature 启用的已认证持久日志）、延迟 promise、面向 LLM 的宿主目录、与审批绑定的能力租约、JSON 契约、固定文件目录与按 feature 启用的 HTTP 端点/来源目录、目录总量上限、安全的宿主桥接，以及封闭的静态目录移动端/嵌入式配置。
- `octoscript-schema`：用于工具契约的有界、可执行 JSON Schema 子集。
- `octoscript-storage`：仅限宿主的已认证记录、回滚保护，以及带隔离令牌（fenced）的 compare-and-swap 后端边界；另有一个可选的锚定 SQLite 负载适配器，它需要平台信任锚和一个有界的事务服务锚客户端；二者都不能替代实际部署的信任权威。
- `octoscript-protocol`：可移植的 worker 消息、能力衰减、固定的 128 项授权清单与 1,024 个保留请求标识的会话上限、带密钥的会话分帧、与实例绑定的进程内授权令牌、严格的普通调用取消，以及宿主侧的调用/结果校验。
- `octoscript-worker`：worker 侧会话运行时、显式的 Rust 适配器注册表、可取消的普通调用驱动、与能力绑定的密钥代理契约，以及已认证的日志存储桥接；它不是操作系统沙箱，也不是平台存储后端。
- `octoscript-sandbox`：针对特定目标平台的 worker 隔离策略；其首个 Bubblewrap 后端仅支持 Linux，且刻意保持狭窄，为临时数据提供由清单选定、有界的临时文件根目录。
- `octoscript-workflow`：宿主拥有的规划、与租约绑定的审批、有界 JSON 数据流、有界的内存及已认证持久事件与跨流遥测日志、按宿主接收顺序的聚合、检查点、持久操作记录、可选的带隔离令牌的 Bubblewrap 停止后对账、多路复用 worker 完成接收端、顺序执行，以及面向静态本地适配器和直接能力模块的封闭移动端/嵌入式工作流门面。
- `octoscript-cli`：本地开发 CLI。
- `octoscript-lsp`：仅限宿主的 stdio 诊断、规范格式化、顶层声明符号，以及针对已打开编辑器文档的有界同文档词法导航、悬停提示和高亮，外加词法补全和与版本绑定的受保护重命名。
- `makepad-script`（git 依赖，`OctoSense-org/makepad` 的 `octoscript` 分支）：继承而来的 VM 与解析器；参见 `UPSTREAM.md`。

当前的威胁模型见 [SECURITY.md](SECURITY.md)，依赖边界见 [UPSTREAM.md](UPSTREAM.md)。[worker 协议](docs/worker-protocol.md)定义了向隔离适配器的交接。[宿主工具目录](docs/tool-catalog.md) 定义了面向 LLM 编排器的安全发现机制。[JSON 工具契约](docs/schema-contracts.md) 定义了可执行的结构化数据边界。[外部工具](docs/external-tools.md) 定义了由宿主管理的异步边界。

[固定文件目录](docs/fixed-file-catalog.md)定义了狭窄的、由描述符固定的本地文本文件边界。

[HTTP 端点与来源目录](docs/http-endpoint-catalog.md)定义了由宿主选定的狭窄出站 JSON 边界、与端点和来源绑定的凭据注入，以及它们明确不作的保证。

[编辑器模块接口投影](docs/module-catalog.md)定义了面向宿主定义的 `mod.*` 接口的有界、可刷新的编写元数据。

[worker 协议 v5](docs/worker-protocol.md)还定义了带密钥的 worker 帧以及实时操作的对账边界。

[工作流检查点](docs/workflow-checkpoints.md)定义了持久的宿主编排边界。

[持久工作流事件](docs/workflow-events.md)定义了已认证的遥测重放边界，它刻意与恢复权限保持分离。

[能力审计导出](docs/capability-audits.md)定义了连续的宿主导出游标、可选的已认证持久日志，以及明确的可观测性缺口行为。

[跨流遥测](docs/cross-stream-telemetry.md)定义了对来源遥测进行有界的内存及已认证持久的按宿主接收顺序聚合，且不产生恢复或能力权限。

[工作流草稿](docs/workflow-drafts.md)定义了在宿主拥有的审批之前，不受信任的 LLM 计划的交换与审查边界。

[定位与可行性](docs/positioning.md)将 Octoscript 与其 Makepad 底座进行比较，并界定了替代 Python/JavaScript 这一说法的现实边界。

[持久操作账本](docs/workflow-operations.md)定义了宿主如何记录不确定的外部副作用，并在重启后安全地对账。

[已认证存储](docs/durable-storage.md)定义了用于持久化这些宿主拥有记录的可信持久记录边界。

[事务性回滚锚服务](docs/rollback-anchor-service.md) 为一个单独受信的持久 CAS 权威定义了有界的客户端协议和可嵌入的服务端分发器。

[worker 持久操作](docs/worker-operations.md)定义了隔离 worker 侧针对有副作用操作键的重放与持久化边界。

[持久 worker 补偿](docs/worker-compensation.md)定义了针对单个显式逆向副作用的宿主审批、worker 日志与崩溃恢复规则。

[worker 适配器运行时](docs/worker-runtime.md)定义了 worker 侧的 Rust 适配器边界，以及隔离后端的集成要求。

[Linux Bubblewrap worker](docs/linux-bubblewrap.md)定义了首个隔离 worker 启动器、它的能力映射，以及明确不作的保证。

[Bubblewrap 停止后恢复](docs/bubblewrap-recovery.md)定义了回收、新会话对账以及带隔离令牌的宿主账本提交顺序。
