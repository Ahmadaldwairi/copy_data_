#!/bin/bash
# Script to initialize git repo and push to GitHub

set -e

cd "$(dirname "$0")"

echo "========================================"
echo "Git Repository Setup"
echo "========================================"
echo ""

# Check if already a git repo
if [ -d .git ]; then
    echo "✓ Git repository already initialized"
else
    echo "Initializing git repository..."
    git init
    echo "✓ Git initialized"
fi

echo ""
echo "Checking git status..."
git status

echo ""
echo "========================================"
echo "Next Steps:"
echo "========================================"
echo ""
echo "1. Create a new repository on GitHub:"
echo "   - Go to: https://github.com/new"
echo "   - Repository name: copytrader-bot (or your choice)"
echo "   - Keep it Private (recommended)"
echo "   - Do NOT initialize with README"
echo ""
echo "2. After creating the repo, GitHub will show you commands."
echo "   Copy the remote URL (looks like: git@github.com:username/copytrader-bot.git)"
echo ""
echo "3. Then run these commands:"
echo ""
echo "   # Add all files (respects .gitignore)"
echo "   git add ."
echo ""
echo "   # Create initial commit"
echo "   git commit -m \"Initial commit: Pump.fun copytrader bot with SOL tracking\""
echo ""
echo "   # Add your GitHub remote (replace with your URL)"
echo "   git remote add origin git@github.com:YOUR_USERNAME/copytrader-bot.git"
echo ""
echo "   # Push to GitHub"
echo "   git branch -M main"
echo "   git push -u origin main"
echo ""
echo "========================================"
echo "What will be EXCLUDED (via .gitignore):"
echo "========================================"
echo "✓ /execution/ folder (your private execution bot)"
echo "✓ /target/ folder (build artifacts)"
echo "✓ /logs/ folder (log files)"
echo "✓ /data/ folder (database files)"
echo "✓ .env file (your credentials)"
echo "✓ Cargo.lock"
echo ""
echo "What WILL be included:"
echo "========================================"
echo "✓ All source code (crates/)"
echo "✓ Documentation (docs/)"
echo "✓ Configuration templates (configs/)"
echo "✓ SQL schemas (sql/)"
echo "✓ Scripts (scripts/)"
echo "✓ README and other docs"
echo "✓ .gitignore itself"
echo ""
echo "========================================"
echo "Ready to proceed!"
echo "========================================"
