---
layout: default
title: null-e - Disk Cleanup Tool for Developers
description: Clean node_modules, target, .venv, Docker images, Xcode caches and 50+ cache types. Reclaim 100+ GB of disk space.
---

<div class="section">
  <h2>The Problem</h2>
  <p>As developers, our disks fill up fast:</p>
  <ul>
    <li><strong>node_modules</strong> folders everywhere (500MB-2GB each)</li>
    <li><strong>Rust target/</strong> directories eating gigabytes</li>
    <li><strong>Python .venv</strong> scattered across projects</li>
    <li><strong>Docker images</strong> you forgot about</li>
    <li><strong>Xcode DerivedData</strong> growing endlessly</li>
    <li><strong>Global caches</strong> from npm, pip, cargo, homebrew...</li>
  </ul>
  <p><strong>The result?</strong> "Your disk is almost full" notifications.</p>
</div>

<div class="section">
  <h2>The Solution</h2>
  <pre><code>cargo install null-e
null-e sweep</code></pre>
  <p>null-e scans your system and finds everything that can be safely cleaned:</p>
  
  <table>
    <thead>
      <tr>
        <th>Category</th>
        <th>What it finds</th>
        <th>Typical savings</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td><strong>Project Artifacts</strong></td>
        <td>node_modules, target, .venv, build</td>
        <td>10-100 GB</td>
      </tr>
      <tr>
        <td><strong>Global Caches</strong></td>
        <td>npm, pip, cargo, go, maven</td>
        <td>5-50 GB</td>
      </tr>
      <tr>
        <td><strong>Xcode</strong></td>
        <td>DerivedData, Simulators, Archives</td>
        <td>20-100 GB</td>
      </tr>
      <tr>
        <td><strong>Docker</strong></td>
        <td>Images, Containers, Build Cache</td>
        <td>10-100 GB</td>
      </tr>
      <tr>
        <td><strong>ML/AI</strong></td>
        <td>Huggingface, Ollama, PyTorch</td>
        <td>10-100 GB</td>
      </tr>
      <tr>
        <td><strong>IDE Caches</strong></td>
        <td>JetBrains, VS Code, Cursor</td>
        <td>2-20 GB</td>
      </tr>
    </tbody>
  </table>
</div>

<div class="section">
  <h2>Features</h2>
  <div class="features-grid">
    <div class="feature-card">
      <div class="feature-icon">⚡</div>
      <h3>Fast</h3>
      <p>Parallel scanning with Rust. Scans thousands of files in seconds.</p>
    </div>
    <div class="feature-card">
      <div class="feature-icon">🛡️</div>
      <h3>Safe</h3>
      <p>Git protection, moves to trash by default. Never lose work.</p>
    </div>
    <div class="feature-card">
      <div class="feature-icon">🎯</div>
      <h3>Smart</h3>
      <p>Detects 15+ languages and frameworks. Knows what's safe to delete.</p>
    </div>
    <div class="feature-card">
      <div class="feature-icon">💻</div>
      <h3>Cross-platform</h3>
      <p>Works on macOS, Linux, and Windows. Same commands everywhere.</p>
    </div>
  </div>
</div>

<div class="section">
  <h2>Quick Start</h2>
  <h3>Install</h3>
  <pre><code>cargo install null-e</code></pre>
  
  <h3>Basic Usage</h3>
  <pre><code># Scan current directory
null-e

# Deep sweep - find EVERYTHING
null-e sweep

# Clean global caches
null-e caches

# Xcode cleanup (macOS)
null-e xcode

# Docker cleanup
null-e docker</code></pre>
</div>

<div class="section">
  <h2>Why "null-e"?</h2>
  <blockquote>
    <code>/dev/null</code> + Wall-E = <strong>null-e</strong>
    <br><br>
    Like the adorable trash-compacting robot from the movie, null-e tirelessly cleans up your developer junk and sends it where it belongs!
  </blockquote>
</div>

<div class="section">
  <h2>Installation Methods</h2>
  
  <h3>Cargo (Recommended)</h3>
  <pre><code>cargo install null-e</code></pre>
  
  <h3>Pre-built Binaries</h3>
  <p>Download from <a href="https://github.com/us/null-e/releases">GitHub Releases</a></p>
  
  <h3>Package Managers</h3>
  <pre><code># Homebrew (coming soon)
brew install null-e

# AUR (Arch Linux)
yay -S null-e

# Scoop (Windows)
scoop install null-e</code></pre>
</div>

<div class="section">
  <h2>Documentation</h2>
  <ul class="blog-list">
    {% for post in site.posts limit:10 %}
    <li class="blog-item">
      <a href="{{ post.url | relative_url }}">{{ post.title }}</a>
      <div class="blog-meta">
        {{ post.date | date: "%B %d, %Y" }}
        {% if post.tags %}
        {% for tag in post.tags limit:3 %}
        <span class="tag">{{ tag }}</span>
        {% endfor %}
        {% endif %n      </div>
    </li>
    {% endfor %}
  </ul>
  <p style="text-align: center; margin-top: 1.5rem;">
    <a href="{{ site.baseurl }}/blog/" style="color: var(--primary); font-weight: 600;">View all {{ site.posts | size }} guides →</a>
  </p>
</div>
