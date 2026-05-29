---
name: browser-automation
category: operational
description: Browser control and web automation
version: 1.0.0
parameters:
  action:
    type: enum
    values: ["navigate", "click", "fill", "screenshot", "evaluate", "wait"]
    required: true
  url:
    type: string
    required: false
  selector:
    type: string
    required: false
---

# Browser Automation

## Steps
1. Launch or connect to browser instance
2. Perform requested action (navigate, click, fill, etc.)
3. Wait for page stability
4. Capture result (screenshot, DOM, console output)
5. Return structured result

## Tools Used
- browser_navigate, browser_click, browser_fill, browser_screenshot, browser_evaluate
