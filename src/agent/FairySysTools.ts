import type { Tool } from '../types/tool'

export const sysTools: Tool[] = [
  {
    id: 'sys-execute_shell',
    name: '执行 Shell',
    description: '执行 Shell 命令',
    logo: '',
    logoType: undefined,
    invokeName: 'execute_shell',
    filePath: '',
    parameters: [
      { name: 'command', type: 'string', description: '要执行的命令', required: true },
      { name: 'timeout', type: 'number', description: '超时时间(秒)', required: false, default: 60 }
    ],
    executor: 'shell_execute',
    code: undefined,
    language: 'typescript',
    createdAt: '2024-01-01T00:00:00.000Z',
    updatedAt: '2024-01-01T00:00:00.000Z'
  },
  {
    id: 'sys-file_manager',
    name: '文件管理',
    description: `文件操作工具。通过 action 参数指定操作类型。
- read: 读取文件内容，支持 offset+limit 读取部分内容
- write: 写入/创建文件（完全覆盖）。修改已有文件应优先用 edit
- edit: 精确替换文件中的内容（SearchReplace 模式）
- delete: 删除文件或目录
- glob: 按文件名模式搜索文件（支持 **/*.ts 等通配符）
- grep: 在文件中搜索内容（正则表达式）`,
    logo: '',
    logoType: undefined,
    invokeName: 'file_manager',
    filePath: '',
    parameters: [
      { name: 'action', type: 'string', description: '操作类型: read|write|edit|delete|glob|grep', required: true },
      { name: 'path', type: 'string', description: '文件路径（read/write/edit/delete/grep 需要）', required: false },
      { name: 'pattern', type: 'string', description: '搜索模式（glob 的通配符模式或 grep 的正则表达式）', required: false },
      { name: 'offset', type: 'integer', description: '起始行号，从 1 开始（action=read 时可选）', required: false },
      { name: 'limit', type: 'integer', description: '读取的最大行数（action=read 时可选）', required: false },
      { name: 'content', type: 'string', description: '文件内容（action=write 时必需）', required: false },
      { name: 'oldStr', type: 'string', description: '要搜索的原始内容（action=edit 时必需）', required: false },
      { name: 'newStr', type: 'string', description: '替换后的新内容（action=edit 时必需）', required: false },
    ],
    executor: 'file_manager' as any,
    code: undefined,
    language: 'typescript',
    createdAt: '2024-01-01T00:00:00.000Z',
    updatedAt: '2024-01-01T00:00:00.000Z'
  },
  {
    id: 'sys-task_manager',
    name: '任务管理',
    description: `任务管理和子任务委派工具。通过 action 参数指定操作类型。
- create_todo: 创建任务清单，用于规划复杂任务（>=3步）。自动创建任务文件夹并生成 task.md。
- update_todo: 更新任务状态（pending → in_progress → completed）。
- dispatch: 将独立子任务委派给子 Agent 执行，子 Agent 拥有独立执行上下文和工具集。`,
    logo: '',
    logoType: undefined,
    invokeName: 'task_manager',
    filePath: '',
    parameters: [
      { name: 'action', type: 'string', description: '操作类型: create_todo|update_todo|dispatch', required: true },
      { name: 'description', type: 'string', description: '子任务描述（action=dispatch 时必需）', required: false },
      { name: 'task_name', type: 'string', description: '任务名称（action=create_todo 时必需）', required: false },
      { name: 'todos', type: 'array', description: '任务数组（action=create_todo 时必需），每项含 id/content/status/priority', required: false },
      { name: 'task_id', type: 'string', description: '任务 ID（action=update_todo 时必需）', required: false },
      { name: 'status', type: 'string', description: '新状态: pending|in_progress|completed（action=update_todo 时必需）', required: false },
      { name: 'summary', type: 'string', description: '完成摘要（action=update_todo 时可选）', required: false },
      { name: 'tools', type: 'array', description: '工具白名单（action=dispatch 时可选，限制子 Agent 可用工具）', required: false },
      { name: 'max_iterations', type: 'integer', description: '最大迭代次数（action=dispatch 时可选，默认 20）', required: false },
    ],
    executor: 'task_manager' as any,
    code: undefined,
    language: 'typescript',
    createdAt: '2024-01-01T00:00:00.000Z',
    updatedAt: '2024-01-01T00:00:00.000Z'
  },
  {
    id: 'sys-browser',
    name: '浏览器控制',
    description: `浏览器自动化工具，基于 FairyAction 引擎。通过 action 参数指定操作类型。
- start: 启动浏览器。默认显示浏览器窗口。当不需要界面干扰或需要后台静默运行时，设置 show_browser=false 以使用 headless 模式
- stop: 关闭浏览器
- navigate: 导航到 URL（自动补全 https://）
- click: 点击指定索引的元素
- input: 向元素输入文本
- scroll: 滚动页面（direction: up/down, amount: 像素）
- send_keys: 发送按键（如 Enter, Control+a）
- screenshot: 截取页面截图
- extract: 提取页面文本内容（可选 query 参数描述提取目标）
- get_dom: 获取当前页面 DOM 层级表示（含可交互元素索引）
- get_state: 获取浏览器状态（URL、标题、标签页等）
- go_back / go_forward / reload: 浏览器导航
- switch_tab / close_tab / new_tab: 标签页管理
- search: 使用搜索引擎搜索，返回结构化结果（标题、URL、摘要）和提取的链接列表
- evaluate: 执行 JavaScript
- select_option: 选择下拉选项
- toggle_annotations: 切换元素标注覆盖层
- save_to_file: 保存内容到本地文件
- read_file: 读取本地文件内容
- wait: 等待指定秒数
- done: 标记浏览器任务完成，附带最终结果文本（可选 success 参数标识是否成功）

调用示例（必须严格遵循此 JSON 格式，action 和其他参数必须分开写，不能合并为一个字段）:
- 搜索: {"action": "search", "query": "关键词", "engine": "bing"}
- 导航: {"action": "navigate", "url": "https://example.com"}
- 点击: {"action": "click", "index": 3}`,
    logo: '',
    logoType: undefined,
    invokeName: 'browser',
    filePath: '',
    parameters: [
      { name: 'action', type: 'string', description: '操作类型（独立字段，请勿与其他字段合并）: start|stop|navigate|click|input|scroll|send_keys|screenshot|extract|get_dom|get_state|go_back|go_forward|reload|switch_tab|close_tab|new_tab|search|evaluate|select_option|toggle_annotations|save_to_file|read_file|wait|done', required: true },
      { name: 'url', type: 'string', description: '目标 URL（仅 action=navigate 或 action=new_tab 时使用，独立字段，请勿与 action 合并）', required: false },
      { name: 'index', type: 'integer', description: '元素索引，从 DOM 获取（action=click/input/screenshot/switch_tab/close_tab/select_option 时使用）', required: false },
      { name: 'text', type: 'string', description: '输入文本（action=input 时使用）', required: false },
      { name: 'query', type: 'string', description: '搜索关键词（仅 action=search 时使用）或提取目标描述（仅 action=extract 时使用），独立字段，请勿与 action 合并', required: false },
      { name: 'keys', type: 'string', description: '按键名称（action=send_keys 时使用，如 Enter, Tab, Control+a）', required: false },
      { name: 'direction', type: 'string', description: '滚动方向 up/down（action=scroll 时使用，默认 down）', required: false },
      { name: 'amount', type: 'number', description: '滚动像素量（action=scroll 时使用，默认视口高度）', required: false },
      { name: 'code', type: 'string', description: 'JavaScript 代码（action=evaluate 时使用）', required: false },
      { name: 'value', type: 'string', description: '选项值（action=select_option 时使用）', required: false },
      { name: 'show_browser', type: 'boolean', description: '是否显示浏览器窗口（action=start 时使用，默认 true 即显示模式。当需要后台静默运行时设为 false）', required: false },
      { name: 'show_empty', type: 'boolean', description: '是否显示空区块（action=get_dom 时使用，默认 false）', required: false },
      { name: 'engine', type: 'string', description: '搜索引擎（action=search 时使用，可选: bing/baidu/google/duckduckgo，默认 bing）', required: false },
      { name: 'seconds', type: 'integer', description: '等待秒数（action=wait 时使用，默认 3，最大 30）', required: false },
      { name: 'new_tab', type: 'boolean', description: '是否在新标签页打开（action=navigate 时使用）', required: false },
      { name: 'clear', type: 'boolean', description: '是否清除已有文本（action=input 时使用，默认 true）', required: false },
      { name: 'show', type: 'boolean', description: '显示/隐藏标注（action=toggle_annotations 时使用）', required: false },
      { name: 'file_name', type: 'string', description: '文件路径（action=save_to_file/read_file 时使用）', required: false },
      { name: 'content', type: 'string', description: '要保存的内容（action=save_to_file 时使用）', required: false },
      { name: 'text', type: 'string', description: '最终结果文本（action=done 时使用）', required: false },
      { name: 'success', type: 'boolean', description: '是否成功完成（action=done 时使用，默认 true）', required: false },
    ],
    executor: 'browser' as any,
    code: undefined,
    language: 'typescript',
    createdAt: '2024-01-01T00:00:00.000Z',
    updatedAt: '2024-01-01T00:00:00.000Z'
  }
]

