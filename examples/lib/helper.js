/* Alef Component Helpers */

/** Alef component wrapper. */
export class Component {
  nodes = []
  slots = []
  effects = []
  disposes = []
  listeners = new Map()
  mounted = false
  constructor(props = {}) {
    this.props = props
  }
  register(...nodes) {
    this.nodes = nodes
  }
  onMount(...effects) {
    this.effects.push(...effects)
  }
}

export function mount(component, el, refEl) {
  if (!component.mounted) {
    component.mounted = true
    if (refEl) {
      component.nodes.forEach(node => dom.insertNode(node, refEl));
    } else {
      component.nodes.forEach(node => dom.appendNode(el, node));
    }
    component.disposes = component.effects.map(effect => effect.update()).filter(isFunction);
  }
}

export function appendChild(component, slot) {
  component.slots.push(slot);
}

export function update(component, key, value) {
  component.props[key] = value;
  component.listeners.get(key)?.forEach(callback => callback());
}

export function unmount(component) {
  if (component.mounted) {
    component.mounted = false;
    component.disposes.forEach(dispose => dispose());
    component.disposes = [];
    component.nodes.forEach(node => dom.removeNode(node));
  }
}

export function listen(component, key, ...callbacks) {
  let a = component.listeners.get(key);
  if (!a) {
    a = [];
    component.listeners.set(key, a);
  }
  a.push(...callbacks);
}

/** Create and return a new component. */
export function New(Component, props, parent) {
  const component = new Component(props)
  if (parent) {
    parent.appendChild(component)
  }
  return component
}

/** Alef element node. */
export class AlefElement {
  nodes = []
  events = []
  disposes = []
  activated = false
  isClosed = false
  constructor(name, props, parent) {
    this.name = name
    this.el = document.createElement(name)
    if (isComponent(props) || props instanceof AlefElement || props instanceof AlefFragment) {
      props.appendChild(this)
    } else if (typeof props === 'object' || props !== null) {
      for (const key in props) {
        this.update(key, props[key])
      }
    }
    if (parent) {
      parent.appendChild(this)
    }
  }
  appendChild(child) {
    if (!this.isClosed) {
      this.nodes.push(child)
    } else {
      throw new Error(this.name + ' is closed.')
    }
  }
  activate() {
    if (!this.activated) {
      this.activated = true
      this.events.forEach(({ name, callback, updates }) => {
        const cb = e => {
          // todo: push to asynchronous update queue
          if (callback(e) !== false) {
            updates.forEach(update => update())
          }
        }
        this.el.addEventListener(name, cb)
        this.disposes.push(() => this.el.removeEventListener(name, cb))
      })
      this.nodes.forEach(node => dom.appendNode(this.el, node))
    }
  }
  deactivate() {
    if (this.activated) {
      this.activated = false
      this.disposes.forEach(dispose => dispose())
      this.disposes = []
      this.nodes.forEach(node => dom.removeNode(node))
    }
  }
  update(key, value) {
    const { el } = this
    if (isFunction(value)) {
      value = value()
    }
    const nullValue = value === undefined || value === null
    switch (key) {
      case 'class':
      case 'className':
        if (nullValue) {
          el.className = ''
          el.removeAttribute('class')
        } else {
          el.className = String(value)
        }
        break
      case 'style':
        if (typeof value === 'string') {
          el.setAttribute('style', value)
        } else if (typeof value === 'object' && value !== null && !Array.isArray(value)) {
          Object.keys(value).forEach(key => {
            let styleValue = value[key]
            if (typeof styleValue === 'number' && !Number.isNaN(styleValue)) {
              let unit = 'px'
              switch (key) {
                case 'opacity':
                case 'lineHeight':
                  unit = ''
                  break
              }
              styleValue = styleValue.toFixed(6).replace(/\.0+$/, '') + unit
            } else if (typeof styleValue === 'string') {
              styleValue = styleValue.trim()
            } else {
              return
            }
            el.style[key.replace(/([a-z])([A-Z])/g, '$1-$2').toLowerCase()] = styleValue
          })
        }
        break
      case 'value':
        if (nullValue) {
          el.value = ''
        } else {
          const val = String(value)
          if (el.value.length != val.length && el.value != val) {
            el.value = val
          }
        }
        break
      case 'disabled':
      case 'checked':
        el[key] = !!value && value !== 'false'
        break
      default:
        if (nullValue) {
          el.removeAttribute(key)
        } else {
          el.setAttribute(key, String(value))
        }
        break
    }
  }
  listen(name, callback, ...updates) {
    this.events.push({
      name,
      callback,
      updates,
    })
  }
}

