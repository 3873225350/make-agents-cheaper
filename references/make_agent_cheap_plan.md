# Make Agent Cheap — Token & Cost Optimization Strategy

## 1. 核心目标
通过优化 Agent 的工具调用、上下文管理、输出格式等环节，大幅降低 Token 消耗和计算成本，实现更长时间、低成本会话执行。

**核心原则：**
- **去重**：避免重复读取同一文件或信息。
- **压缩**：减少冗余输出和 diff 信息。
- **智能上下文管理**：防止 Context 膨胀浪费 Token。
- **轻量输出**：降低输出 Token，减少不必要的内容生成。

---

## 2. 技术方案

### 2.1 读取层（Read Layer）
| 方案 | 原理 | 预期节省 |
|------|------|-----------|
| **Read 去重 Hook** | 在 PreToolUse 阶段对读取操作做缓存，重复文件直接跳过 | ~40% |
| read-once diff mode | 文件改动后只返回 diff 而非全文 | 80-95% |
| .claudeignore | 忽略 node_modules、dist 等大文件 | 避免一次性大文件读取 |
| 精确指向 Prompt | 针对文件特定行而非全文件 | 减少探索性读取 |

**实现方式：**
```bash
# pre-tool-dedup.sh
if file_path in /tmp/.claude_read_cache_${SESSION_ID}; then
  exit 2  # skip
else
  append to cache
  exit 0
fi
```

---

### 2.2 工具/命令层（Tool Layer）
| 方案 | 原理 |
|------|------|
| git diff 压缩 Hook | 使用 compress-diff.sh 过滤冗余上下文，仅保留变更行 |
| RTK / squeez | 自动包裹 Shell 输出，如 git status、npm test，超长输出截断或摘要 |
| 跨调用去重 | 利用哈希检测重复输出，返回占位符 |
| Hook 截断长输出 | PostToolUse 或 PreToolUse 截断长日志输出 |

---

### 2.3 上下文管理层（Context Layer）
| 方案 | 原理 |
|------|------|
| StatusLine 桥接 | 使用 statusline.sh + user-prompt-submit.py 监控 Context 占用率，超过阈值自动 /compact |
| 主动 /compact | 在 Context 50-60% 时提前压缩历史对话，避免无效 Token 消耗 |
| Session 结构化 | 一个 Session 仅处理一个逻辑任务，无关任务分开 /clear |
| Subagent 隔离 | 调研类任务交给 Subagent，主 Session 只接收摘要 |

---

### 2.4 输出层（Output Layer）
| 方案 | 原理 |
|------|------|
| Caveman /compact mode | 强制极简格式输出，减少 Token |
| 模型降级策略 | 日常使用低成本模型（如 Sonnet/Haiku），复杂决策时才用高成本模型 |
| Plan Mode | 输出执行计划，人工审查后再执行，避免试错循环浪费 |

---

### 2.5 架构/自动化层（Architecture Layer）
| 方案 | 原理 |
|------|------|
| Hooks 组合栈 | CBM + context-mode + RTK + Headroom + Caveman，实现高达 90% 综合压缩 |
| Qdrant 记忆 + HyDE | 向量库存储项目记忆，避免重复探索 |
| Progressive Disclosure | 默认只显示参考材料第一行，按需展开 |

**调用链路示意：**
```text
用户输入
   ↓
Shell Wrapper (claude.bash/fish/zsh)
   ↓
Headroom 包装器（注入压缩逻辑）
   ↓
Claude Code 主程序
   ↓
Hooks 层：
    ├─ PreToolUse (Read去重 + Diff压缩)
    ├─ PostToolUse (清理 + 压缩)
    └─ RTK 实时压缩输出
   ↓
Caveman 模式强制极简输出
```

---

## 3. 优先级与落地策略
1. **Read 去重** — 最高收益、最简单、零副作用。
2. **Diff 压缩** — 高频重构场景收益显著。
3. **StatusLine 桥接** — 可调阈值 55-70%。
4. **叠加 .claudeignore + 精简 CLAUDE.md** — 即刻生效。
5. **进阶** — 引入 RTK / squeez + read-once diff mode 覆盖更多场景。

---

## 4. 一键部署参考（Linux/MacOS）
```bash
# 1. 创建目录
mkdir -p ~/.claude/hooks ~/.claude/rules

# 2. 复制 Hooks 并赋权
cp hooks/* ~/.claude/hooks/ && chmod +x ~/.claude/hooks/*

# 3. 复制规则文档
cp rules/*.md ~/.claude/rules/

# 4. 覆盖 settings.json
cp settings/settings.json ~/.claude/settings.json

# 5. CLAUDE.md 示例
cp CLAUDE.md.example ~/.claude/CLAUDE.md

# 6. 安装 StatusLine 命令
cp statusline/statusline-command.sh ~/.claude/ && chmod +x ~/.claude/statusline-command.sh

# 7. Shell 集成
cat shell/claude.bash >> ~/.bashrc
# 或 cat shell/claude.fish >> ~/.config/fish/config.fish
# 或 cat shell/claude.zsh >> ~/.zshrc
```

---

## 5. 总结
通过这一套全栈优化方案：
- **减少重复读取** → 节省 40%+ Token。
- **压缩 diff 和长日志** → 节省 70%+ Token。
- **上下文管理与极简输出** → 防止 Context 膨胀，延长会话寿命。
- **组合 Hooks 与自动化** → 综合节省可达 90%+ Token，实现真正低成本、高效率的 Agent 运行。
