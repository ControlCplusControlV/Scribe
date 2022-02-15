use primitive_types::U256;
use scribe::test_utilities::run_example;

#[ignore]
#[test]
fn u256_add() {
    run_example(
        "
            let x:u256 := 28948022309329048855892746252171976963317496166410141009864396001978282409984
            let y:u256 := 21711016731996786641919559689128982722488122124807605757398297001483711807488
            u256add(x, y)
        ",
        vec![
        // TODO: unclear how we want to handle checking the output, probably some function that
        // takes the stack and transforms the top 8 values into a U256, below is the real expected
        // value
        // "50659039041325835497812305941300959685805618291217746767262693003461994217472".parse::<U256>().unwrap()
        0,
        ],
    );
}

#[ignore]
#[test]
fn u256_less_than() {
    run_example(
        "
            let x:u256 := 28948022309329048855892746252171976963317496166410141009864396001978282409984
            let y:u256 := 21711016731996786641919559689128982722488122124807605757398297001483711807488
            let foo:u32 := 0
            if lt(y, x) {
                foo = 1
            }
            foo
        ",
        vec![1],
    );
}
