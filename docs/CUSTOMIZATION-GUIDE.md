# 自定义指南

> 版本: 0.1.0-alpha
> 最后更新: 2026-07-11

---

## 概述

本指南提供 Hermes Game Operator 的自定义和扩展方法。

---

## 主题自定义

### CSS 变量

```css
:root {
  /* 主色调 */
  --primary-color: #3b82f6;
  --secondary-color: #10b981;
  
  /* 背景色 */
  --bg-primary: #ffffff;
  --bg-secondary: #f3f4f6;
  
  /* 文本色 */
  --text-primary: #111827;
  --text-secondary: #6b7280;
  
  /* 边框色 */
  --border-color: #e5e7eb;
  
  /* 阴影 */
  --shadow-sm: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
  --shadow-md: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
  --shadow-lg: 0 10px 15px -3px rgba(0, 0, 0, 0.1);
  
  /* 圆角 */
  --radius-sm: 0.25rem;
  --radius-md: 0.375rem;
  --radius-lg: 0.5rem;
  
  /* 间距 */
  --spacing-xs: 0.25rem;
  --spacing-sm: 0.5rem;
  --spacing-md: 1rem;
  --spacing-lg: 1.5rem;
  --spacing-xl: 2rem;
}

/* 暗色主题 */
[data-theme="dark"] {
  --bg-primary: #111827;
  --bg-secondary: #1f2937;
  --text-primary: #f9fafb;
  --text-secondary: #9ca3af;
  --border-color: #374151;
}
```

### 应用主题

```javascript
// 切换主题
function toggleTheme() {
  const currentTheme = document.documentElement.getAttribute('data-theme');
  const newTheme = currentTheme === 'dark' ? 'light' : 'dark';
  document.documentElement.setAttribute('data-theme', newTheme);
  localStorage.setItem('theme', newTheme);
}

// 加载保存的主题
const savedTheme = localStorage.getItem('theme');
if (savedTheme) {
  document.documentElement.setAttribute('data-theme', savedTheme);
}
```

---

## 布局自定义

### 侧边栏宽度

```css
.sidebar {
  width: 250px; /* 默认宽度 */
}

/* 紧凑模式 */
.sidebar.compact {
  width: 60px;
}

/* 响应式 */
@media (max-width: 768px) {
  .sidebar {
    position: fixed;
    left: -250px;
    transition: left 0.3s ease;
  }
  
  .sidebar.open {
    left: 0;
  }
}
```

### 主内容区域

```css
.main-content {
  margin-left: 250px;
  padding: 20px;
}

/* 无侧边栏 */
.main-content.no-sidebar {
  margin-left: 0;
}

/* 全宽 */
.main-content.full-width {
  max-width: 100%;
}
```

---

## 组件自定义

### 按钮样式

```css
/* 自定义按钮 */
.btn-custom {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border: none;
  padding: 10px 20px;
  border-radius: 8px;
  cursor: pointer;
  transition: transform 0.2s ease;
}

.btn-custom:hover {
  transform: translateY(-2px);
}

/* 禁用状态 */
.btn-custom:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
```

### 卡片样式

```css
.card-custom {
  background: white;
  border-radius: 12px;
  padding: 20px;
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  transition: box-shadow 0.3s ease;
}

.card-custom:hover {
  box-shadow: 0 10px 15px rgba(0, 0, 0, 0.15);
}

/* 带边框 */
.card-custom.bordered {
  border: 1px solid #e5e7eb;
}
```

---

## 插件开发

### 插件结构

```javascript
// my-plugin.js
export default {
  name: 'my-plugin',
  version: '1.0.0',
  
  // 安装钩子
  install(app, options) {
    // 注册全局组件
    app.component('MyComponent', MyComponent);
    
    // 添加全局方法
    app.config.globalProperties.$myMethod = () => {
      console.log('My method called');
    };
    
    // 提供依赖注入
    app.provide('myPlugin', {
      config: options
    });
  }
};
```

### 使用插件

```javascript
import { createApp } from 'vue';
import MyPlugin from './my-plugin';

const app = createApp(App);

app.use(MyPlugin, {
  option1: 'value1',
  option2: 'value2'
});

app.mount('#app');
```

---

## API 扩展

### 自定义 API 端点

```javascript
// server.js
app.post('/api/custom-endpoint', async (req, res) => {
  try {
    const { data } = req.body;
    
    // 处理逻辑
    const processedData = await processData(data);
    
    res.json({
      success: true,
      data: processedData
    });
  } catch (error) {
    res.status(500).json({
      success: false,
      error: error.message
    });
  }
});
```

### 中间件

```javascript
// middleware.js
function customMiddleware(req, res, next) {
  // 请求前处理
  console.log('Request received:', req.method, req.path);
  
  // 添加自定义头
  res.setHeader('X-Custom-Header', 'value');
  
  next();
}

// 使用中间件
app.use('/api/custom', customMiddleware);
```

