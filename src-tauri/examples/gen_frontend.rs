//! 单独生成前端代码

use std::sync::Arc;
use std::fs;
use std::path::PathBuf;
use regex::Regex;
use hermes_core::Message;

#[tokio::main]
async fn main() {
    println!("=== 生成前端 Vue 代码 ===\n");

    let project_dir = PathBuf::from("D:/smallProject");

    // 配置 Hermes
    let config_dir = Some("D:/dingsun/acp-ui/hermes".to_string());
    let model = "alibaba-coding-plan:qwen3.6-plus".to_string();
    let gateway_config = hermes_config::load_config(config_dir.as_deref()).unwrap();

    let agent_config = hermes_agent::agent_builder::build_agent_config(&gateway_config, &model, Some("erp-frontend"));
    let llm_provider = hermes_agent::agent_builder::build_provider(&gateway_config, &model);
    let tools = hermes_tools::ToolRegistry::new();
    let tool_registry = Arc::new(hermes_agent::agent_builder::bridge_tool_registry(&tools));
    let agent_loop = hermes_agent::AgentLoop::new(agent_config, tool_registry, llm_provider);

    let frontend_prompt = r#"你是 Vue 3 前端开发专家。根据 ERP 需求，生成 Vue 3 + TypeScript 前端项目代码。

请严格使用以下格式输出每个文件：

### FILE: frontend/package.json
```json
{
  "name": "erp-frontend",
  ...
}
```

### FILE: frontend/vite.config.ts
```typescript
import { defineConfig } from 'vite'
...
```

### FILE: frontend/src/main.ts
```typescript
import { createApp } from 'vue'
...
```

### FILE: frontend/src/App.vue
```vue
<template>
  ...
</template>
```

### FILE: frontend/src/views/ProductManagement.vue
```vue
<template>
  ...
</template>
```

### FILE: frontend/src/api/index.ts
```typescript
import axios from 'axios'
...
```

### FILE: frontend/src/router/index.ts
```typescript
import { createRouter } from 'vue-router'
...
```

请确保每个文件都使用 ### FILE: 文件名 格式开头。
代码要求：
1. Vue 3 Composition API
2. TypeScript
3. Element Plus UI
4. Axios HTTP
5. 中文界面"#;

    let messages = vec![Message::user(frontend_prompt)];
    println!("执行前端代码生成...\n");

    let result = agent_loop.run(messages, None).await.unwrap();
    let response = result.messages.iter()
        .rev()
        .find_map(|m| if m.role == hermes_core::MessageRole::Assistant { m.content.clone() } else { None })
        .unwrap_or_default();

    println!("前端代码生成完成 ({} 字符)\n", response.len());

    // 使用更宽松的正则
    let pattern = Regex::new(r"###\s*FILE:\s*[`]?([^`\n]+)[`]?[\s\n]+```[\w]*\s*([\s\S]*?)```").unwrap();

    let mut files_written = 0;
    for caps in pattern.captures_iter(&response) {
        let file_path = caps[1].trim();
        let content = caps[2].trim();

        let full_path = project_dir.join(file_path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&full_path, content).unwrap();
        println!("✅ 写入: {}", file_path);
        files_written += 1;
    }

    println!("\n前端文件数: {}", files_written);

    // 显示部分响应内容
    println!("\n响应预览:\n{}", response.lines().take(30).collect::<Vec<_>>().join("\n"));
}