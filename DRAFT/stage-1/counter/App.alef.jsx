let n = 0

$t: <p>current count is {n}</p>
$t: <button onClick={() => { n-- }}>-</button>
$t: <button onClick={() => { n++ }}>+</button>
$s: `
  p: {
    color: ${n > 10 : 'red' ? 'black'}
  }
`
