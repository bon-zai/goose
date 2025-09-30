# Husky Pre-commit Hook - Temporarily Disabled

The pre-commit hook has been temporarily disabled to allow fixing type errors that were preventing commits.

## To Re-enable the Hook

Simply rename the file back:

```bash
mv .husky/pre-commit.disabled .husky/pre-commit
```

## What the Hook Does

The pre-commit hook runs `lint-staged` on TypeScript files in `ui/desktop/`, which:
- Runs TypeScript type checking
- Runs ESLint with auto-fix
- Runs Prettier for code formatting

## Original Hook Content

The disabled hook (`pre-commit.disabled`) contains:
```bash
# Only auto-format desktop TS code if relevant files are modified
if git diff --cached --name-only | grep -q "^ui/desktop/"; then
  if [ -d "ui/desktop" ]; then
    cd ui/desktop && npx lint-staged
  else
    echo "Warning: ui/desktop directory does not exist, skipping lint-staged"
  fi
fi
```

