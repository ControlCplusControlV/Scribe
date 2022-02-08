object "fibonnaci" {
  code {
    let n := 10
    let a := 0
    let b := 1
    let c := 0

    for { let i := 0 } lt(i, n) { i := add(i, 1)}
    {
        c := add(a,b)
        a := b
        b := c
    }
    b
  }
}
