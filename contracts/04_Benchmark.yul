object "Benchmark One" {
  code {
    
    let c := 10

    let n := 0x3 

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
                    zero := not(zero)
                }
                if or(zero, 0) {
                    sum2 := 1
                }
            }
        }
     case 3 {
            let x := 100 // Find sqrt of x
            // Start off with z at 1.
            let z := 1

            // Used below to help find a nearby power of 2.
            let y := x

            // Find the lowest power of 2 that is at least sqrt(x).
            if iszero(lt(y, 0x100000000000000000000000000000000)) {
                y := shr(128, y) // Like dividing by 2 ** 128.
                z := shl(64, z)
            }
            if iszero(lt(y, 0x10000000000000000)) {
                y := shr(64, y) // Like dividing by 2 ** 64.
                z := shl(32, z)
            }
            if iszero(lt(y, 0x100000000)) {
                y := shr(32, y) // Like dividing by 2 ** 32.
                z := shl(16, z)
            }
            if iszero(lt(y, 0x10000)) {
                y := shr(16, y) // Like dividing by 2 ** 16.
                z := shl(8, z)
            }
            if iszero(lt(y, 0x100)) {
                y := shr(8, y) // Like dividing by 2 ** 8.
                z := shl(4, z)
            }
            if iszero(lt(y, 0x10)) {
                y := shr(4, y) // Like dividing by 2 ** 4.
                z := shl(2, z)
            }
            if iszero(lt(y, 0x8)) {
                // Equivalent to 2 ** z.
                z := shl(1, z)
            }

            // Shifting right by 1 is like dividing by 2.
            z := shr(1, add(z, div(x, z)))
            z := shr(1, add(z, div(x, z)))
            z := shr(1, add(z, div(x, z)))
            z := shr(1, add(z, div(x, z)))
            z := shr(1, add(z, div(x, z)))
            z := shr(1, add(z, div(x, z)))
            z := shr(1, add(z, div(x, z)))

            // Compute a rounded down version of z.
            let zRoundDown := div(x, z)

            // If zRoundDown is smaller, use it.
            if lt(zRoundDown, z) {
                z := zRoundDown
            }
      }
    }
  }
}
