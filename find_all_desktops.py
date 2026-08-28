import os
import glob

search_paths = [
    os.path.expanduser("~/.local/share/applications"),
    "/usr/share/applications"
]

print("Searching for desktop files...")
found_files = []
for base_path in search_paths:
    if os.path.exists(base_path):
        for filename in os.listdir(base_path):
            if filename.endswith(".desktop"):
                full_path = os.path.join(base_path, filename)
                match = False
                # Case insensitive check on filename
                if "rika" in filename.lower() or "tauri" in filename.lower():
                    match = True
                else:
                    # Check file contents
                    try:
                        with open(full_path, "r", errors="ignore") as f:
                            content = f.read().lower()
                            if "rika" in content or "tauri" in content:
                                match = True
                    except Exception:
                        pass
                
                if match:
                    print(f"Found match: {full_path}")
                    found_files.append(full_path)
                    try:
                        with open(full_path, "r") as f:
                            lines = f.readlines()
                            # Print first few lines and Exec/Icon lines
                            print("--- Info ---")
                            for line in lines:
                                if any(x in line for x in ["Name=", "Exec=", "Icon=", "Type="]):
                                    print(f"  {line.strip()}")
                            print("------------")
                    except Exception as e:
                        print(f"  Error reading: {e}")

if not found_files:
    print("No matching desktop files found.")
