![Alef Component Draft](./assets/banner.svg)

# Alef Component Draft

Alef Component for Modern Web Apps, it's inspired by **React** and **Svelte** and based on **TSX**. Core features include:

- Born in **Typescript**
- With Standard **JSX** Syntax
- **AOT** Compile in Rust
- Zero Runtime
- No Virtual DOM
- Reactive
- Builtin Styling
- Support **SSR**

## Stages
This draft is parted in three stages, currently accept any new feature and improvement. After the draft is locked, the **AOT** comilper in Rust will be implemented to make it works in nodejs, Deno and browsers.

- Stage I ([RFC](https://github.com/alephjs/alef-component-draft/issues/3))
  - **Nodes Rendering** - render nodes using native DOM
  - **Conditional Rendering** - ...if...else...
  - **Loop Rendering** - render list
  - **Events** - handle events to update view
  - **Memo** - use computed states
  - **Side Effect** - react for state changing
- Stage II ([RFC](https://github.com/alephjs/alef-component-draft/issues/4))
  - **Import Alef Component** - `import Logo from "./Logo.alef"`
  - **Slots** - `<Logo><img ... /></Logo>`
  - **Reuse Pattern** - reuse common logics
  - **Context** - share state in child component tree
  - **Styling** - inline CSS with scope
- Stage III ([RFC](https://github.com/alephjs/alef-component-draft/issues/5))
  - **Asynchronous Component** - wait for data fetching
  - **Error Boundary** - catch errors in child component tree
  - **SSR** - server side rendering 

## Run Draft

```bash
git clone https://github.com/alephjs/alef-component-draft
cd alef-component-draft

npx serve
```

## Status
Drafting the draft.
