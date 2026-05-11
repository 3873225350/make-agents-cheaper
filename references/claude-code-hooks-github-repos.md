# Claude Code Token 优化 & Hooks 相关 GitHub 仓库汇总

> 调研时间：2026-05-11
> 主题：Claude Code Hooks、Token 节省、权限控制、安全拦截

---

## 一、Token 优化 & Hooks 核心工具

| 仓库 | 说明 | 核心能力 |
|------|------|----------|
| [**sgaabdu4/claude-code-tips**](https://github.com/sgaabdu4/claude-code-tips) | 全栈 Token 优化方案 | CBM + context-mode + RTK + Headroom + Caveman + enforcement hooks，号称省 90%+ Token |
| [**rtk-ai/rtk**](https://github.com/rtk-ai/rtk) | Rust 编写的 CLI 代理 | 自动重写 Bash 命令输出，省 60-90% Token；2 周实战省 10M tokens（89%） |
| [**JuliusBrussee/caveman**](https://github.com/JuliusBrussee/caveman) | Caveman 模式 + 压缩技能 | 强制极简回复，省 65% 输出 Token；支持 `/caveman` 分级压缩、文言文模式、文件压缩 |
| [**Bande-a-Bonnot/Boucle-framework**](https://github.com/Bande-a-Bonnot/Boucle-framework) | read-once Hook 源码 | PreToolUse 拦截 Read 工具，阻止重复文件读取，省 ~40% Read Token；支持 diff mode |
| [**iChenwin/claude-code-translator**](https://github.com/iChenwin/claude-code-translator) | 翻译 Hook | 中英 Prompt 互转，省 30% Token 消耗 |

---

## 二、权限控制 & 安全 Hooks

| 仓库 | 说明 | 核心能力 |
|------|------|----------|
| [**M-Gregoire/claude-hook-guard**](https://github.com/M-Gregoire/claude-hook-guard) | Go 编写的语义权限系统 | 基于 action_type（read/write）和 tool_family 做规则匹配，支持 allow/deny/ask 三种决策 |
| [**shaxxx/claude-permission-hook**](https://github.com/shaxxx/claude-permission-hook) | .NET 权限 Hook | 规则引擎匹配工具名和参数，支持 `--explain` 解释决策过程 |
| [**coo-quack/sensitive-canary**](https://github.com/coo-quack/sensitive-canary) | 安全防泄密 Hook | 检测 secrets / PII，在发送到 Anthropic API 前拦截；31 条检测规则，支持熵过滤和 Luhn 校验 |
| [**anthropics/claude-code-permissions-hook**](https://github.com/anthropics/claude-code-permissions-hook) | Anthropic 官方示例 | Rust 编写的官方权限 Hook 参考实现 |

---

## 三、配置集合 & 其他 Hooks

| 仓库 | 说明 | 核心能力 |
|------|------|----------|
| [**sumulige/ecc-conveyor**](https://github.com/sumulige/ecc-conveyor) | 完整 Claude Code 配置集合 | agents、skills、hooks、commands、rules、MCPs；Anthropic 黑客松获胜者作品 |
| [**codelably/harmony-claude-code**](https://github.com/codelably/harmony-claude-code) | 中文 Claude Code 配置集合 | ecc-conveyor 的中文 fork/适配版，含完整 hooks、rules、skills 示例 |
| [**Ixe1/claude-code-checkpointing-hook**](https://github.com/Ixe1/claude-code-checkpointing-hook) | Git 快照 Hook | 在 Claude 修改文件前自动创建 git checkpoint，方便回滚 |
| [**igoryuzo/uniswapV4-hooks-skill**](https://github.com/igoryuzo/uniswapV4-hooks-skill) | V4 Hooks 安全 Skill | 针对 Uniswap V4 hook 开发的安全检查 Skill |

---

## 四、快速安装命令汇总

```bash
# 1. 全栈优化方案（90%+）
git clone https://github.com/sgaabdu4/claude-code-tips.git

# 2. RTK Shell 输出压缩（60-90%）
cargo install --git https://github.com/rtk-ai/rtk
rtk init -g

# 3. Caveman 输出压缩（65%）
bash <(curl -s https://raw.githubusercontent.com/JuliusBrussee/caveman/main/hooks/install.sh)

# 4. Read 去重（40%）
curl -fsSL https://raw.githubusercontent.com/Bande-a-Bonnot/Boucle-framework/main/tools/read-once/install.sh | bash

# 5. 权限控制
go install github.com/M-Gregoire/claude-hook-guard/cmd/claude-hook-guard@latest

# 6. 安全防泄密
# 参考 https://github.com/coo-quack/sensitive-canary 文档安装
```

> 提示：**1、2、3、4** 可以叠加使用，覆盖 Read 层、Bash 层、输出层，实现最大程度的 Token 节省。

---

## 五、Hook 类型速查

| Hook 类型 | 触发时机 | 典型用途 |
|-----------|----------|----------|
| `PreToolUse` | AI 调用工具前 | 拦截/改写/去重 Read、Bash 等工具调用 |
| `PostToolUse` | AI 调用工具后 | 压缩/过滤/标记工具输出 |
| `UserPromptSubmit` | 用户发送消息前 | 注入 `/compact`、翻译 Prompt、自动补全 |
| `StatusLine` | 状态更新时 | 读取 Context 百分比，写入临时文件桥接 |
| `SessionStart` | 会话开始时 | 加载项目图谱、激活模式、打印提醒 |

---

*整理 by AI Agent | 数据来源：公开 GitHub 仓库及社区文档*
