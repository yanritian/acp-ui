# Phase 4: Marketing & Finance Agents

## Overview

Phase 4 extends Agent Platform to support marketing and finance scenarios:
- Marketing: Kimi (文案生成), Kling (视频生成), Jimeng (图像生成)
- Finance: WPS (办公协作), 飞书/钉钉/企业微信对接

## Timeline: 4 Weeks

### Week 1: Marketing Content Adapters

**Goal**: Marketing content generation support

#### Tasks

1. **Kimi Adapter** (`src-tauri/src/agent_adapter/kimi_adapter.rs`)
   - Kimi API integration (Moonshot AI)
   - Capabilities: copywriting, content-rewrite, translation
   - Token tracking via API headers

2. **Jimeng Adapter** (`src-tauri/src/agent_adapter/jimeng_adapter.rs`)
   - Jimeng API (即梦 AI 图像生成)
   - Capabilities: image-gen, style-transfer
   - Cost per image generation

3. **Marketing Tauri Commands**
   - `kimi_generate_text` - Text generation
   - `kimi_translate` - Translation
   - `jimeng_generate_image` - Image generation

### Week 2: Video & Media Adapters

**Goal**: Video generation and media processing

#### Tasks

1. **Kling Adapter** (`src-tauri/src/agent_adapter/kling_adapter.rs`)
   - Kling API (可灵 AI 视频生成)
   - Capabilities: video-gen, video-edit, avatar-gen
   - Async job polling (video generation is slow)

2. **Media Processing Commands**
   - `kling_generate_video` - Video generation
   - `kling_generate_avatar` - AI avatar
   - `media_get_job_status` - Async job polling

### Week 3: Finance & Office Adapters

**Goal**: Office automation and finance integration

#### Tasks

1. **WPS Adapter** (`src-tauri/src/agent_adapter/wps_adapter.rs`)
   - WPS AI API integration
   - Capabilities: doc-gen, ppt-gen, excel-analysis
   - Office file format support

2. **Feishu Integration Enhancement**
   - Extend existing `feishu_adapter` for finance approval
   - Message parsing for financial reports

3. **Finance Commands**
   - `wps_generate_doc` - Document generation
   - `wps_analyze_excel` - Excel analysis
   - `finance_parse_report` - Financial report parsing

### Week 4: API Cost Tracking & Budget

**Goal**: Unified cost tracking across all adapters

#### Tasks

1. **Cost Tracker** (`src-tauri/src/cost_tracker.rs`)
   - Real-time cost aggregation
   - Budget limit enforcement
   - Cost alerts when approaching limit

2. **Token Dashboard Types**
   - Per-adapter token usage
   - Daily/weekly/monthly totals
   - Cost per task breakdown

3. **Budget Commands**
   - `budget_set_limit` - Set budget limit
   - `budget_get_usage` - Get current usage
   - `budget_get_forecast` - Get forecast

## New Types

```rust
pub enum MarketingAsset {
    TextCopy,
    Image,
    Video,
    Avatar,
    StyleGuide,
}

pub enum FinanceDocument {
    Report,
    Invoice,
    Contract,
    Spreadsheet,
}

pub struct ApiConfig {
    api_key: String,
    base_url: String,
    rate_limit: u32,
    timeout_ms: u64,
}

pub struct CostBudget {
    daily_limit: f32,
    monthly_limit: f32,
    currency: String,
}
```

## New Tauri Commands

| Command | Description |
|---------|-------------|
| `kimi_generate_text` | Kimi text generation |
| `kimi_translate` | Translation |
| `jimeng_generate_image` | Image generation |
| `kling_generate_video` | Video generation |
| `kling_get_job_status` | Async job status |
| `wps_generate_doc` | Document generation |
| `wps_analyze_excel` | Excel analysis |
| `budget_set_limit` | Set budget |
| `budget_get_usage` | Get usage |
| `budget_get_forecast` | Cost forecast |

## Success Criteria

1. ✅ Kimi adapter generates text successfully
2. ✅ Jimeng adapter generates images
3. ✅ Kling adapter creates videos (async)
4. ✅ WPS adapter processes documents
5. ✅ Cost tracking works across all adapters
6. ✅ Budget limit enforcement
7. ✅ E2E tests for marketing flows

## Risks

1. **API keys**: Requires external API credentials
2. **Rate limits**: External API rate limiting
3. **Async jobs**: Video generation takes minutes
4. **Cost**: API calls incur actual costs

## Mitigation

1. Store API keys securely (encrypted)
2. Implement retry with exponential backoff
3. Async job polling with timeout
4. Budget limit before execution