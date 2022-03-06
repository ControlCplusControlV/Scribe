object "Benchmark One" {
  code {
    
    let c := 10

    let n := 3

    let sum := 0

    let sum1 := 0
    let sum2 := 0
    let sum3 := 0

    for { let i := 0 } lt(i, n) { i := add(i, 1)} {

        switch i

        case 1 {
            // Arbitrary math tests
            let arbitrary_uint1 := 100
            let arbitrary_uint2 := 100

            let sub_test := 1

            sum := add(arbitrary_uint1, arbitrary_uint2)
            sum := sub(sum, sub_test)
            if eq(sum, 199) {
                sum1 := 1
            }
        }

        case 2 {
            // Arbitrary Boolean Operations

            let true_test := 1
            
            // To make sure a 1 won't break
            // boolean comparisons with uint256
            if true_test {
                let zero := 0
                if iszero(zero) {
                    zero := add(1, zero)
                }
                if or(zero, 0) {
                    sum2 := 1
                }
            }
        }
        case 3 {
            let x := 100
            let y := 100
            let z := 100

            mstore(200, x)
            mstore(201, y)
            mstore(202, z)

            let a := mload(200)
            let b := mload(201)
            let c := mload(202)

            let f := add(a,b)
            let finalResult := add(f,c)
        }
    }
  }
}
