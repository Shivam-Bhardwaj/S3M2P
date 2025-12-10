/**
 * ═══════════════════════════════════════════════════════════════════════════════
 * FILE: scan_docs.js | SCRIPTS/scan_docs.js
 * PURPOSE: Scans codebase extracting documentation and purpose from source files into JSON database
 * MODIFIED: 2025-12-09
 * ═══════════════════════════════════════════════════════════════════════════════
 */

const fs = require('fs');
const path = require('path');

const ROOT_DIR = path.resolve(__dirname, '..');
const OUTPUT_FILE = path.join(ROOT_DIR, 'ARCH', 'src', 'db.json');

const IGNORE_DIRS = ['target', 'node_modules', '.git', '.vscode', '.idea', 'dist', 'build', 'assets', 'images'];
const IGNORE_FILES = ['Cargo.lock', 'Cargo.toml', 'package-lock.json', 'package.json', 'tsconfig.json', '.gitignore', 'LICENSE', 'Caddyfile'];
const INCLUDE_EXTS = ['.rs', '.js', '.ts', '.html', '.css', '.md', '.sh'];
const MAX_FILE_SIZE = 100 * 1024; // 100KB limit for file content

function getPurpose(content, ext) {
    const lines = content.split('\n');

    // First pass: look for explicit PURPOSE: line (our header convention)
    for (let line of lines) {
        line = line.trim();
        // Remove comment prefixes
        line = line.replace(/^\/\/[!\/]?\s*/, '').replace(/^\*\s*/, '').replace(/^<!--\s*/, '');

        if (line.match(/^PURPOSE:\s*(.+)/i)) {
            const purpose = line.replace(/^PURPOSE:\s*/i, '').trim();
            if (purpose && purpose.length > 0) {
                return purpose;
            }
        }
    }

    // Second pass: extract from doc comments or headers
    let buffer = [];
    for (let line of lines) {
        line = line.trim();
        if (!line) continue;

        // Skip separator lines (═, ─, -, =, *)
        if (line.match(/^[═─\-=*]{3,}$/)) continue;
        if (line.match(/^\/\/[!\/]?\s*[═─\-=*]{3,}$/)) continue;
        if (line.match(/^<!--[═─\-=*\s]*$/)) continue;

        // Skip FILE: lines
        if (line.match(/FILE:/)) continue;
        // Skip MODIFIED: lines
        if (line.match(/MODIFIED:/)) continue;
        // Skip LAYER: lines
        if (line.match(/LAYER:/)) continue;

        let cleanLine = null;

        if (ext === '.rs') {
            if (line.startsWith('//!')) cleanLine = line.replace('//!', '').trim();
            else if (line.startsWith('///')) cleanLine = line.replace('///', '').trim();
        } else if (ext === '.js' || ext === '.ts') {
            if (line.startsWith('/**')) continue;
            if (line.startsWith('*/')) continue;
            if (line.startsWith('*')) cleanLine = line.replace('*', '').trim();
        } else if (ext === '.md') {
            if (line.startsWith('#')) cleanLine = line.replace(/^#+/, '').trim();
            else if (buffer.length === 0 && line.length > 0) cleanLine = line;
        } else if (ext === '.html') {
            if (line.startsWith('<!--')) continue;
            if (line.startsWith('-->')) continue;
        }

        if (cleanLine && cleanLine.length > 0) {
            // Skip common noise
            if (cleanLine.toLowerCase().startsWith('copyright')) continue;
            if (cleanLine.toLowerCase().startsWith('license')) continue;
            if (cleanLine.match(/^[═─\-=*]+$/)) continue;

            buffer.push(cleanLine);
            if (buffer.length >= 1) break; // Just grab first meaningful line
        }
    }

    if (buffer.length > 0) return buffer[0];
    return "No description available.";
}

function getMainFunction(content, ext) {
    if (ext !== '.rs' && ext !== '.js' && ext !== '.ts') return "N/A";

    // Simple regex pointers
    if (content.match(/fn\s+main\s*\(/)) return "fn main()";

    // Find first pub fn or exported function
    const pubFnMatch = content.match(/pub\s+fn\s+([a-z0-9_]+)/);
    if (pubFnMatch) return pubFnMatch[1];

    const jsFnMatch = content.match(/export\s+(?:async\s+)?function\s+([a-zA-Z0-9_]+)/);
    if (jsFnMatch) return jsFnMatch[1];

    if (content.includes('struct ')) {
        const structMatch = content.match(/pub\s+struct\s+([a-zA-Z0-9_]+)/);
        if (structMatch) return "struct " + structMatch[1];
    }

    return "Module/Lib";
}

function walk(dir, fileList = []) {
    const files = fs.readdirSync(dir);
    for (const file of files) {
        const filePath = path.join(dir, file);
        const stat = fs.statSync(filePath);
        if (stat.isDirectory()) {
            if (!IGNORE_DIRS.includes(file)) {
                walk(filePath, fileList);
            }
        } else {
            if (IGNORE_FILES.includes(file)) continue;
            const ext = path.extname(file);
            if (INCLUDE_EXTS.includes(ext)) {
                fileList.push(filePath);
            }
        }
    }
    return fileList;
}

const db = {};

const dirsToScan = ['DNA', 'TOOLS', 'SIMULATION', 'HELIOS', 'BLOG', 'LEARN', 'ARCH', 'WELCOME'];

dirsToScan.forEach(dirName => {
    const dirPath = path.join(ROOT_DIR, dirName);
    if (fs.existsSync(dirPath)) {
        const files = walk(dirPath);
        files.forEach(f => {
            const relPath = path.relative(ROOT_DIR, f);
            const content = fs.readFileSync(f, 'utf-8');
            const stats = fs.statSync(f);

            // Include full file content if under size limit
            let fileContent = null;
            if (stats.size <= MAX_FILE_SIZE) {
                fileContent = content;
            } else {
                fileContent = `[File too large to display: ${stats.size} bytes. Size limit: ${MAX_FILE_SIZE} bytes]`;
            }

            db[relPath] = {
                path: relPath,
                name: path.basename(f),
                purpose: getPurpose(content, path.extname(f)),
                main_function: getMainFunction(content, path.extname(f)),
                type: path.extname(f),
                content: fileContent
            };
        });
    }
});

fs.writeFileSync(OUTPUT_FILE, JSON.stringify(db, null, 2));
console.log(`Documentation DB generated at ${OUTPUT_FILE}`);
