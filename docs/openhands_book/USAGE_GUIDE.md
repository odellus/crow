# OpenHands Documentation Book - Usage Guide

This guide provides comprehensive instructions on how to use the OpenHands Documentation Book and convert its contents into OpenHands Skills.

## 📚 Table of Contents

1. [Understanding the Book Structure](#-understanding-the-book-structure)
2. [Reading the Documentation](#-reading-the-documentation)
3. [Converting to OpenHands Skills](#-converting-to-openhands-skills)
4. [Using the Conversion Script](#-using-the-conversion-script)
5. [Manual Skill Creation](#-manual-skill-creation)
6. [Testing and Validation](#-testing-and-validation)
7. [Deployment Options](#-deployment-options)
8. [Maintenance and Updates](#-maintenance-and-updates)
9. [Advanced Usage](#-advanced-usage)
10. [Troubleshooting](#-troubleshooting)

## 📁 Understanding the Book Structure

The OpenHands Documentation Book is organized hierarchically:

```
openhands_book/
├── README.md                # Main documentation
├── index.md                 # Entry point
├── generate_book.sh         # Regeneration script
├── docs/                    # Main documentation section
│   ├── index.md             # Section index
│   ├── api-reference/       # API endpoints
│   ├── usage-guides/        # How-to guides
│   ├── sdk-reference/       # SDK documentation
│   └── ... (more categories)
└── optional/                # Additional resources
    ├── index.md
    └── ...
```

### Key Files:

- **`index.md`**: Main entry point with overview and navigation
- **`docs/index.md`**: List of all documentation entries with links
- **Individual `.md` files**: Detailed documentation for each topic
- **`generate_book.sh`**: Script to regenerate the book from source

## 📖 Reading the Documentation

### Navigation Flow:

1. **Start at the top**: Open `openhands_book/index.md`
2. **Browse sections**: Follow links to different documentation areas
3. **Read individual entries**: Each `.md` file contains:
   - Title and description
   - Original URL reference
   - Detailed content from llms.txt
   - Source attribution

### Search Tips:

- Use your IDE's search functionality (Ctrl+Shift+F / Cmd+Shift+F)
- Search for specific keywords like "API", "configuration", or "troubleshooting"
- Look for related entries in section indexes

### Content Structure:

Each documentation entry follows this format:

```markdown
# Title

URL: https://docs.openhands.dev/...

## Description

Detailed explanation of the feature, API, or concept.

## Additional Information

Related details, examples, or usage notes.

---
This document is part of the OpenHands documentation book...
```

## 🔄 Converting to OpenHands Skills

OpenHands Skills are specialized prompts that enhance the OpenHands AI agent with domain-specific knowledge.

### Why Convert Documentation to Skills?

- **Enhanced capabilities**: Turn static documentation into active expertise
- **Context-aware guidance**: Skills activate based on conversation context
- **Automation**: Skills can provide proactive suggestions and best practices
- **Knowledge sharing**: Share expertise across teams and projects
- **Productivity**: Reduce time spent searching documentation

### Skill Types:

1. **Knowledge Skills**: Shareable expertise (stored in `OpenHands/skills/`)
2. **Repository Skills**: Project-specific guidance (stored in `.openhands/skills/`)

## 💻 Using the Conversion Script

The `convert_to_skills.py` script automates the conversion process.

### Basic Usage:

```bash
# Convert all documentation to skills
python openhands_book/convert_to_skills.py \
    --input openhands_book/docs \
    --output skills/openhands-docs
```

### Advanced Options:

```bash
# Convert specific directory (non-recursive)
python openhands_book/convert_to_skills.py \
    --input openhands_book/docs/api-reference \
    --output skills/openhands-api \
    --no-recursive

# Convert and include subcategories
python openhands_book/convert_to_skills.py \
    --input openhands_book/docs \
    --output skills/openhands-complete
```

### Script Features:

- **Automatic trigger generation**: Creates appropriate triggers based on content
- **Intelligent naming**: Converts titles to proper skill names
- **Content enhancement**: Adds skill-specific sections
- **Organizational preservation**: Maintains original directory structure
- **README generation**: Creates comprehensive index of converted skills

## ✏️ Manual Skill Creation

For more control, create skills manually using the documentation as reference.

### Step-by-Step Process:

1. **Choose a documentation entry** to convert
2. **Create a new skill file** in the appropriate directory
3. **Add YAML frontmatter**:

```markdown
---
name: skill-name
type: knowledge
version: 1.0.0
agent: CodeActAgent
triggers:
- trigger1
- trigger2
---
```

4. **Copy documentation content** and enhance it:
   - Add usage examples
   - Include best practices
   - Provide troubleshooting tips
   - Add related skills references

5. **Save and test** the skill

### Example Conversion:

**Original Documentation** (`docs/get-microagents.md`):
```markdown
# Get Microagents

URL: https://docs.openhands.dev/api-reference/get-microagents.md

## Description

This endpoint returns all repository and knowledge microagents...
```

**Converted Skill** (`skills/get-microagents.md`):
```markdown
---
name: get-microagents
type: knowledge
version: 1.0.0
agent: CodeActAgent
triggers:
- get microagents
- list microagents
- repository agents
- knowledge agents
---

# Get Microagents Skill

## Official Documentation
URL: https://docs.openhands.dev/api-reference/get-microagents.md

## Description

This endpoint returns all repository and knowledge microagents...

## Usage Examples

```bash
# Get all microagents
get_microagents()

# Get microagents for specific repository
get_microagents(repository_url="https://github.com/...")
```

## Best Practices

1. Use microagents to understand available knowledge skills
2. Check microagent content for repository-specific guidelines
3. Review knowledge agents for relevant expertise

## Troubleshooting

- If no microagents are returned, ensure you're in a repository
- Check repository configuration for microagent setup
- Verify proper authentication

## Related Skills
- **repository-configuration**: Configure repository settings
- **knowledge-management**: Manage knowledge agents
- **workspace-setup**: Setup workspace environment
```

## 🧪 Testing and Validation

### Testing Individual Skills:

```bash
# Add skill to your test repository
cp skills/get-microagents.md .openhands/skills/

# Test with OpenHands by mentioning trigger words
# "Show me the microagents available"
# "How do I get repository agents?"
```

### Validation Checklist:

1. **Triggers work**: Skill activates with expected keywords
2. **Content is relevant**: Skill provides useful information
3. **Examples work**: Code snippets and commands are correct
4. **Best practices are actionable**: Guidance is practical
5. **Related skills are useful**: Connections make sense

### Debugging Skills:

- Check OpenHands logs for skill activation messages
- Verify YAML frontmatter syntax
- Test triggers in isolation
- Ensure content is markdown-compatible

## 🚀 Deployment Options

### Option 1: Personal Use

1. Copy skills to your repository:
   ```bash
   cp -r skills/openhands-docs .openhands/skills/
   ```
2. Skills will activate automatically when working with the repository

### Option 2: Team/Organization

1. Add skills to organization's OpenHands configuration
2. Store in shared location accessible to all team members
3. Document skill availability for the team

### Option 3: Community Contribution

1. Test skills thoroughly
2. Improve documentation and examples
3. Submit pull request to OpenHands repository
4. Follow community contribution guidelines

## 🔄 Maintenance and Updates

### Keeping Skills Current:

1. **Monitor documentation changes**: Watch for updates to llms.txt
2. **Regenerate book regularly**:
   ```bash
   cd openhands_book && ./generate_book.sh
   ```
3. **Reconvert skills**:
   ```bash
   python openhands_book/convert_to_skills.py \
       --input openhands_book/docs \
       --output skills/openhands-docs
   ```
4. **Merge manual improvements**: Preserve manual enhancements

### Update Workflow:

```bash
# 1. Get latest documentation
git pull origin main

# 2. Regenerate book
./openhands_book/generate_book.sh

# 3. Reconvert skills
python openhands_book/convert_to_skills.py \
    --input openhands_book/docs \
    --output skills/openhands-docs

# 4. Test new skills
cd test-repo && openhands chat

# 5. Commit changes
git add skills/openhands-docs/
git commit -m "Update OpenHands skills from latest documentation"
```

## 🎯 Advanced Usage

### Custom Skill Categories:

Organize skills into logical categories:

```bash
# Create categorized structure
mkdir -p skills/openhands-docs/api-reference
mkdir -p skills/openhands-docs/usage-guides
mkdir -p skills/openhands-docs/troubleshooting

# Convert specific categories
python openhands_book/convert_to_skills.py \
    --input openhands_book/docs/api-reference \
    --output skills/openhands-docs/api-reference

python openhands_book/convert_to_skills.py \
    --input openhands_book/docs/usage-guides \
    --output skills/openhands-docs/usage-guides
```

### Skill Metadata Enhancement:

Add custom metadata to skills:

```markdown
---
name: advanced-git-operations
type: knowledge
version: 1.0.0
agent: CodeActAgent
triggers:
- git advanced
- complex git
- git expert
metadata:
  difficulty: advanced
  categories:
    - version-control
    - collaboration
  requires:
    - git-knowledge
  provides:
    - git-expertise
---
```

### Combining Multiple Documentation Entries:

Create comprehensive skills by combining related entries:

```python
# Create a meta-skill that combines multiple documentation entries
def create_combined_skill(input_files, output_file):
    combined_content = []

    for input_file in input_files:
        with open(input_file, 'r') as f:
            content = f.read()

        # Extract title and content
        title = extract_title(content)
        content = extract_content_after_frontmatter(content)

        combined_content.append(f"## {title}\n\n{content}\n")

    # Create combined skill
    skill_content = f"""---
    name: combined-skill
    type: knowledge
    version: 1.0.0
    agent: CodeActAgent
    triggers:
    - combined
    - comprehensive
    - all about
    ---

    # Comprehensive Guide Skill

    {''.join(combined_content)}
    """

    with open(output_file, 'w') as f:
        f.write(skill_content)
```

## ❓ Troubleshooting

### Common Issues and Solutions:

#### **Issue 1: Skills not activating**
- **Cause**: Incorrect triggers or skill not in proper directory
- **Solution**:
  ```bash
  # Verify skill location
  ls -la .openhands/skills/

  # Check triggers in skill file
  grep "triggers:" skills/my-skill.md
  ```

#### **Issue 2: Documentation not converting properly**
- **Cause**: Malformed markdown in source file
- **Solution**:
  ```bash
  # Check file for markdown errors
  python -c "import markdown; markdown.markdown(open('docs/trouble.md').read())"

  # Manually fix problematic files before conversion
  ```

#### **Issue 3: Triggers not working as expected**
- **Cause**: Triggers too specific or not matching user language
- **Solution**: Edit skill file to add more natural triggers:
  ```markdown
  triggers:
  - get microagents
  - list microagents
  - show available microagents
  - what microagents do i have
  ```

#### **Issue 4: Content not displaying correctly**
- **Cause**: Markdown rendering issues
- **Solution**:
  ```bash
  # Validate markdown
  cat skills/problem-skill.md | pandoc -t html -o /tmp/test.html

  # Check for unescaped special characters
  grep -E "[*_[]{}" skills/problem-skill.md
  ```

### Getting Help:

1. **Check logs**: OpenHands provides detailed logs about skill activation
2. **Test isolated**: Create a test repository with just one skill
3. **Consult documentation**: Review OpenHands Skills documentation
4. **Community support**: Ask in OpenHands community channels

## 📈 Best Practices

### For Skill Creators:

1. **Focus on actionability**: Skills should help users accomplish tasks
2. **Use clear, concise language**: Avoid unnecessary technical jargon
3. **Include practical examples**: Show real-world usage
4. **Provide troubleshooting**: Help users resolve common issues
5. **Link related knowledge**: Connect to other relevant skills
6. **Test thoroughly**: Verify skills work in real conversations
7. **Keep skills focused**: One skill per specific topic
8. **Use natural language triggers**: Match how users actually speak

### For Repository Maintainers:

1. **Organize skills logically**: Group related skills together
2. **Document skill availability**: Let team members know what's available
3. **Review and update regularly**: Keep skills current
4. **Gather feedback**: Improve based on user experience
5. **Share useful skills**: Contribute valuable skills back to community
6. **Balance automation and control**: Use skills to augment, not replace human expertise

## 🎓 Learning Resources

### Official Documentation:
- [OpenHands Skills Overview](https://docs.openhands.dev/overview/skills)
- [Keyword-Triggered Skills](https://docs.openhands.dev/overview/skills/keyword)
- [Repository Skills](https://docs.openhands.dev/overview/skills/repo)
- [Skills Development Guide](https://docs.openhands.dev/sdk/guides/skill)

### Example Skills Repository:
- [OpenHands Official Skills](https://github.com/OpenHands/OpenHands/tree/main/skills)
- [Skills Registry](https://github.com/OpenHands/skills)

### Tutorials and Guides:
- [Creating Your First Skill](https://docs.openhands.dev/guides/skills-first)
- [Skill Best Practices](https://docs.openhands.dev/guides/skills-best-practices)
- [Advanced Skill Features](https://docs.openhands.dev/guides/skills-advanced)

---

This comprehensive guide provides everything you need to effectively use the OpenHands Documentation Book and convert its contents into powerful OpenHands Skills. By following these instructions, you can transform static documentation into active, intelligent assistance that enhances your development workflow.