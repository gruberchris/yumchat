# GitHub Copilot Context Files

This directory contains comprehensive documentation for GitHub Copilot to generate code consistent with the YumChat project's standards, architecture, and exact technology versions.

## Files

### 📋 copilot-instructions.md (16KB)
**Primary guidance file for GitHub Copilot**

Contains:
- Priority guidelines (version compatibility, architecture, code quality)
- Exact technology versions and constraints
- Code quality standards (maintainability, testing, documentation)
- Rust-specific patterns and idioms
- Ratatui UI patterns
- Tokio async patterns
- Error handling strategies
- Testing requirements and patterns
- Common code patterns and anti-patterns

**Use this for**: All code generation tasks

### 🏗️ architecture.md (14KB)
**System architecture and design patterns**

Contains:
- High-level architecture overview
- Layer responsibilities and boundaries
- Data flow diagrams
- Concurrency model (hybrid sync/async)
- Error handling architecture
- Testing architecture
- Performance considerations
- Extensibility points
- Design principles

**Use this for**: Understanding system design and component interactions

### 🔧 tech-stack.md (8KB)
**Detailed technology stack documentation**

Contains:
- Language and runtime specifications
- All dependencies with exact versions
- Build configuration
- Platform support details
- External API specifications
- Version compatibility notes
- Known limitations
- Performance characteristics

**Use this for**: Version-specific queries and dependency management

## How GitHub Copilot Uses These Files

GitHub Copilot will automatically reference these files when:
1. Generating new code in this repository
2. Suggesting code completions
3. Answering questions about the codebase
4. Proposing refactoring or improvements

The files are prioritized in this order:
1. **copilot-instructions.md** - Primary guidance
2. **architecture.md** - Architectural decisions
3. **tech-stack.md** - Technical specifications

## Updating These Files

When making significant changes to the project:

### Update copilot-instructions.md when:
- Adding new patterns or conventions
- Changing code quality standards
- Updating testing approaches
- Introducing new architectural patterns

### Update architecture.md when:
- Adding new layers or components
- Changing data flow patterns
- Modifying the concurrency model
- Introducing new design patterns

### Update tech-stack.md when:
- Upgrading dependencies
- Adding new dependencies
- Changing build configuration
- Updating platform support

## File Maintenance

### Version Sync
All three files must stay synchronized with:
- `Cargo.toml` - Dependency versions
- `src/` - Code patterns and architecture
- Tests - Testing patterns and coverage

### Review Checklist
Before committing changes:
- [ ] Version numbers match Cargo.toml
- [ ] Patterns match actual code
- [ ] Examples are up-to-date
- [ ] No assumptions about unimplemented features
- [ ] Architecture diagrams reflect current structure

## Key Principles

These context files follow these principles:

1. **Evidence-Based**: Only document patterns that exist in the code
2. **Version-Specific**: Always specify exact versions
3. **Pattern-First**: Show concrete examples, not abstract rules
4. **Maintenance-Friendly**: Keep docs synchronized with code
5. **Copilot-Optimized**: Structured for AI consumption

## Quick Reference

### For New Contributors
1. Read **architecture.md** first (high-level understanding)
2. Review **tech-stack.md** for technology details
3. Reference **copilot-instructions.md** while coding

### For Copilot Integration
- Primary file: **copilot-instructions.md**
- Context depth: All three files are loaded automatically
- Update frequency: After significant changes

### For Documentation Updates
1. Make code changes first
2. Update relevant context file(s)
3. Verify examples still work
4. Check version numbers are current
5. Commit documentation with code changes

## Benefits

Having these comprehensive context files ensures:

✅ **Consistency**: Copilot generates code matching our patterns
✅ **Quality**: Maintains our zero-warning standard
✅ **Compatibility**: Uses correct versions of dependencies
✅ **Architecture**: Respects layer boundaries
✅ **Testing**: Follows our test patterns
✅ **Maintainability**: Generated code is maintainable

## Examples

### Good Copilot Usage
```rust
// Copilot knows to use anyhow::Result with context
pub fn load_config() -> Result<AppConfig> {
    let path = get_config_path()?;
    fs::read_to_string(&path)
        .context("Failed to read config")?
        // Copilot suggests proper error handling
}
```

### Copilot Follows Our Patterns
- Uses exact crate versions from Cargo.toml
- Follows our async/sync hybrid model
- Adds proper error context
- Writes tests alongside code
- Uses our naming conventions
- Respects architectural boundaries

## Support

For questions about these context files:
1. Check the existing code for patterns
2. Review the specific context file
3. Ensure Copilot is using the latest context
4. Update context files if patterns have changed

## Version

Context files version: 1.0
Last updated: 2026-02-01
YumChat version: 0.1.0
Implementation: ~60% complete (Phases 1-7 Part 2)
