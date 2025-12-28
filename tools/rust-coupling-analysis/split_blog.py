#!/usr/bin/env python3
"""
Blog Markdown Splitter

Automatically splits blog.md into sections and places them in the contents/ directory.
"""

import os
import re
from pathlib import Path
from typing import List, Tuple


def extract_sections(content: str) -> List[Tuple[str, str, str]]:
    """
    Extract sections from markdown content.

    Args:
        content: Full markdown content

    Returns:
        List of (section_number, section_title, section_content) tuples
    """
    sections = []
    lines = content.split('\n')

    current_section = None
    current_title = None
    current_content = []
    section_number = 0

    for line in lines:
        # Check if line is a level 2 heading (##)
        if line.startswith('## '):
            # Save previous section if exists
            if current_section is not None:
                sections.append((
                    f"{section_number:02d}",
                    current_section,
                    '\n'.join(current_content)
                ))

            # Start new section
            section_number += 1
            current_section = line[3:].strip()
            current_title = line
            current_content = [line]
        else:
            if current_content is not None:
                current_content.append(line)

    # Save last section
    if current_section is not None:
        sections.append((
            f"{section_number:02d}",
            current_section,
            '\n'.join(current_content)
        ))

    return sections


def sanitize_filename(title: str) -> str:
    """
    Convert section title to safe filename.

    Args:
        title: Section title

    Returns:
        Sanitized filename
    """
    # Remove special characters and convert to lowercase
    # Keep only alphanumeric, spaces, and some safe chars
    safe = re.sub(r'[^\w\s\-()]', '', title)
    # Replace spaces with hyphens
    safe = re.sub(r'\s+', '-', safe)
    # Remove multiple consecutive hyphens
    safe = re.sub(r'-+', '-', safe)
    # Convert to lowercase
    safe = safe.lower()
    return safe


def create_contents_directory(base_path: Path) -> Path:
    """
    Create contents directory if it doesn't exist.

    Args:
        base_path: Base project path

    Returns:
        Path to contents directory
    """
    contents_dir = base_path / 'contents'
    contents_dir.mkdir(exist_ok=True)
    return contents_dir


def write_section_files(
    sections: List[Tuple[str, str, str]],
    contents_dir: Path
) -> List[Path]:
    """
    Write section content to individual files.

    Args:
        sections: List of section tuples
        contents_dir: Directory to write files to

    Returns:
        List of created file paths
    """
    created_files = []

    for number, title, content in sections:
        filename = f"{number}-{sanitize_filename(title)}.md"
        filepath = contents_dir / filename

        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(content)

        created_files.append(filepath)
        print(f"Created: {filepath}")

    return created_files


def create_index(
    sections: List[Tuple[str, str, str]],
    contents_dir: Path
) -> Path:
    """
    Create index file with links to all sections.

    Args:
        sections: List of section tuples
        contents_dir: Directory containing section files

    Returns:
        Path to index file
    """
    index_path = contents_dir / 'README.md'

    with open(index_path, 'w', encoding='utf-8') as f:
        f.write('# Blog Contents Index\n\n')
        f.write('Automatically generated from blog.md\n\n')
        f.write('## Sections\n\n')

        for number, title, _ in sections:
            filename = f"{number}-{sanitize_filename(title)}.md"
            f.write(f"{number}. [{title}](./{filename})\n")

    print(f"Created index: {index_path}")
    return index_path


def main():
    """Main entry point."""
    # Get script directory
    script_dir = Path(__file__).parent.resolve()
    blog_path = script_dir / 'blog.md'

    # Check if blog.md exists
    if not blog_path.exists():
        print(f"Error: {blog_path} not found")
        return 1

    print(f"Reading {blog_path}...")
    with open(blog_path, 'r', encoding='utf-8') as f:
        content = f.read()

    # Extract sections
    print("Extracting sections...")
    sections = extract_sections(content)
    print(f"Found {len(sections)} sections")

    # Create contents directory
    contents_dir = create_contents_directory(script_dir)
    print(f"Contents directory: {contents_dir}")

    # Write section files
    print("\nWriting section files...")
    created_files = write_section_files(sections, contents_dir)

    # Create index
    print("\nCreating index...")
    create_index(sections, contents_dir)

    print(f"\n✅ Successfully split blog.md into {len(created_files)} files")
    return 0


if __name__ == '__main__':
    exit(main())
