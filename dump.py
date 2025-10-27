import os
import pyperclip

context = ""
for (current_dir, dirs, files) in os.walk('src'):
    for file in files:
        if file.endswith('.rs'):
            full_path = os.path.join(current_dir, file)
            context += f"```rust\n"
            context += f"// {full_path}\n\n"
            content = open(full_path, 'r').read()
            context += content
            context += f"```\n\n"

pyperclip.copy(context)

