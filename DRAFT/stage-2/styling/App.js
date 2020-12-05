import {
  Component,
  Element,
  listen,
  setText,
  space,
  Style,
  StyleId,
  Text
} from '../../helper.js'

export default class App extends Component {
  constructor() {
    super()

    // create style ids
    const sid = StyleId() // todo(stage-3): get ssr id

    // initiate state
    let n = 0

    // create nodes
    let p = Element('p', { class: sid })
    let t = Text('current count is ', p)
    let t2 = Text(n, p)
    let s = space()
    let button = Element('button', { class: sid })
    let t3 = Text('-', button)
    let s2 = space()
    let button2 = Element('button', { class: sid })
    let t4 = Text('+', button2)

    let style = new Style(sid, id => `
/* unused h1 */
/*
  h1 {
    font-size: 200%;
  }
*/
p.${id} {
  color: ${Math.abs(n) >= 10 ? 'red' : 'green'}    
}
button.${id} {
  display: inline-block;
  width: 24px;
  height: 24px;
  font-weight: bold;
}
`)

    // event handles
    const _1 /* button[0].onClick */ = () => {
      n-- // dirty data: n
    }
    const _2 /* button[1].onClick */ = () => {
      n++ // dirty data: n
    }

    // register nodes
    this.nodes = [style, p, s, button, s2, button2]

    // listen events
    this.disposes = [
      listen(button, 'click', _1, () => {
        setText(t2, n) // <- n
        $style.update() // <- n
      }),
      listen(button2, 'click', _2, () => {
        setText(t2, n) // <- n
        $style.update() // <- n
      })
    ]
  }
}
