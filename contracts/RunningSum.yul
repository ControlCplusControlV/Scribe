object "runningSum" {
  code {
            let sum := 10

            for { let i := 0 } lt(i, 100) { i := add(i, 1)}
            {
                sum := add(sum,i)
            }
            sum
  }
}