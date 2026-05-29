---
name: media-generation
category: operational
description: Generate images, video, and audio
version: 1.0.0
parameters:
  type:
    type: enum
    values: ["image", "video", "audio"]
    required: true
  prompt:
    type: string
    required: true
    description: Generation prompt
  style:
    type: string
    required: false
---

# Media Generation

## Steps
1. Parse generation request and validate parameters
2. Select appropriate model (GPT-Image, DALL-E, etc.)
3. Generate media content
4. Save output and return URL/path

## Tools Used
- image_gen, gpt_image, video_gen, audio_gen
