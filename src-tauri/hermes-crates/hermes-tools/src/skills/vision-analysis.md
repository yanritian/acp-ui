---
name: vision-analysis
category: operational
description: Image understanding and analysis
version: 1.0.0
parameters:
  image:
    type: string
    required: true
    description: Image URL or base64 data
  question:
    type: string
    required: false
    description: Question about the image
---

# Vision Analysis

## Steps
1. Load image from URL or decode base64
2. Run vision model analysis
3. Extract relevant information
4. Return structured analysis

## Tools Used
- vision_analyze