---

## 钩子和事件

### 生命周期钩子

```javascript
// 任务创建前
client.hooks.beforeTaskCreate.tap('MyPlugin', async (taskData) => {
  // 修改任务数据
  taskData.metadata.custom = 'value';
  return taskData;
});

// 任务创建后
client.hooks.afterTaskCreate.tap('MyPlugin', async (task) => {
  console.log('Task created:', task.task_id);
});

// 任务更新前
client.hooks.beforeTaskUpdate.tap('MyPlugin', async (taskData) => {
  // 验证更新
  if (!taskData.goal) {
    throw new Error('Goal is required');
  }
  return taskData;
});
```

### 事件监听

```javascript
// 监听任务事件
client.events.on('task.created', (task) => {
  console.log('New task:', task.task_id);
});

// 监听审批事件
client.events.on('approval.requested', (approval) => {
  console.log('Approval requested:', approval.approval_id);
});

// 监听错误事件
client.events.on('error', (error) => {
  console.error('Error:', error.message);
});
```

---

## 主题开发

### 主题包结构

```
my-theme/
├── package.json
├── src/
│   ├── index.js
│   ├── components/
│   │   ├── Button.vue
│   │   ├── Card.vue
│   │   └── ...
│   └── styles/
│       ├── variables.scss
│       ├── mixins.scss
│       └── theme.scss
└── README.md
```

### 主题配置

```json
{
  "name": "my-theme",
  "version": "1.0.0",
  "main": "src/index.js",
  "theme": {
    "colors": {
      "primary": "#3b82f6",
      "secondary": "#10b981"
    },
    "fonts": {
      "primary": "Inter, sans-serif",
      "monospace": "Fira Code, monospace"
    }
  }
}
```

---

## 国际化扩展

### 添加新语言

```javascript
// locales/zh-TW.js
export default {
  common: {
    appName: 'Hermes Game Operator',
    save: '儲存',
    cancel: '取消'
  },
  task: {
    create: '建立任務',
    status: {
      running: '執行中',
      paused: '已暫停'
    }
  }
};
```

### 注册语言

```javascript
import zhTW from './locales/zh-TW';

i18n.setLocaleMessage('zh-TW', zhTW);
i18n.locale = 'zh-TW';
```

---

## 性能优化

### 懒加载组件

```javascript
// 异步加载组件
const AsyncComponent = () => import('./AsyncComponent.vue');

// 路由懒加载
const routes = [
  {
    path: '/dashboard',
    component: () => import('./views/Dashboard.vue')
  }
];
```

### 代码分割

```javascript
// webpack.config.js
module.exports = {
  optimization: {
    splitChunks: {
      chunks: 'all',
      cacheGroups: {
        vendor: {
          test: /[\\/]node_modules[\\/]/,
          name: 'vendors',
          chunks: 'all'
        }
      }
    }
  }
};
```

---

## 测试自定义

### 自定义测试工具

```javascript
// test-utils.js
import { mount } from '@vue/test-utils';
import { createI18n } from 'vue-i18n';

export function mountWithI18n(component, options = {}) {
  const i18n = createI18n({
    locale: 'en',
    messages: {
      en: options.messages || {}
    }
  });
  
  return mount(component, {
    global: {
      plugins: [i18n],
      ...options
    }
  });
}
```

### 自定义匹配器

```javascript
// matchers.js
expect.extend({
  toHaveTaskStatus(received, status) {
    const pass = received.status === status;
    return {
      pass,
      message: () => `expected task status to be ${status}`
    };
  }
});

// 使用
expect(task).toHaveTaskStatus('running');
```

---

## 部署自定义

### Docker 构建参数

```dockerfile
# Dockerfile
ARG NODE_VERSION=18
ARG APP_VERSION=0.1.0

FROM node:${NODE_VERSION}-alpine

LABEL version=${APP_VERSION}
LABEL maintainer="your-email@example.com"

# 构建参数
ARG API_URL
ENV VUE_APP_API_URL=${API_URL}
```

### 环境变量

```bash
# .env.production
VUE_APP_API_URL=https://api.your-domain.com
VUE_APP_ENABLE_ANALYTICS=true
VUE_APP_LOG_LEVEL=error
```

---

## 示例

### 完整自定义插件

```javascript
// custom-plugin.js
export default {
  name: 'custom-plugin',
  
  install(app, options) {
    // 注册组件
    app.component('CustomButton', {
      template: '<button class="custom-btn"><slot></slot></button>',
      props: ['variant'],
      computed: {
        className() {
          return `custom-btn--${this.variant}`;
        }
      }
    });
    
    // 添加全局方法
    app.config.globalProperties.$formatDate = (date) => {
      return new Date(date).toLocaleDateString();
    };
    
    // 提供配置
    app.provide('customConfig', options);
  }
};
```

---

**最后更新**: 2026-07-11
**维护者**: Hermes Game Operator Customization Team