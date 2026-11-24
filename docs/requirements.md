---
title: Requirements
weight: 3
---

The lighthouse-rust package requires Rust 1.89 or newer, Node.js 22.19 or newer, Lighthouse, chrome-launcher 1, and Chrome or Chromium. CI tests Lighthouse 13.6.0 on all three operating systems and Lighthouse 11.7.1 on Linux for its budget audits.

Use current Lighthouse for new applications. Lighthouse 12 removed performance budgets, and current releases no longer include PWA and service-worker audits. The wrapper retains budget configuration and historical report access; running budget audits requires Lighthouse 11.7.1. This older runtime is optional, not recommended as the default.

Lighthouse runs locally. It still needs Node and a browser installed on the machine running your application. Installing the Rust crate does not download these tools.

The package is tested on Linux, macOS and Windows. Set `CHROME_PATH` when chrome-launcher cannot find your Chrome installation.
