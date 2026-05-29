---
name: web-research
category: operational
description: Web search, extraction, and research capabilities
version: 1.0.0
parameters:
  query:
    type: string
    required: true
    description: Search query
  sources:
    type: array
    required: false
    default: ["google", "bing"]
    description: Search sources to use
  max_results:
    type: integer
    required: false
    default: 10
    description: Maximum results to return
---

# Web Research

## Description
Perform web searches and extract information from web pages.

## Steps
1. Parse the search query and identify key terms
2. Search across configured sources (Google, Bing, etc.)
3. Extract relevant snippets and URLs
4. If deep extraction needed, fetch and parse target pages
5. Summarize findings with source attribution

## Tools Used
- web_search
- web_extract

## Output Format
Return structured results with: title, URL, snippet, relevance score.