export const memorySearchTool: Tool = {
  id: 'sys-memory_search',
  name: '记忆搜索',
  description: '搜索过往记忆，获取与查询相关的记忆片段。当你需要回忆之前对话的内容、用户偏好、项目背景等信息时调用此工具。',
  logo: '',
  logoType: undefined,
  invokeName: 'memory_search',
  filePath: '',
  parameters: [],
  executor: 'memory_search' as any,
  code: undefined,
  language: 'typescript',
  createdAt: '2024-01-01T00:00:00.000Z',
  updatedAt: '2024-01-01T00:00:00.000Z'
}

export const roleConfigTool: Tool = {
  id: 'sys-role_config',
  name: '角色配置',
  description: `读取或修改你的角色身份配置。通过 action 参数指定操作。
- get: 获取当前所有角色配置字段
- update: 修改指定的角色配置字段。只传需要修改的字段，未传的字段保持不变`,
  logo: '',
  logoType: undefined,
  invokeName: 'role_config',
  filePath: '',
  parameters: [
    { name: 'action', type: 'string', description: '操作类型: get|update', required: true },
    { name: 'fairyName', type: 'string', description: '你的名字（action=update 时可选）', required: false },
    { name: 'userName', type: 'string', description: '对用户的称呼（action=update 时可选）', required: false },
    { name: 'fairyPositioning', type: 'string', description: '你的角色定位，如"智能助手"、"创意伙伴"（action=update 时可选）', required: false },
    { name: 'fairyStyle', type: 'string', description: '你的交流风格，如"温柔体贴"、"简洁干练"（action=update 时可选）', required: false },
    { name: 'fairySupplement', type: 'string', description: '对你人格的额外补充描述（action=update 时可选）', required: false },
    { name: 'habitSupplement', type: 'string', description: '对你行为习惯的额外补充描述（action=update 时可选）', required: false },
  ],
  executor: 'role_config' as any,
  code: undefined,
  language: 'typescript',
  createdAt: '2024-01-01T00:00:00.000Z',
  updatedAt: '2024-01-01T00:00:00.000Z'
}