/** Create and return a new element node. */
export function Element(name, props, parent) {
  return new AlefElement(name, props, parent)
}

/** Alef fragment node. */
export class AlefFragment {
  nodes = []
  placeholder = document.createTextNode('')
  constructor(parent) {
    if (parent) {
      parent.appendChild(this)
    }
  }
  appendChild(child) {
    this.nodes.push(child)
  }
}

/** Create and return a new fragment. */
export function Fragment(parent) {
  return new AlefFragment(parent)
}

/** If block to handle conditional rendering. */
export class IfBlock {
  placeholder = document.createTextNode('')
  block = null
  isTrue = false
  constructor(validate, init, keepAlive, parent) {
    this.validate = validate
    this.init = init
    this.keepAlive = keepAlive
    if (parent) {
      parent.appendChild(this)
    }
  }
  update() {
    if (this.validate()) {
      this.truify()
    } else {
      this.falsify()
    }
  }
  truify() {
    if (!this.isTrue) {
      this.isTrue = true
      if (this.block === null) {
        this.block = this.init()
      }
      dom.insertNode(this.block.node, this.placeholder)
    }
  }
  falsify() {
    if (this.isTrue) {
      this.isTrue = false
      const block = this.block
      if (block !== null) {
        if (!this.keepAlive) {
          this.block = null
        }
        block.dispose && block.dispose()
        dom.removeNode(block.node)
      }
    }
  }
}

/** Create and return a new if block. */
export function If(validate, init, keepAlive, parent) {
  return new IfBlock(validate, init, keepAlive, parent)
}

/** If-Else block to handle conditional rendering. */
export class IfElseBlock {
  currentStep = null
  constructor(steps, keepAlive, parent) {
    this.steps = steps.map(([validate, init]) => If(validate, init, keepAlive, parent))
  }
  current() {
    return this.currentStep.block
  }
  update() {
    let hasTrue = false
    this.steps.forEach(step => {
      if (hasTrue) {
        step.falsify()
      } else {
        step.update()
        hasTrue = step.isTrue
        if (hasTrue) {
          this.currentStep = step
        }
      }
    })
  }
}

/** Create and return a new if-else block. */
export function IfElse(validate, parent) {
  return new IfElseBlock(validate, parent)
}

