# Renaming the Repository

Follow these steps to rename the repository from `claude-mcp-manager` to `claude-code-tool-manager`.

## 1. Rename Local Folder

Close all editors and terminals that are using this folder, then run:

```powershell
# Navigate to parent directory
cd C:\Code

# Rename the folder
Rename-Item -Path "claude-mcp-manager" -NewName "claude-code-tool-manager"

# Enter the renamed folder
cd claude-code-tool-manager
```

## 2. Rename GitHub Repository (if applicable)

1. Go to your repository on GitHub
2. Click **Settings** (gear icon)
3. Under **General**, find **Repository name**
4. Change `claude-mcp-manager` to `claude-code-tool-manager`
5. Click **Rename**

## 3. Update Local Git Remote

After renaming on GitHub, update your local remote URL:

```powershell
# Update the remote URL (replace YOUR_USERNAME with your GitHub username)
git remote set-url origin https://github.com/YOUR_USERNAME/claude-code-tool-manager.git

# Verify the change
git remote -v
```

## 4. Verify Everything Works

```powershell
# Run tests
npm test

# Start the app
npm run tauri dev
```

---

You can delete this file after completing the rename.
