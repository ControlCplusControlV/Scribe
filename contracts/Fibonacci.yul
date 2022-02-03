object "fib" {
  code {
    let f := 1
    let s := 1
    let next
    for { let i := 0 } lt(i, 10) { i := add(i, 1)}
    {
      if lt(i, 2) {
        mstore(i, 1)
      }
      if gt(i, 1) {
        next := add(s, f)
        f := s
        s := next
        mstore(i, s)
      }
    }
  }
}
