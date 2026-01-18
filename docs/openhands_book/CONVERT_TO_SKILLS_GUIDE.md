# OpenHands Documentation to Skills Conversion Guide

This guide explains how to convert the OpenHands Documentation Book into OpenHands Skills that can be used by the OpenHands AI agent.

## Understanding OpenHands Skills

OpenHands Skills are specialized prompts that enhance the agent with domain-specific knowledge and expertise. There are two main types:

### 1. Knowledge Skills (Public/Shareable)

- Located in `OpenHands/skills/` directory
- Available to all OpenHands users
- Triggered by keywords in conversations
- Provide expertise on specific topics (GitHub, Docker, Python, etc.)

### 2. Repository Skills (Private)

- Located in `.openhands/skills/` directory within repositories
- Specific to individual repositories
- Automatically loaded when working with that repository
- Perfect for project-specific guidelines

## Skill File Format

OpenHands skills use markdown files with YAML frontmatter. Example format:

```markdown
---
name: skill-name       # Unique identifier
type: knowledge        # 'knowledge' or 'repository'
version: 1.0.0         # Version number
agent: CodeActAgent    # Agent type (usually CodeActAgent)
triggers:              # Keywords that activate this skill
- trigger1
- trigger2
---
```

## Conversion Strategy

### Option 1: Create Knowledge Skills from Documentation

Convert documentation entries into reusable knowledge skills.

#### Steps:

1. **Identify suitable documentation entries** to convert to skills:
   - API references with specific usage patterns
   - Configuration guides that provide expert knowledge
   - Troubleshooting guides that offer problem-solving expertise
   - Best practice guides for specific technologies

2. **Create skill files** based on documentation:
   ```bash
   # Create directory for new skills
   mkdir -p skills/openhands-docs

   # Create a skill from a documentation entry
   cp docs/get-microagents.md skills/openhands-docs/microagents.md
   ```

3. **Add YAML frontmatter** to convert documentation to a skill:
   ```markdown
   ---
   name: get-microagents
   type: knowledge
   version: 1.0.0
   agent: CodeActAgent
   triggers:
   - microagents
   - get microagents
   - repository microagents
   ---
   ```

4. **Enhance the content** to be skill-like:
   - Add clear instructions
   - Include examples
   - Provide troubleshooting tips
   - Add best practices

### Option 2: Create Repository Skills

Use the documentation as a foundation for repository-specific skills.

#### Steps:

1. **Create repository skill directory**:
   ```bash
   mkdir -p .openhands/skills
   ```

2. **Create a repo.md file** with documentation content:
   ```markdown
   ---
   name: openhands-repo-docs
   type: repository
   version: 1.0.0
   agent: CodeActAgent
   ---
   ```

3. **Add documentation content** with skill enhancements.

### Option 3: Create Hybrid Approach

Combine multiple documentation entries into comprehensive skills.

#### Example: Create an API Guide Skill

1. **Identify related documentation**:
   - All API reference entries
   - Configuration options
   - Usage guides

2. **Create comprehensive skill**:
   ```markdown
   ---
   name: openhands-api-guide
   type: knowledge
   version: 1.0.0
   agent: CodeActAgent
   triggers:
   - openhands api
   - api reference
   - openhands endpoints
   ---
   ```

3. **Organize content** by categories with clear sections.

## Example: Converting Documentation to Skills

### Before (Documentation):

```markdown
# Git Changes

URL: https://docs.openhands.dev/api-reference/git-changes.md

## Description

Get the status of git changes in the repository.

Args:
    path: Path to the repository (default: current working directory)
    use_git_status: Whether to use git status (default: true)

Returns:
    Dictionary containing:
    - added: List of added files
    - modified: List of modified files
    - deleted: List of deleted files
    - renamed: List of renamed files
```

### After (Skill):

```markdown
---
name: git-changes
type: knowledge
version: 1.0.0
agent: CodeActAgent
triggers:
- git changes
- git status
- what files changed
- check git modifications
---

# Git Changes Skill

## Usage

Use the `git_changes` tool to check the status of git changes in the repository.

## Examples

### Get current git status:
```bash
git_changes()
```

### Check changes in specific directory:
```bash
git_changes(path="/path/to/repo")
```

## Best Practices

1. Always check git status before making changes
2. Use `git_changes` to identify modified files before committing
3. Check for conflicts before pushing changes

## Troubleshooting

If git_changes doesn't work:
- Ensure you're in a git repository
- Check git configuration with `git config --list`
- Verify git is installed with `git --version`

## Related Skills

- `git-diff`: Get detailed diff information
- `git-commit`: Commit changes to git
- `git-push`: Push changes to remote repository
```

