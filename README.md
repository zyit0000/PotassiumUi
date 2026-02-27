# Potassium UI

## Features

- **Monaco editor** (Lua), tabs, rename/close tabs
- **Attach / Detach** to ports + “Attach to Any Available”
- **Execute** to selected port or all ports (configurable)
- **Auto-attach** option
- **Built-in `functions.txt` IntelliSense** (completion + hover docs)
- **Outline sidebar** (optional): shows symbols in the current file
- **Settings persistence** via `localStorage`
- **Autosave tabs** (optional): restores your tabs/content on next launch
- **Themes**
  - Built-in: Default Dark + Light
  - Optional **Custom Themes** (import/add) with full CSS variable overrides
  - Optional Monaco theme override via theme JSON

## Running (dev)

From the `potassium-ui` folder:

```bash
npm install
npm run tauri -- dev
```

## Custom Themes (how it works)

Custom themes are stored locally in `localStorage` and can be:

- **Imported** from a JSON file
- **Added** from the current editor tab (JSON)

Themes can override **any** of the UI CSS variables (colors, accents, borders, dropdown colors, shadows, etc.).
If you include a `monacoTheme` object, it will also theme the Monaco editor (using Monaco `defineTheme`).