/** List block for map rendering. */
export class ListBlock {
  blocks = []
  placeholder = document.createTextNode('')
  constructor(items, init, parent) {
    this.items = items
    this.init = init
    if (parent) {
      parent.appendChild(this)
    }
  }
  update() {
    const items = Array.isArray(this.items) ? this.items : this.items()
    const newBlocks = []
    if (Array.isArray(items)) {
      items.forEach((item, index) => {
        const { create, key: jsxKey } = this.init(item)
        const key = computeLiKey(index, jsxKey)
        const prev = this.blocks.find(n => n.item === item || n.key === key)
        if (prev) {
          if (prev.item !== item) {
            prev.item = item
            prev.update(true, item)
          } else {
            prev.update()
          }
          prev.index = index
          prev.key = key
          newBlocks.push(prev)
        } else {
          newBlocks.push({ ...create(), item, index, key, })
        }
      })
    }
    const { parentNode } = this.placeholder
    if (parentNode) {
      const indexs = []
      this.blocks.forEach((node) => {
        if (newBlocks.length === 0 || newBlocks.findIndex(newNode => newNode === node) === -1) {
          // remove non-existent blocks
          dom.removeNode(node.node)
        } else {
          indexs.push([indexs.length, node.index])
        }
      })
      newBlocks.forEach((newNode) => {
        if (this.blocks.length === 0 || this.blocks.findIndex(node => newNode === node) === -1) {
          // append new blocks
          dom.insertNode(newNode.node, this.placeholder)
          indexs.push([indexs.length, newNode.index])
        }
      })
      // fix order
      const { childNodes } = parentNode
      const moving = []
      let placeholderIndex = 0
      for (let i = 0; i < childNodes.length; i++) {
        if (childNodes.item(i) === this.placeholder) {
          placeholderIndex = i
          break
        }
      }
      const domStartIndex = placeholderIndex - newBlocks.length
      indexs.forEach(([domIndex, treeIndex]) => {
        if (domIndex !== treeIndex) {
          const node = childNodes.item(domStartIndex + domIndex)
          moving.push([node, domStartIndex + treeIndex])
        }
      })
      moving.forEach(([node, treeIndex]) => {
        if (node !== childNodes.item(treeIndex)) {
          parentNode.insertBefore(node, childNodes.item(treeIndex + 1))
        }
      })
    }
    this.blocks = newBlocks
  }
  unmount() {
    this.blocks.forEach(({ node, dispose }) => {
      dispose && dispose()
      dom.removeNode(node)
    })
    this.blocks = []
  }
}

/** Create and return a new list block. */
export function List(items, init, parent) {
  return new ListBlock(items, init, parent)
}

/** Alef text node to display text. */
export class AlefText {
  constructor(text, parent) {
    this.node = document.createTextNode(text)
    if (parent) {
      parent.appendChild(this)
    }
  }
  update(text) {
    if (isFunction(text)) {
      text = text()
    }
    this.node.textContent = text
  }
}

/** Create and return a new text node. */
export function Text(text, parent) {
  return new AlefText(text, parent)
}

/** A shortcut for `new Text(' ')`. */
export function Space(parent) {
  return new AlefText(' ', parent)
}

/** Alef style node to apply style. */
export class AlefStyle {
  el = document.createElement('style')
  constructor(id, templateFn) {
    this.id = id
    this.templateFn = templateFn
    this.el.setAttribute('id', 'alef-' + id)
  }
  update() {
    this.el.innerHTML = this.templateFn(this.id)
  }
}

/** Create and return a new style node. */
export function Style(templateFn) {
  const id = StyleId() // todo(stage-3): get ssr id
  return new AlefStyle(id, templateFn)
}

/** Create and return a memo. */
export function Memo(update) {
  return {
    value: update(),
    update() {
      return this.value = update()
    }
  }
}

/** Create and return a effect. */
export function Effect(update) {
  return {
    update
  }
}

/** Banch update helper. */
export function banchUpdate(...updates) {
  updates.forEach((updater) => {
    if (Array.isArray(updater) && updater.length > 0) {
      updater.shift().update(...updater)
    } else if (typeof updater === 'object' && updater !== null && isFunction(updater.update)) {
      updater.update()
    }
  })
}

/* Returns a empty callback */
export function nope() { }