## Automation Script

Create a script to automate the conversion process:

```python
#!/usr/bin/env python3
"""
Automated script to convert documentation to OpenHands skills.
"""

import os
from pathlib import Path
import re

def create_skill_from_docs(docs_path, output_dir, skill_name, triggers):
    """Convert a documentation file to a skill."""
    docs_path = Path(docs_path)
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    # Read documentation
    with open(docs_path, 'r') as f:
        content = f.read()

    # Extract title and description
    title_match = re.search(r'^# (.+)$', content, re.MULTILINE)
    title = title_match.group(1) if title_match else "Unknown"

    # Extract URL
    url_match = re.search(r'^URL: (.+)$', content, re.MULTILINE)
    url = url_match.group(1) if url_match else ""

    # Extract description
    desc_match = re.search(r'## Description\n\n(.+?)(?=\n\n|$)', content, re.DOTALL)
    description = desc_match.group(1).strip() if desc_match else ""

    # Create skill frontmatter
    frontmatter = f"""---
name: {skill_name.lower().replace(' ', '-')}
type: knowledge
version: 1.0.0
agent: CodeActAgent
triggers:
{''.join(f'  - {trigger}\n' for trigger in triggers)}
---

# {title} Skill

## Official Documentation
URL: {url}

## Description
{description}
"""

    # Add the original content after the frontmatter
    skill_content = frontmatter + "\n\n" + content

    # Write skill file
    output_file = output_dir / f"{skill_name.lower().replace(' ', '-')}.md"
    with open(output_file, 'w') as f:
        f.write(skill_content)

    return output_file

def main():
    # Example usage
    doc_file = "openhands_book/docs/git-changes.md"
    output_dir = "skills/openhands-docs"
    skill_name = "Git Changes"
    triggers = ["git changes", "git status", "check git modifications"]

    skill_file = create_skill_from_docs(doc_file, output_dir, skill_name, triggers)
    print(f"Created skill: {skill_file}")

if __name__ == "__main__":
    main()
```

## Testing Skills

After creating skills, test them:

1. **Manual testing**: Add skills to your `.openhands/skills/` directory and test with OpenHands
2. **Keyword testing**: Use the trigger words in conversations to activate skills
3. **Content verification**: Ensure skills provide relevant, actionable information

## Best Practices for Skill Creation

1. **Choose effective triggers**: Use words users would naturally say
2. **Provide actionable information**: Skills should help solve problems or provide guidance
3. **Include examples**: Show how to use the skill's knowledge
4. **Add troubleshooting**: Help users resolve common issues
5. **Keep skills focused**: Each skill should cover one specific topic
6. **Use clear language**: Write instructions that are easy to follow
7. **Include related skills**: Help users find complementary knowledge

## Advanced: Creating Skill Categories

Organize skills into categories for better management:

```
skills/
├── openhands-docs/
│   ├── api-reference/
│   │   ├── get-microagents.md
│   │   ├── git-changes.md
│   │   └── ...
│   ├── guides/
│   │   ├── installation.md
│   │   ├── configuration.md
│   │   └── ...
│   └── troubleshooting/
│       ├── common-errors.md
│       ├── debugging.md
│       └── ...
└── README.md
```

## Deployment

1. **For personal use**: Add skills to your repository's `.openhands/skills/` directory
2. **For community sharing**: Submit skills to the OpenHands repository via pull request
3. **For organization-wide**: Add skills to your organization's OpenHands configuration

## Maintenance

- Keep skills updated when documentation changes
- Add new skills for new features
- Improve existing skills based on user feedback
- Remove deprecated skills

## Resources

- [OpenHands Skills Documentation](https://docs.openhands.dev/overview/skills)
- [Keyword-triggered Skills](https://docs.openhands.dev/overview/skills/keyword)
- [Repository Skills](https://docs.openhands.dev/overview/skills/repo)
- [Official Skills Registry](https://github.com/OpenHands/skills)

By following this guide, you can systematically convert the OpenHands documentation book into valuable, actionable skills that enhance the capabilities of the OpenHands AI agent.