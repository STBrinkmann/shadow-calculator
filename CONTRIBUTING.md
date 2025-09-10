# Contributing to Shadow Calculator

> *"Help us make shadow calculations even more unnecessarily precise!"*

First off, thanks for considering contributing to this project! Whether you're fixing a bug, adding a feature, or just improving documentation, every contribution helps make this shadow-calculating machine a little bit better.

## 🚀 Quick Start for Contributors

### Got an Idea? Start Here!

1. **Check existing issues** - Maybe someone else had the same brilliant/terrible idea
2. **Open an issue** - Describe what you want to build/fix/improve
3. **Wait for feedback** - We'll discuss if it fits the project (spoiler: it probably does)
4. **Fork and code** - Make your magic happen
5. **Submit a PR** - We'll review it faster than shadows move at noon

### Setting Up Your Dev Environment

Follow the main [README installation guide](README.md#getting-started-aka-the-ritual) to get everything running locally. If you run into issues, that's probably our documentation's fault, not yours.

## 🎯 Types of Contributions We Love

### 🐛 Bug Reports
Found something broken? Great! We need:
- **What you expected** vs **what actually happened**
- **Steps to reproduce** the issue (the more detailed, the better)
- **Your environment** (OS, browser, Node/Rust versions)
- **Sample data** if it's data-related (we can't fix shadows we can't see)

### ✨ Feature Requests
Got ideas for new features? Awesome! Tell us:
- **What problem** you're trying to solve
- **How you envision** it working
- **Why others** might find it useful
- **If you're willing** to implement it yourself (bonus points!)

### 📝 Documentation Improvements
Documentation is never perfect. If you found something confusing, unclear, or just plain wrong, please let us know or fix it directly!

### 🔧 Code Contributions
Ready to dive into the code? Here's what we're looking for:

#### Frontend (React/TypeScript)
- UI/UX improvements
- Better data visualization
- Performance optimizations
- Mobile responsiveness (because someone will try to calculate shadows on their phone)

#### Backend (Rust)
- Algorithm optimizations
- New export formats
- Better error handling
- Memory usage improvements

#### Testing
- Unit tests for algorithms
- Integration tests for the full pipeline
- Performance benchmarks
- Edge case handling

## 🛠️ Development Guidelines

### Code Style

**Rust:**
- Follow `rustfmt` defaults (run `cargo fmt`)
- Use `clippy` for linting (`cargo clippy`)
- Write descriptive commit messages
- Comment complex algorithms (future you will thank present you)

**TypeScript:**
- Follow the existing patterns in the codebase
- Use meaningful variable names
- Add type definitions for new interfaces
- Keep components focused and reusable

### Testing Your Changes

Before submitting a PR:

1. **Make sure it compiles:**
   ```bash
   npm run build
   cd src-tauri && cargo build --release
   ```

2. **Test with actual data:**
   ```bash
   npm run tauri dev
   # Load some test data and make sure your changes work
   ```

3. **Run existing tests** (once we have them):
   ```bash
   cargo test
   npm test
   ```

### Commit Messages

We like commit messages that tell a story:

- **Good**: `Add hover tooltips to shadow visualization`
- **Better**: `Add hover tooltips showing shadow percentage on map overlay`
- **Best**: `Add interactive hover tooltips displaying shadow percentage with smooth follow cursor behavior`

## 📋 Pull Request Process

1. **Fork the repo** and create a feature branch from `main`
2. **Make your changes** following the guidelines above
3. **Update documentation** if needed (README, comments, etc.)
4. **Test everything** works as expected
5. **Submit a PR** with:
   - Clear title describing what you did
   - Description of the changes
   - Any relevant issue numbers
   - Screenshots if it's a UI change

### PR Review Process

We'll try to review PRs promptly, but remember:
- Feedback is about the code, not you personally
- We might ask for changes - that's normal!
- Small PRs are easier to review than large ones
- Tests and documentation make reviews faster

## 🚫 What We're NOT Looking For

- **Breaking changes** without good reason
- **Massive refactors** without prior discussion  
- **New dependencies** without justification
- **Code that only you understand** (comments are your friend)
- **Features that make the UI more complex** without significant benefit

## 🎨 Design Philosophy

When contributing, keep in mind our core principles:

- **Accuracy first** - Shadows should be calculated correctly
- **Performance matters** - Users shouldn't wait forever for results
- **Usability is key** - If it's hard to use, it's not worth having
- **Keep it simple** - Complexity should be hidden under the hood
- **Fail gracefully** - When things go wrong, fail with helpful messages

## 🆘 Getting Help

Stuck? Confused? Not sure if your idea fits? Don't hesitate to:

- **Open an issue** to discuss your idea before coding
- **Comment on existing issues** if you want to work on something
- **Ask questions** in PR discussions if you need clarification

## 🏆 Recognition

Contributors will be:
- Listed in the project's acknowledgments
- Mentioned in release notes for significant contributions
- Given credit in any academic papers that might result from this work
- Eternally grateful recipients of our virtual high-fives 🙌

## 📜 Code of Conduct

Be nice. Seriously, that's it. We're all here because we think calculating shadows with ridiculous precision is somehow important and/or fun. Let's keep it friendly and professional.

---

Thanks for contributing to making shadow calculations more awesome than they have any right to be! 🌞