const dom = {
  /** Append the node to DOM */
  appendNode(parentNode, node) {
    if (isComponent(node)) {
      node.mount(parentNode)
    } else if (node instanceof AlefFragment) {
      appendEl(parentNode, node.placeholder)
      node.nodes.forEach(childNode => dom.insertNode(childNode, node.placeholder))
    } else if (node instanceof AlefElement) {
      appendEl(parentNode, node.el)
      node.activate()
    } else if (node instanceof IfBlock) {
      appendEl(parentNode, node.placeholder)
      node.update()
    } else if (node instanceof IfElseBlock) {
      node.steps.forEach(step => appendEl(parentNode, step.placeholder))
      node.update()
    } else if (node instanceof ListBlock) {
      appendEl(parentNode, node.placeholder)
      node.update()
    } else if (node instanceof AlefText) {
      appendEl(parentNode, node.node)
    } else if (node instanceof AlefStyle) {
      appendEl(document.head, node.el)
      node.update()
    }
  },

  /** insert the node before given refEl. */
  insertNode(node, refEl) {
    if (isComponent(node)) {
      node.mount(undefined, refEl)
    } else if (node instanceof AlefFragment) {
      insertEl(node.placeholder, refEl)
      node.nodes.forEach(childNode => dom.insertNode(childNode, node.placeholder))
    } else if (node instanceof AlefElement) {
      insertEl(node.el, refEl)
      node.activate()
    } else if (node instanceof IfBlock) {
      insertEl(node.placeholder, refEl)
      node.update()
    } else if (node instanceof IfElseBlock) {
      node.steps.forEach(step => insertEl(step.placeholder, refEl))
      node.update()
    } else if (node instanceof ListBlock) {
      insertEl(node.placeholder, refEl)
      node.update()
    } else if (node instanceof AlefText) {
      insertEl(node.node, refEl)
    } else if (node instanceof AlefStyle) {
      appendEl(document.head, node.el)
      node.update()
    }
  },

  /** Remove the node from its parent. */
  removeNode(node) {
    if (isComponent(node)) {
      node.unmount()
    } else if (node instanceof AlefFragment) {
      node.nodes.forEach(node => dom.removeNode(node))
      removeEl(node.placeholder)
    } else if (node instanceof AlefElement) {
      node.deactivate()
      removeEl(node.el)
    } else if (node instanceof IfBlock) {
      node.falsify()
      removeEl(node.placeholder)
    } else if (node instanceof IfElseBlock) {
      node.steps.forEach(step => step.falsify())
      node.steps.forEach(step => appendEl(step.placeholder))
    } else if (node instanceof ListBlock) {
      node.unmount()
      removeEl(node.placeholder)
    } else if (node instanceof AlefStyle) {
      removeEl(node.el)
    } else if (node instanceof AlefText) {
      removeEl(node.node)
    }
  }
}

/** Append an element to parent element. */
function appendEl(parent, el) {
  parent.appendChild(el)
}

/** Insert an element to befor ref element. */
function insertEl(el, refEl) {
  const { parentNode } = refEl
  if (parentNode) {
    parentNode.insertBefore(el, refEl)
  }
}

/** Remove the element from DOM. */
function removeEl(el) {
  const { parentNode } = el
  if (parentNode) {
    parentNode.removeChild(el)
  }
}

/** Check object whether is a component. */
function isComponent(obj) {
  if (!(typeof obj === 'object' && obj !== null)) {
    return false
  }
  return obj instanceof Component || obj.__proto__ instanceof Component
}

/** Check object whether is a function. */
function isFunction(obj) {
  return typeof obj === 'function'
}

/**
 * Compute list item key, use index instead of the key
 * when it is not string or number
 */
function computeLiKey(index, key) {
  if (typeof key === 'string' && key.length > 0) {
    return '$' + key
  } else if (typeof key === 'number' && !Number.isNaN(key)) {
    return '%' + key.toString(16)
  } else {
    return '#' + index.toString(16)
  }
}

const idTable = '1234567890abcdefghijklmnopqrstuvwxyz'
const idLen = 6

/** Create and return a style unique ID. */
export function StyleId() {
  let id = idTable.slice(10).charAt(Math.floor(26 * Math.random())) // starts with a-z
  for (let i = 0; i < idLen - 1; i++) {
    id += idTable.charAt(Math.floor(36 * Math.random()))
  }
  return id
}
