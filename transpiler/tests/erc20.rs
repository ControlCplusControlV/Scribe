#[cfg(test)]
mod tests {

    use scribe::parser::parse_yul_syntax;
    use scribe::types::expressions_to_tree;

    #[ignore] // hex literal issues for now
    #[test]
    fn parse_erc20() {
        insta::assert_snapshot!(expressions_to_tree(&parse_yul_syntax(
            r###" object "ERC20_587" {
    code {
        /// @src 1:1393:12620  "contract ERC20 is Context, IERC20, IERC20Metadata {..."
        mstore(64, memoryguard(128))
        if callvalue() { revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() }

        let _1, _2 := copy_arguments_for_constructor_44_object_ERC20_587()
        constructor_ERC20_587(_1, _2)

        let _3 := allocate_unbounded()
        codecopy(_3, dataoffset("ERC20_587_deployed"), datasize("ERC20_587_deployed"))

        return(_3, datasize("ERC20_587_deployed"))

        function allocate_unbounded() -> memPtr {
            memPtr := mload(64)
        }

        function revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() {
            revert(0, 0)
        }

        function round_up_to_mul_of_32(value) -> result {
            result := and(add(value, 31), not(31))
        }

        function panic_error_0x41() {
            mstore(0, 35408467139433450592217433187231851964531694900788300625387963629091585785856)
            mstore(4, 0x41)
            revert(0, 0x24)
        }

        function finalize_allocation(memPtr, size) {
            let newFreePtr := add(memPtr, round_up_to_mul_of_32(size))
            // protect against overflow
            if or(gt(newFreePtr, 0xffffffffffffffff), lt(newFreePtr, memPtr)) { panic_error_0x41() }
            mstore(64, newFreePtr)
        }

        function allocate_memory(size) -> memPtr {
            memPtr := allocate_unbounded()
            finalize_allocation(memPtr, size)
        }

        function revert_error_dbdddcbe895c83990c08b3492a0e83918d802a52331272ac6fdb6a7c4aea3b1b() {
            revert(0, 0)
        }

        function revert_error_c1322bf8034eace5e0b5c7295db60986aa89aae5e0ea0873e4689e076861a5db() {
            revert(0, 0)
        }

        function revert_error_1b9f4a0a5773e33b91aa01db23bf8c55fce1411167c872835e7fa00a4f17d46d() {
            revert(0, 0)
        }

        function revert_error_987264b3b1d58a9c7f8255e93e81c77d86d6299019c33110a076957a3e06e2ae() {
            revert(0, 0)
        }

        function array_allocation_size_t_string_memory_ptr(length) -> size {
            // Make sure we can allocate memory without overflow
            if gt(length, 0xffffffffffffffff) { panic_error_0x41() }

            size := round_up_to_mul_of_32(length)

            // add length slot
            size := add(size, 0x20)

        }

        function copy_memory_to_memory(src, dst, length) {
            let i := 0
            for { } lt(i, length) { i := add(i, 32) }
            {
                mstore(add(dst, i), mload(add(src, i)))
            }
            if gt(i, length)
            {
                // clear end
                mstore(add(dst, length), 0)
            }
        }

        function abi_decode_available_length_t_string_memory_ptr_fromMemory(src, length, end) -> array {
            array := allocate_memory(array_allocation_size_t_string_memory_ptr(length))
            mstore(array, length)
            let dst := add(array, 0x20)
            if gt(add(src, length), end) { revert_error_987264b3b1d58a9c7f8255e93e81c77d86d6299019c33110a076957a3e06e2ae() }
            copy_memory_to_memory(src, dst, length)
        }

        // string
        function abi_decode_t_string_memory_ptr_fromMemory(offset, end) -> array {
            if iszero(slt(add(offset, 0x1f), end)) { revert_error_1b9f4a0a5773e33b91aa01db23bf8c55fce1411167c872835e7fa00a4f17d46d() }
            let length := mload(offset)
            array := abi_decode_available_length_t_string_memory_ptr_fromMemory(add(offset, 0x20), length, end)
        }

        function abi_decode_tuple_t_string_memory_ptrt_string_memory_ptr_fromMemory(headStart, dataEnd) -> value0, value1 {
            if slt(sub(dataEnd, headStart), 64) { revert_error_dbdddcbe895c83990c08b3492a0e83918d802a52331272ac6fdb6a7c4aea3b1b() }

            {

                let offset := mload(add(headStart, 0))
                if gt(offset, 0xffffffffffffffff) { revert_error_c1322bf8034eace5e0b5c7295db60986aa89aae5e0ea0873e4689e076861a5db() }

                value0 := abi_decode_t_string_memory_ptr_fromMemory(add(headStart, offset), dataEnd)
            }

            {

                let offset := mload(add(headStart, 32))
                if gt(offset, 0xffffffffffffffff) { revert_error_c1322bf8034eace5e0b5c7295db60986aa89aae5e0ea0873e4689e076861a5db() }

                value1 := abi_decode_t_string_memory_ptr_fromMemory(add(headStart, offset), dataEnd)
            }

        }

        function copy_arguments_for_constructor_44_object_ERC20_587() -> ret_param_0, ret_param_1 {
            let programSize := datasize("ERC20_587")
            let argSize := sub(codesize(), programSize)

            let memoryDataOffset := allocate_memory(argSize)
            codecopy(memoryDataOffset, programSize, argSize)

            ret_param_0, ret_param_1 := abi_decode_tuple_t_string_memory_ptrt_string_memory_ptr_fromMemory(memoryDataOffset, add(memoryDataOffset, argSize))
        }

        function panic_error_0x00() {
            mstore(0, 35408467139433450592217433187231851964531694900788300625387963629091585785856)
            mstore(4, 0x00)
            revert(0, 0x24)
        }

        function array_length_t_string_memory_ptr(value) -> length {

            length := mload(value)

        }

        function panic_error_0x22() {
            mstore(0, 35408467139433450592217433187231851964531694900788300625387963629091585785856)
            mstore(4, 0x22)
            revert(0, 0x24)
        }

        function extract_byte_array_length(data) -> length {
            length := div(data, 2)
            let outOfPlaceEncoding := and(data, 1)
            if iszero(outOfPlaceEncoding) {
                length := and(length, 0x7f)
            }

            if eq(outOfPlaceEncoding, lt(length, 32)) {
                panic_error_0x22()
            }
        }

        function array_dataslot_t_string_storage(ptr) -> data {
            data := ptr

            mstore(0, ptr)
            data := keccak256(0, 0x20)

        }

        function divide_by_32_ceil(value) -> result {
            result := div(add(value, 31), 32)
        }

        function shift_left_dynamic(bits, value) -> newValue {
            newValue :=

            shl(bits, value)

        }

        function update_byte_slice_dynamic32(value, shiftBytes, toInsert) -> result {
            let shiftBits := mul(shiftBytes, 8)
            let mask := shift_left_dynamic(shiftBits, 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff)
            toInsert := shift_left_dynamic(shiftBits, toInsert)
            value := and(value, not(mask))
            result := or(value, and(toInsert, mask))
        }

        function cleanup_t_uint256(value) -> cleaned {
            cleaned := value
        }

        function identity(value) -> ret {
            ret := value
        }

        function convert_t_uint256_to_t_uint256(value) -> converted {
            converted := cleanup_t_uint256(identity(cleanup_t_uint256(value)))
        }

        function prepare_store_t_uint256(value) -> ret {
            ret := value
        }

        function update_storage_value_t_uint256_to_t_uint256(slot, offset, value_0) {
            let convertedValue_0 := convert_t_uint256_to_t_uint256(value_0)
            sstore(slot, update_byte_slice_dynamic32(sload(slot), offset, prepare_store_t_uint256(convertedValue_0)))
        }

        function zero_value_for_split_t_uint256() -> ret {
            ret := 0
        }

        function storage_set_to_zero_t_uint256(slot, offset) {
            let zero_0 := zero_value_for_split_t_uint256()
            update_storage_value_t_uint256_to_t_uint256(slot, offset, zero_0)
        }

        function clear_storage_range_t_bytes1(start, end) {
            for {} lt(start, end) { start := add(start, 1) }
            {
                storage_set_to_zero_t_uint256(start, 0)
            }
        }

        function clean_up_bytearray_end_slots_t_string_storage(array, len, startIndex) {

            if gt(len, 31) {
                let dataArea := array_dataslot_t_string_storage(array)
                let deleteStart := add(dataArea, divide_by_32_ceil(startIndex))
                // If we are clearing array to be short byte array, we want to clear only data starting from array data area.
                if lt(startIndex, 32) { deleteStart := dataArea }
                clear_storage_range_t_bytes1(deleteStart, add(dataArea, divide_by_32_ceil(len)))
            }

        }

        function shift_right_unsigned_dynamic(bits, value) -> newValue {
            newValue :=

            shr(bits, value)

        }

        function mask_bytes_dynamic(data, bytes) -> result {
            let mask := not(shift_right_unsigned_dynamic(mul(8, bytes), not(0)))
            result := and(data, mask)
        }
        function extract_used_part_and_set_length_of_short_byte_array(data, len) -> used {
            // we want to save only elements that are part of the array after resizing
            // others should be set to zero
            data := mask_bytes_dynamic(data, len)
            used := or(data, mul(2, len))
        }
        function copy_byte_array_to_storage_from_t_string_memory_ptr_to_t_string_storage(slot, src) {

            let newLen := array_length_t_string_memory_ptr(src)
            // Make sure array length is sane
            if gt(newLen, 0xffffffffffffffff) { panic_error_0x41() }

            let oldLen := extract_byte_array_length(sload(slot))

            // potentially truncate data
            clean_up_bytearray_end_slots_t_string_storage(slot, oldLen, newLen)

            let srcOffset := 0

            srcOffset := 0x20

            switch gt(newLen, 31)
            case 1 {
                let loopEnd := and(newLen, not(0x1f))

                let dstPtr := array_dataslot_t_string_storage(slot)
                let i := 0
                for { } lt(i, loopEnd) { i := add(i, 0x20) } {
                    sstore(dstPtr, mload(add(src, srcOffset)))
                    dstPtr := add(dstPtr, 1)
                    srcOffset := add(srcOffset, 32)
                }
                if lt(loopEnd, newLen) {
                    let lastValue := mload(add(src, srcOffset))
                    sstore(dstPtr, mask_bytes_dynamic(lastValue, and(newLen, 0x1f)))
                }
                sstore(slot, add(mul(newLen, 2), 1))
            }
            default {
                let value := 0
                if newLen {
                    value := mload(add(src, srcOffset))
                }
                sstore(slot, extract_used_part_and_set_length_of_short_byte_array(value, newLen))
            }
        }

        function update_storage_value_offset_0t_string_memory_ptr_to_t_string_storage(slot, value_0) {

            copy_byte_array_to_storage_from_t_string_memory_ptr_to_t_string_storage(slot, value_0)
        }

        /// @ast-id 44
        /// @src 1:1968:2081  "constructor(string memory name_, string memory symbol_) {..."
        function constructor_ERC20_587(var_name__30_mpos, var_symbol__32_mpos) {

            /// @src 1:1968:2081  "constructor(string memory name_, string memory symbol_) {..."
            constructor_IERC20Metadata_712()

            /// @src 1:2042:2047  "name_"
            let _4_mpos := var_name__30_mpos
            let expr_36_mpos := _4_mpos
            /// @src 1:2034:2047  "_name = name_"
            update_storage_value_offset_0t_string_memory_ptr_to_t_string_storage(0x03, expr_36_mpos)
            let _5_slot := 0x03
            let expr_37_slot := _5_slot
            /// @src 1:2067:2074  "symbol_"
            let _6_mpos := var_symbol__32_mpos
            let expr_40_mpos := _6_mpos
            /// @src 1:2057:2074  "_symbol = symbol_"
            update_storage_value_offset_0t_string_memory_ptr_to_t_string_storage(0x04, expr_40_mpos)
            let _7_slot := 0x04
            let expr_41_slot := _7_slot

        }
        /// @src 1:1393:12620  "contract ERC20 is Context, IERC20, IERC20Metadata {..."

        /// @src 3:277:682  "interface IERC20Metadata is IERC20 {..."
        function constructor_IERC20Metadata_712() {

            /// @src 3:277:682  "interface IERC20Metadata is IERC20 {..."
            constructor_IERC20_687()

        }
        /// @src 1:1393:12620  "contract ERC20 is Context, IERC20, IERC20Metadata {..."

        /// @src 2:202:2766  "interface IERC20 {..."
        function constructor_IERC20_687() {

            /// @src 2:202:2766  "interface IERC20 {..."
            constructor_Context_609()

        }
        /// @src 1:1393:12620  "contract ERC20 is Context, IERC20, IERC20Metadata {..."

        /// @src 0:608:843  "abstract contract Context {..."
        function constructor_Context_609() {

            /// @src 0:608:843  "abstract contract Context {..."

        }
        /// @src 1:1393:12620  "contract ERC20 is Context, IERC20, IERC20Metadata {..."

    }
    /// @use-src 0:"Context.sol", 1:"ERC20.sol"
    object "ERC20_587_deployed" {
        code {
            /// @src 1:1393:12620  "contract ERC20 is Context, IERC20, IERC20Metadata {..."
            mstore(64, memoryguard(128))

            if iszero(lt(calldatasize(), 4))
            {
                let selector := shift_right_224_unsigned(calldataload(0))
                switch selector

                case 0x06fdde03
                {
                    // name()

                    if callvalue() { revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() }
                    abi_decode_tuple_(4, calldatasize())
                    let ret_0 :=  fun_name_54()
                    let memPos := allocate_unbounded()
                    let memEnd := abi_encode_tuple_t_string_memory_ptr__to_t_string_memory_ptr__fromStack(memPos , ret_0)
                    return(memPos, sub(memEnd, memPos))
                }

                case 0x095ea7b3
                {
                    // approve(address,uint256)

                    if callvalue() { revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() }
                    let param_0, param_1 :=  abi_decode_tuple_t_addresst_uint256(4, calldatasize())
                    let ret_0 :=  fun_approve_166(param_0, param_1)
                    let memPos := allocate_unbounded()
                    let memEnd := abi_encode_tuple_t_bool__to_t_bool__fromStack(memPos , ret_0)
                    return(memPos, sub(memEnd, memPos))
                }

                case 0x18160ddd
                {
                    // totalSupply()

                    if callvalue() { revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() }
                    abi_decode_tuple_(4, calldatasize())
                    let ret_0 :=  fun_totalSupply_84()
                    let memPos := allocate_unbounded()
                    let memEnd := abi_encode_tuple_t_uint256__to_t_uint256__fromStack(memPos , ret_0)
                    return(memPos, sub(memEnd, memPos))
                }

                case 0x23b872dd
                {
                    // transferFrom(address,address,uint256)

                    if callvalue() { revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() }
                    let param_0, param_1, param_2 :=  abi_decode_tuple_t_addresst_addresst_uint256(4, calldatasize())
                    let ret_0 :=  fun_transferFrom_199(param_0, param_1, param_2)
                    let memPos := allocate_unbounded()
                    let memEnd := abi_encode_tuple_t_bool__to_t_bool__fromStack(memPos , ret_0)
                    return(memPos, sub(memEnd, memPos))
                }

                case 0x313ce567
                {
                    // decimals()

                    if callvalue() { revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() }
                    abi_decode_tuple_(4, calldatasize())
                    let ret_0 :=  fun_decimals_74()
                    let memPos := allocate_unbounded()
                    let memEnd := abi_encode_tuple_t_uint8__to_t_uint8__fromStack(memPos , ret_0)
                    return(memPos, sub(memEnd, memPos))
                }

                case 0x39509351
                {
                    // increaseAllowance(address,uint256)

                    if callvalue() { revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() }
                    let param_0, param_1 :=  abi_decode_tuple_t_addresst_uint256(4, calldatasize())
                    let ret_0 :=  fun_increaseAllowance_229(param_0, param_1)
                    let memPos := allocate_unbounded()
                    let memEnd := abi_encode_tuple_t_bool__to_t_bool__fromStack(memPos , ret_0)
                    return(memPos, sub(memEnd, memPos))
                }

                case 0x70a08231
                {
                    // balanceOf(address)

                    if callvalue() { revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() }
                    let param_0 :=  abi_decode_tuple_t_address(4, calldatasize())
                    let ret_0 :=  fun_balanceOf_98(param_0)
                    let memPos := allocate_unbounded()
                    let memEnd := abi_encode_tuple_t_uint256__to_t_uint256__fromStack(memPos , ret_0)
                    return(memPos, sub(memEnd, memPos))
                }

                case 0x95d89b41
                {
                    // symbol()

                    if callvalue() { revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() }
                    abi_decode_tuple_(4, calldatasize())
                    let ret_0 :=  fun_symbol_64()
                    let memPos := allocate_unbounded()
                    let memEnd := abi_encode_tuple_t_string_memory_ptr__to_t_string_memory_ptr__fromStack(memPos , ret_0)
                    return(memPos, sub(memEnd, memPos))
                }

                case 0xa457c2d7
                {
                    // decreaseAllowance(address,uint256)

                    if callvalue() { revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() }
                    let param_0, param_1 :=  abi_decode_tuple_t_addresst_uint256(4, calldatasize())
                    let ret_0 :=  fun_decreaseAllowance_271(param_0, param_1)
                    let memPos := allocate_unbounded()
                    let memEnd := abi_encode_tuple_t_bool__to_t_bool__fromStack(memPos , ret_0)
                    return(memPos, sub(memEnd, memPos))
                }

                case 0xa9059cbb
                {
                    // transfer(address,uint256)

                    if callvalue() { revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() }
                    let param_0, param_1 :=  abi_decode_tuple_t_addresst_uint256(4, calldatasize())
                    let ret_0 :=  fun_transfer_123(param_0, param_1)
                    let memPos := allocate_unbounded()
                    let memEnd := abi_encode_tuple_t_bool__to_t_bool__fromStack(memPos , ret_0)
                    return(memPos, sub(memEnd, memPos))
                }

                case 0xdd62ed3e
                {
                    // allowance(address,address)

                    if callvalue() { revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() }
                    let param_0, param_1 :=  abi_decode_tuple_t_addresst_address(4, calldatasize())
                    let ret_0 :=  fun_allowance_141(param_0, param_1)
                    let memPos := allocate_unbounded()
                    let memEnd := abi_encode_tuple_t_uint256__to_t_uint256__fromStack(memPos , ret_0)
                    return(memPos, sub(memEnd, memPos))
                }

                default {}
            }

            revert_error_42b3090547df1d2001c96683413b8cf91c1b902ef5e3cb8d9f6f304cf7446f74()

            function shift_right_224_unsigned(value) -> newValue {
                newValue :=

                shr(224, value)

            }

            function allocate_unbounded() -> memPtr {
                memPtr := mload(64)
            }

            function revert_error_ca66f745a3ce8ff40e2ccaf1ad45db7774001b90d25810abd9040049be7bf4bb() {
                revert(0, 0)
            }

            function revert_error_dbdddcbe895c83990c08b3492a0e83918d802a52331272ac6fdb6a7c4aea3b1b() {
                revert(0, 0)
            }

            function abi_decode_tuple_(headStart, dataEnd)   {
                if slt(sub(dataEnd, headStart), 0) { revert_error_dbdddcbe895c83990c08b3492a0e83918d802a52331272ac6fdb6a7c4aea3b1b() }

            }

            function array_length_t_string_memory_ptr(value) -> length {

                length := mload(value)

            }

            function array_storeLengthForEncoding_t_string_memory_ptr_fromStack(pos, length) -> updated_pos {
                mstore(pos, length)
                updated_pos := add(pos, 0x20)
            }

            function copy_memory_to_memory(src, dst, length) {
                let i := 0
                for { } lt(i, length) { i := add(i, 32) }
                {
                    mstore(add(dst, i), mload(add(src, i)))
                }
                if gt(i, length)
                {
                    // clear end
                    mstore(add(dst, length), 0)
                }
            }

            function round_up_to_mul_of_32(value) -> result {
                result := and(add(value, 31), not(31))
            }

            function abi_encode_t_string_memory_ptr_to_t_string_memory_ptr_fromStack(value, pos) -> end {
                let length := array_length_t_string_memory_ptr(value)
                pos := array_storeLengthForEncoding_t_string_memory_ptr_fromStack(pos, length)
                copy_memory_to_memory(add(value, 0x20), pos, length)
                end := add(pos, round_up_to_mul_of_32(length))
            }

            function abi_encode_tuple_t_string_memory_ptr__to_t_string_memory_ptr__fromStack(headStart , value0) -> tail {
                tail := add(headStart, 32)

                mstore(add(headStart, 0), sub(tail, headStart))
                tail := abi_encode_t_string_memory_ptr_to_t_string_memory_ptr_fromStack(value0,  tail)

            }

            function revert_error_c1322bf8034eace5e0b5c7295db60986aa89aae5e0ea0873e4689e076861a5db() {
                revert(0, 0)
            }

            function cleanup_t_uint160(value) -> cleaned {
                cleaned := and(value, 0xffffffffffffffffffffffffffffffffffffffff)
            }

            function cleanup_t_address(value) -> cleaned {
                cleaned := cleanup_t_uint160(value)
            }

            function validator_revert_t_address(value) {
                if iszero(eq(value, cleanup_t_address(value))) { revert(0, 0) }
            }

            function abi_decode_t_address(offset, end) -> value {
                value := calldataload(offset)
                validator_revert_t_address(value)
            }

            function cleanup_t_uint256(value) -> cleaned {
                cleaned := value
            }

            function validator_revert_t_uint256(value) {
                if iszero(eq(value, cleanup_t_uint256(value))) { revert(0, 0) }
            }

            function abi_decode_t_uint256(offset, end) -> value {
                value := calldataload(offset)
                validator_revert_t_uint256(value)
            }

            function abi_decode_tuple_t_addresst_uint256(headStart, dataEnd) -> value0, value1 {
                if slt(sub(dataEnd, headStart), 64) { revert_error_dbdddcbe895c83990c08b3492a0e83918d802a52331272ac6fdb6a7c4aea3b1b() }

                {

                    let offset := 0

                    value0 := abi_decode_t_address(add(headStart, offset), dataEnd)
                }

                {

                    let offset := 32

                    value1 := abi_decode_t_uint256(add(headStart, offset), dataEnd)
                }

            }

            function cleanup_t_bool(value) -> cleaned {
                cleaned := iszero(iszero(value))
            }

            function abi_encode_t_bool_to_t_bool_fromStack(value, pos) {
                mstore(pos, cleanup_t_bool(value))
            }

            function abi_encode_tuple_t_bool__to_t_bool__fromStack(headStart , value0) -> tail {
                tail := add(headStart, 32)

                abi_encode_t_bool_to_t_bool_fromStack(value0,  add(headStart, 0))

            }

            function abi_encode_t_uint256_to_t_uint256_fromStack(value, pos) {
                mstore(pos, cleanup_t_uint256(value))
            }

            function abi_encode_tuple_t_uint256__to_t_uint256__fromStack(headStart , value0) -> tail {
                tail := add(headStart, 32)

                abi_encode_t_uint256_to_t_uint256_fromStack(value0,  add(headStart, 0))

            }

            function abi_decode_tuple_t_addresst_addresst_uint256(headStart, dataEnd) -> value0, value1, value2 {
                if slt(sub(dataEnd, headStart), 96) { revert_error_dbdddcbe895c83990c08b3492a0e83918d802a52331272ac6fdb6a7c4aea3b1b() }

                {

                    let offset := 0

                    value0 := abi_decode_t_address(add(headStart, offset), dataEnd)
                }

                {

                    let offset := 32

                    value1 := abi_decode_t_address(add(headStart, offset), dataEnd)
                }

                {

                    let offset := 64

                    value2 := abi_decode_t_uint256(add(headStart, offset), dataEnd)
                }

            }

            function cleanup_t_uint8(value) -> cleaned {
                cleaned := and(value, 0xff)
            }

            function abi_encode_t_uint8_to_t_uint8_fromStack(value, pos) {
                mstore(pos, cleanup_t_uint8(value))
            }

            function abi_encode_tuple_t_uint8__to_t_uint8__fromStack(headStart , value0) -> tail {
                tail := add(headStart, 32)

                abi_encode_t_uint8_to_t_uint8_fromStack(value0,  add(headStart, 0))

            }

            function abi_decode_tuple_t_address(headStart, dataEnd) -> value0 {
                if slt(sub(dataEnd, headStart), 32) { revert_error_dbdddcbe895c83990c08b3492a0e83918d802a52331272ac6fdb6a7c4aea3b1b() }

                {

                    let offset := 0

                    value0 := abi_decode_t_address(add(headStart, offset), dataEnd)
                }

            }

            function abi_decode_tuple_t_addresst_address(headStart, dataEnd) -> value0, value1 {
                if slt(sub(dataEnd, headStart), 64) { revert_error_dbdddcbe895c83990c08b3492a0e83918d802a52331272ac6fdb6a7c4aea3b1b() }

                {

                    let offset := 0

                    value0 := abi_decode_t_address(add(headStart, offset), dataEnd)
                }

                {

                    let offset := 32

                    value1 := abi_decode_t_address(add(headStart, offset), dataEnd)
                }

            }

            function revert_error_42b3090547df1d2001c96683413b8cf91c1b902ef5e3cb8d9f6f304cf7446f74() {
                revert(0, 0)
            }

            function zero_value_for_split_t_string_memory_ptr() -> ret {
                ret := 96
            }

            function panic_error_0x22() {
                mstore(0, 35408467139433450592217433187231851964531694900788300625387963629091585785856)
                mstore(4, 0x22)
                revert(0, 0x24)
            }

            function extract_byte_array_length(data) -> length {
                length := div(data, 2)
                let outOfPlaceEncoding := and(data, 1)
                if iszero(outOfPlaceEncoding) {
                    length := and(length, 0x7f)
                }

                if eq(outOfPlaceEncoding, lt(length, 32)) {
                    panic_error_0x22()
                }
            }

            function array_storeLengthForEncoding_t_string_memory_ptr(pos, length) -> updated_pos {
                mstore(pos, length)
                updated_pos := add(pos, 0x20)
            }

            function array_dataslot_t_string_storage(ptr) -> data {
                data := ptr

                mstore(0, ptr)
                data := keccak256(0, 0x20)

            }

            // string -> string
            function abi_encode_t_string_storage_to_t_string_memory_ptr(value, pos) -> ret {
                let slotValue := sload(value)
                let length := extract_byte_array_length(slotValue)
                pos := array_storeLengthForEncoding_t_string_memory_ptr(pos, length)
                switch and(slotValue, 1)
                case 0 {
                    // short byte array
                    mstore(pos, and(slotValue, not(0xff)))
                    ret := add(pos, 0x20)
                }
                case 1 {
                    // long byte array
                    let dataPos := array_dataslot_t_string_storage(value)
                    let i := 0
                    for { } lt(i, length) { i := add(i, 0x20) } {
                        mstore(add(pos, i), sload(dataPos))
                        dataPos := add(dataPos, 1)
                    }
                    ret := add(pos, i)
                }
            }

            function abi_encodeUpdatedPos_t_string_storage_to_t_string_memory_ptr(value0, pos) -> updatedPos {
                updatedPos := abi_encode_t_string_storage_to_t_string_memory_ptr(value0, pos)
            }

            function panic_error_0x41() {
                mstore(0, 35408467139433450592217433187231851964531694900788300625387963629091585785856)
                mstore(4, 0x41)
                revert(0, 0x24)
            }

            function finalize_allocation(memPtr, size) {
                let newFreePtr := add(memPtr, round_up_to_mul_of_32(size))
                // protect against overflow
                if or(gt(newFreePtr, 0xffffffffffffffff), lt(newFreePtr, memPtr)) { panic_error_0x41() }
                mstore(64, newFreePtr)
            }

            function copy_array_from_storage_to_memory_t_string_storage(slot) -> memPtr {
                memPtr := allocate_unbounded()
                let end := abi_encodeUpdatedPos_t_string_storage_to_t_string_memory_ptr(slot, memPtr)
                finalize_allocation(memPtr, sub(end, memPtr))
            }

            function convert_array_t_string_storage_to_t_string_memory_ptr(value) -> converted  {

                // Copy the array to a free position in memory
                converted :=

                copy_array_from_storage_to_memory_t_string_storage(value)

            }

            /// @ast-id 54
            /// @src 1:2146:2244  "function name() public view virtual override returns (string memory) {..."
            function fun_name_54() -> var__49_mpos {
                /// @src 1:2200:2213  "string memory"
                let zero_t_string_memory_ptr_1_mpos := zero_value_for_split_t_string_memory_ptr()
                var__49_mpos := zero_t_string_memory_ptr_1_mpos

                /// @src 1:2232:2237  "_name"
                let _2_slot := 0x03
                let expr_51_slot := _2_slot
                /// @src 1:2225:2237  "return _name"
                var__49_mpos := convert_array_t_string_storage_to_t_string_memory_ptr(expr_51_slot)
                leave

            }
            /// @src 1:1393:12620  "contract ERC20 is Context, IERC20, IERC20Metadata {..."

            /// @ast-id 64
            /// @src 1:2357:2459  "function symbol() public view virtual override returns (string memory) {..."
            function fun_symbol_64() -> var__59_mpos {
                /// @src 1:2413:2426  "string memory"
                let zero_t_string_memory_ptr_3_mpos := zero_value_for_split_t_string_memory_ptr()
                var__59_mpos := zero_t_string_memory_ptr_3_mpos

                /// @src 1:2445:2452  "_symbol"
                let _4_slot := 0x04
                let expr_61_slot := _4_slot
                /// @src 1:2438:2452  "return _symbol"
                var__59_mpos := convert_array_t_string_storage_to_t_string_memory_ptr(expr_61_slot)
                leave

            }
            /// @src 1:1393:12620  "contract ERC20 is Context, IERC20, IERC20Metadata {..."

            function zero_value_for_split_t_uint8() -> ret {
                ret := 0
            }

            function cleanup_t_rational_18_by_1(value) -> cleaned {
                cleaned := value
            }

            function identity(value) -> ret {
                ret := value
            }

            function convert_t_rational_18_by_1_to_t_uint8(value) -> converted {
                converted := cleanup_t_uint8(identity(cleanup_t_rational_18_by_1(value)))
            }

            /// @ast-id 74
            /// @src 1:3083:3174  "function decimals() public view virtual override returns (uint8) {..."
            function fun_decimals_74() -> var__69 {
                /// @src 1:3141:3146  "uint8"
                let zero_t_uint8_5 := zero_value_for_split_t_uint8()
                var__69 := zero_t_uint8_5

                /// @src 1:3165:3167  "18"
                let expr_71 := 0x12
                /// @src 1:3158:3167  "return 18"
                var__69 := convert_t_rational_18_by_1_to_t_uint8(expr_71)
                leave

            }
            /// @src 1:1393:12620  "contract ERC20 is Context, IERC20, IERC20Metadata {..."

            function zero_value_for_split_t_uint256() -> ret {
                ret := 0
            }

            function shift_right_0_unsigned(value) -> newValue {
                newValue :=

                shr(0, value)

            }

            function cleanup_from_storage_t_uint256(value) -> cleaned {
                cleaned := value
            }

            function extract_from_storage_value_offset_0t_uint256(slot_value) -> value {
                value := cleanup_from_storage_t_uint256(shift_right_0_unsigned(slot_value))
            }

            function read_from_storage_split_offset_0_t_uint256(slot) -> value {
                value := extract_from_storage_value_offset_0t_uint256(sload(slot))

            }

            /// @ast-id 84
            /// @src 1:3234:3340  "function totalSupply() public view virtual override returns (uint256) {..."
            function fun_totalSupply_84() -> var__79 {
                /// @src 1:3295:3302  "uint256"
                let zero_t_uint256_6 := zero_value_for_split_t_uint256()
                var__79 := zero_t_uint256_6

                /// @src 1:3321:3333  "_totalSupply"
                let _7 := read_from_storage_split_offset_0_t_uint256(0x02)
                let expr_81 := _7
                /// @src 1:3314:3333  "return _totalSupply"
                var__79 := expr_81
                leave

            }
            /// @src 1:1393:12620  "contract ERC20 is Context, IERC20, IERC20Metadata {..."

            function convert_t_uint160_to_t_uint160(value) -> converted {
                converted := cleanup_t_uint160(identity(cleanup_t_uint160(value)))
            }

            function convert_t_uint160_to_t_address(value) -> converted {
                converted := convert_t_uint160_to_t_uint160(value)
            }

            function convert_t_address_to_t_address(value) -> converted {
                converted := convert_t_uint160_to_t_address(value)
            }

            function mapping_index_access_t_mapping$_t_address_$_t_uint256_$_of_t_address(slot , key) -> dataSlot {
                mstore(0, convert_t_address_to_t_address(key))
                mstore(0x20, slot)
                dataSlot := keccak256(0, 0x40)
            }

            /// @ast-id 98
            /// @src 1:3398:3523  "function balanceOf(address account) public view virtual override returns (uint256) {..."
            function fun_balanceOf_98(var_account_87) -> var__91 {
                /// @src 1:3472:3479  "uint256"
                let zero_t_uint256_8 := zero_value_for_split_t_uint256()
                var__91 := zero_t_uint256_8

                /// @src 1:3498:3507  "_balances"
                let _9 := 0x00
                let expr_93 := _9
                /// @src 1:3508:3515  "account"
                let _10 := var_account_87
                let expr_94 := _10
                /// @src 1:3498:3516  "_balances[account]"
                let _11 := mapping_index_access_t_mapping$_t_address_$_t_uint256_$_of_t_address(expr_93,expr_94)
                let _12 := read_from_storage_split_offset_0_t_uint256(_11)
                let expr_95 := _12
                /// @src 1:3491:3516  "return _balances[account]"
                var__91 := expr_95
                leave

            }
            /// @src 1:1393:12620  "contract ERC20 is Context, IERC20, IERC20Metadata {..."

            function zero_value_for_split_t_bool() -> ret {
                ret := 0
            }

            /// @ast-id 123
            /// @src 1:3719:3908  "function transfer(address to, uint256 amount) public virtual override returns (bool) {..."
            function fun_transfer_123(var_to_101, var_amount_103) -> var__107 {
                /// @src 1:3798:3802  "bool"
                let zero_t_bool_13 := zero_value_for_split_t_bool()
                var__107 := zero_t_bool_13

                /// @src 1:3830:3842  "_msgSender()"
                let expr_112 := fun__msgSender_599()
                /// @src 1:3814:3842  "address owner = _msgSender()"
                let var_owner_110 := expr_112
                /// @src 1:3862:3867  "owner"
                let _14 := var_owner_110
                let expr_115 := _14
                /// @src 1:3869:3871  "to"
                let _15 := var_to_101
                let expr_116 := _15
                /// @src 1:3873:3879  "amount"
                let _16 := var_amount_103
                let expr_117 := _16
                fun__transfer_348(expr_115, expr_116, expr_117)
                /// @src 1:3897:3901  "true"
                let expr_120 := 0x01
                /// @src 1:3890:3901  "return true"
                var__107 := expr_120
                leave

            }
            /// @src 1:1393:12620  "contract ERC20 is Context, IERC20, IERC20Metadata {..."

            function mapping_index_access_t_mapping$_t_address_$_t_mapping$_t_address_$_t_uint256_$_$_of_t_address(slot , key) -> dataSlot {
                mstore(0, convert_t_address_to_t_address(key))
                mstore(0x20, slot)
                dataSlot := keccak256(0, 0x40)
            }

            /// @ast-id 141
            /// @src 1:3966:4115  "function allowance(address owner, address spender) public view virtual override returns (uint256) {..."
            function fun_allowance_141(var_owner_126, var_spender_128) -> var__132 {
                /// @src 1:4055:4062  "uint256"
                let zero_t_uint256_17 := zero_value_for_split_t_uint256()
                var__132 := zero_t_uint256_17

                /// @src 1:4081:4092  "_allowances"
                let _18 := 0x01
                let expr_134 := _18
                /// @src 1:4093:4098  "owner"
                let _19 := var_owner_126
                let expr_135 := _19
                /// @src 1:4081:4099  "_allowances[owner]"
                let _20 := mapping_index_access_t_mapping$_t_address_$_t_mapping$_t_address_$_t_uint256_$_$_of_t_address(expr_134,expr_135)
                let _21 := _20
                let expr_136 := _21
                /// @src 1:4100:4107  "spender"
                let _22 := var_spender_128
                let expr_137 := _22
                /// @src 1:4081:4108  "_allowances[owner][spender]"
                let _23 := mapping_index_access_t_mapping$_t_address_$_t_uint256_$_of_t_address(expr_136,expr_137)
                let _24 := read_from_storage_split_offset_0_t_uint256(_23)
                let expr_138 := _24
                /// @src 1:4074:4108  "return _allowances[owner][spender]"
                var__132 := expr_138
                leave

            }
            /// @src 1:1393:12620  "contract ERC20 is Context, IERC20, IERC20Metadata {..."

            /// @ast-id 166
            /// @src 1:4423:4620  "function approve(address spender, uint256 amount) public virtual override returns (bool) {..."
            function fun_approve_166(var_spender_144, var_amount_146) -> var__150 {
                /// @src 1:4506:4510  "bool"
                let zero_t_bool_25 := zero_value_for_split_t_bool()
                var__150 := zero_t_bool_25

                /// @src 1:4538:4550  "_msgSender()"
                let expr_155 := fun__msgSender_599()
                /// @src 1:4522:4550  "address owner = _msgSender()"
                let var_owner_153 := expr_155
                /// @src 1:4569:4574  "owner"
                let _26 := var_owner_153
                let expr_158 := _26
                /// @src 1:4576:4583  "spender"
                let _27 := var_spender_144
                let expr_159 := _27
                /// @src 1:4585:4591  "amount"
                let _28 := var_amount_146
                let expr_160 := _28
                fun__approve_521(expr_158, expr_159, expr_160)
                /// @src 1:4609:4613  "true"
                let expr_163 := 0x01
                /// @src 1:4602:4613  "return true"
                var__150 := expr_163
                leave

            }
            /// @src 1:1393:12620  "contract ERC20 is Context, IERC20, IERC20Metadata {..."

            /// @ast-id 199
            /// @src 1:5182:5468  "function transferFrom(..."
            function fun_transferFrom_199(var_from_169, var_to_171, var_amount_173) -> var__177 {
                /// @src 1:5309:5313  "bool"
                let zero_t_bool_29 := zero_value_for_split_t_bool()
                var__177 := zero_t_bool_29

                /// @src 1:5343:5355  "_msgSender()"
                let expr_182 := fun__msgSender_599()
                /// @src 1:5325:5355  "address spender = _msgSender()"
                let var_spender_180 := expr_182
                /// @src 1:5381:5385  "from"
                let _30 := var_from_169
                let expr_185 := _30
                /// @src 1:5387:5394  "spender"
                let _31 := var_spender_180
                let expr_186 := _31
                /// @src 1:5396:5402  "amount"
                let _32 := var_amount_173
                let expr_187 := _32
                fun__spendAllowance_564(expr_185, expr_186, expr_187)
                /// @src 1:5423:5427  "from"
                let _33 := var_from_169
                let expr_191 := _33
                /// @src 1:5429:5431  "to"
                let _34 := var_to_171
                let expr_192 := _34
                /// @src 1:5433:5439  "amount"
                let _35 := var_amount_173
                let expr_193 := _35
                fun__transfer_348(expr_191, expr_192, expr_193)
                /// @src 1:5457:5461  "true"
                let expr_196 := 0x01
                /// @src 1:5450:5461  "return true"
                var__177 := expr_196
                leave

            }
            /// @src 1:1393:12620  "contract ERC20 is Context, IERC20, IERC20Metadata {..."

            function panic_error_0x11() {
                mstore(0, 35408467139433450592217433187231851964531694900788300625387963629091585785856)
                mstore(4, 0x11)
                revert(0, 0x24)
            }

            function checked_add_t_uint256(x, y) -> sum {
                x := cleanup_t_uint256(x)
                y := cleanup_t_uint256(y)

                // overflow, if x > (maxValue - y)
                if gt(x, sub(0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff, y)) { panic_error_0x11() }

                sum := add(x, y)
            }

            /// @ast-id 229
            /// @src 1:5863:6099  "function increaseAllowance(address spender, uint256 addedValue) public virtual returns (bool) {..."
            function fun_increaseAllowance_229(var_spender_202, var_addedValue_204) -> var__207 {
                /// @src 1:5951:5955  "bool"
                let zero_t_bool_36 := zero_value_for_split_t_bool()
                var__207 := zero_t_bool_36

                /// @src 1:5983:5995  "_msgSender()"
                let expr_212 := fun__msgSender_599()
                /// @src 1:5967:5995  "address owner = _msgSender()"
                let var_owner_210 := expr_212
                /// @src 1:6014:6019  "owner"
                let _37 := var_owner_210
                let expr_215 := _37
                /// @src 1:6021:6028  "spender"
                let _38 := var_spender_202
                let expr_216 := _38
                /// @src 1:6030:6041  "_allowances"
                let _39 := 0x01
                let expr_217 := _39
                /// @src 1:6042:6047  "owner"
                let _40 := var_owner_210
                let expr_218 := _40
                /// @src 1:6030:6048  "_allowances[owner]"
                let _41 := mapping_index_access_t_mapping$_t_address_$_t_mapping$_t_address_$_t_uint256_$_$_of_t_address(expr_217,expr_218)
                let _42 := _41
                let expr_219 := _42
                /// @src 1:6049:6056  "spender"
                let _43 := var_spender_202
                let expr_220 := _43
                /// @src 1:6030:6057  "_allowances[owner][spender]"
                let _44 := mapping_index_access_t_mapping$_t_address_$_t_uint256_$_of_t_address(expr_219,expr_220)
                let _45 := read_from_storage_split_offset_0_t_uint256(_44)
                let expr_221 := _45
                /// @src 1:6060:6070  "addedValue"
                let _46 := var_addedValue_204
                let expr_222 := _46
                /// @src 1:6030:6070  "_allowances[owner][spender] + addedValue"
                let expr_223 := checked_add_t_uint256(expr_221, expr_222)

                fun__approve_521(expr_215, expr_216, expr_223)
                /// @src 1:6088:6092  "true"
                let expr_226 := 0x01
                /// @src 1:6081:6092  "return true"
                var__207 := expr_226
                leave

            }
            /// @src 1:1393:12620  "contract ERC20 is Context, IERC20, IERC20Metadata {..."

            function store_literal_in_memory_f8b476f7d28209d77d4a4ac1fe36b9f8259aa1bb6bddfa6e89de7e51615cf8a8(memPtr) {

                mstore(add(memPtr, 0), "ERC20: decreased allowance below")

                mstore(add(memPtr, 32), " zero")

            }

            function abi_encode_t_stringliteral_f8b476f7d28209d77d4a4ac1fe36b9f8259aa1bb6bddfa6e89de7e51615cf8a8_to_t_string_memory_ptr_fromStack(pos) -> end {
                pos := array_storeLengthForEncoding_t_string_memory_ptr_fromStack(pos, 37)
                store_literal_in_memory_f8b476f7d28209d77d4a4ac1fe36b9f8259aa1bb6bddfa6e89de7e51615cf8a8(pos)
                end := add(pos, 64)
            }

            function abi_encode_tuple_t_stringliteral_f8b476f7d28209d77d4a4ac1fe36b9f8259aa1bb6bddfa6e89de7e51615cf8a8__to_t_string_memory_ptr__fromStack(headStart ) -> tail {
                tail := add(headStart, 32)

                mstore(add(headStart, 0), sub(tail, headStart))
                tail := abi_encode_t_stringliteral_f8b476f7d28209d77d4a4ac1fe36b9f8259aa1bb6bddfa6e89de7e51615cf8a8_to_t_string_memory_ptr_fromStack( tail)

            }

            function require_helper_t_stringliteral_f8b476f7d28209d77d4a4ac1fe36b9f8259aa1bb6bddfa6e89de7e51615cf8a8(condition ) {
                if iszero(condition) {
                    let memPtr := allocate_unbounded()
                    mstore(memPtr, 0x08c379a000000000000000000000000000000000000000000000000000000000)
                    let end := abi_encode_tuple_t_stringliteral_f8b476f7d28209d77d4a4ac1fe36b9f8259aa1bb6bddfa6e89de7e51615cf8a8__to_t_string_memory_ptr__fromStack(add(memPtr, 4) )
                    revert(memPtr, sub(end, memPtr))
                }
            }

            function wrapping_sub_t_uint256(x, y) -> diff {
                diff := cleanup_t_uint256(sub(x, y))
            }

            /// @ast-id 271
            /// @src 1:6586:7015  "function decreaseAllowance(address spender, uint256 subtractedValue) public virtual returns (bool) {..."
            function fun_decreaseAllowance_271(var_spender_232, var_subtractedValue_234) -> var__237 {
                /// @src 1:6679:6683  "bool"
                let zero_t_bool_47 := zero_value_for_split_t_bool()
                var__237 := zero_t_bool_47

                /// @src 1:6711:6723  "_msgSender()"
                let expr_242 := fun__msgSender_599()
                /// @src 1:6695:6723  "address owner = _msgSender()"
                let var_owner_240 := expr_242
                /// @src 1:6760:6771  "_allowances"
                let _48 := 0x01
                let expr_246 := _48
                /// @src 1:6772:6777  "owner"
                let _49 := var_owner_240
                let expr_247 := _49
                /// @src 1:6760:6778  "_allowances[owner]"
                let _50 := mapping_index_access_t_mapping$_t_address_$_t_mapping$_t_address_$_t_uint256_$_$_of_t_address(expr_246,expr_247)
                let _51 := _50
                let expr_248 := _51
                /// @src 1:6779:6786  "spender"
                let _52 := var_spender_232
                let expr_249 := _52
                /// @src 1:6760:6787  "_allowances[owner][spender]"
                let _53 := mapping_index_access_t_mapping$_t_address_$_t_uint256_$_of_t_address(expr_248,expr_249)
                let _54 := read_from_storage_split_offset_0_t_uint256(_53)
                let expr_250 := _54
                /// @src 1:6733:6787  "uint256 currentAllowance = _allowances[owner][spender]"
                let var_currentAllowance_245 := expr_250
                /// @src 1:6805:6821  "currentAllowance"
                let _55 := var_currentAllowance_245
                let expr_253 := _55
                /// @src 1:6825:6840  "subtractedValue"
                let _56 := var_subtractedValue_234
                let expr_254 := _56
                /// @src 1:6805:6840  "currentAllowance >= subtractedValue"
                let expr_255 := iszero(lt(cleanup_t_uint256(expr_253), cleanup_t_uint256(expr_254)))
                /// @src 1:6797:6882  "require(currentAllowance >= subtractedValue, \"ERC20: decreased allowance below zero\")"
                require_helper_t_stringliteral_f8b476f7d28209d77d4a4ac1fe36b9f8259aa1bb6bddfa6e89de7e51615cf8a8(expr_255)
                /// @src 1:6925:6930  "owner"
                let _57 := var_owner_240
                let expr_260 := _57
                /// @src 1:6932:6939  "spender"
                let _58 := var_spender_232
                let expr_261 := _58
                /// @src 1:6941:6957  "currentAllowance"
                let _59 := var_currentAllowance_245
                let expr_262 := _59
                /// @src 1:6960:6975  "subtractedValue"
                let _60 := var_subtractedValue_234
                let expr_263 := _60
                /// @src 1:6941:6975  "currentAllowance - subtractedValue"
                let expr_264 := wrapping_sub_t_uint256(expr_262, expr_263)

                fun__approve_521(expr_260, expr_261, expr_264)
                /// @src 1:7004:7008  "true"
                let expr_268 := 0x01
                /// @src 1:6997:7008  "return true"
                var__237 := expr_268
                leave

            }
            /// @src 1:1393:12620  "contract ERC20 is Context, IERC20, IERC20Metadata {..."

            function cleanup_t_rational_0_by_1(value) -> cleaned {
                cleaned := value
            }

            function convert_t_rational_0_by_1_to_t_uint160(value) -> converted {
                converted := cleanup_t_uint160(identity(cleanup_t_rational_0_by_1(value)))
            }

            function convert_t_rational_0_by_1_to_t_address(value) -> converted {
                converted := convert_t_rational_0_by_1_to_t_uint160(value)
            }

            function store_literal_in_memory_baecc556b46f4ed0f2b4cb599d60785ac8563dd2dc0a5bf12edea1c39e5e1fea(memPtr) {

                mstore(add(memPtr, 0), "ERC20: transfer from the zero ad")

                mstore(add(memPtr, 32), "dress")

            }

            function abi_encode_t_stringliteral_baecc556b46f4ed0f2b4cb599d60785ac8563dd2dc0a5bf12edea1c39e5e1fea_to_t_string_memory_ptr_fromStack(pos) -> end {
                pos := array_storeLengthForEncoding_t_string_memory_ptr_fromStack(pos, 37)
                store_literal_in_memory_baecc556b46f4ed0f2b4cb599d60785ac8563dd2dc0a5bf12edea1c39e5e1fea(pos)
                end := add(pos, 64)
            }

            function abi_encode_tuple_t_stringliteral_baecc556b46f4ed0f2b4cb599d60785ac8563dd2dc0a5bf12edea1c39e5e1fea__to_t_string_memory_ptr__fromStack(headStart ) -> tail {
                tail := add(headStart, 32)

                mstore(add(headStart, 0), sub(tail, headStart))
                tail := abi_encode_t_stringliteral_baecc556b46f4ed0f2b4cb599d60785ac8563dd2dc0a5bf12edea1c39e5e1fea_to_t_string_memory_ptr_fromStack( tail)

            }

            function require_helper_t_stringliteral_baecc556b46f4ed0f2b4cb599d60785ac8563dd2dc0a5bf12edea1c39e5e1fea(condition ) {
                if iszero(condition) {
                    let memPtr := allocate_unbounded()
                    mstore(memPtr, 0x08c379a000000000000000000000000000000000000000000000000000000000)
                    let end := abi_encode_tuple_t_stringliteral_baecc556b46f4ed0f2b4cb599d60785ac8563dd2dc0a5bf12edea1c39e5e1fea__to_t_string_memory_ptr__fromStack(add(memPtr, 4) )
                    revert(memPtr, sub(end, memPtr))
                }
            }

            function store_literal_in_memory_0557e210f7a69a685100a7e4e3d0a7024c546085cee28910fd17d0b081d9516f(memPtr) {

                mstore(add(memPtr, 0), "ERC20: transfer to the zero addr")

                mstore(add(memPtr, 32), "ess")

            }

            function abi_encode_t_stringliteral_0557e210f7a69a685100a7e4e3d0a7024c546085cee28910fd17d0b081d9516f_to_t_string_memory_ptr_fromStack(pos) -> end {
                pos := array_storeLengthForEncoding_t_string_memory_ptr_fromStack(pos, 35)
                store_literal_in_memory_0557e210f7a69a685100a7e4e3d0a7024c546085cee28910fd17d0b081d9516f(pos)
                end := add(pos, 64)
            }

            function abi_encode_tuple_t_stringliteral_0557e210f7a69a685100a7e4e3d0a7024c546085cee28910fd17d0b081d9516f__to_t_string_memory_ptr__fromStack(headStart ) -> tail {
                tail := add(headStart, 32)

                mstore(add(headStart, 0), sub(tail, headStart))
                tail := abi_encode_t_stringliteral_0557e210f7a69a685100a7e4e3d0a7024c546085cee28910fd17d0b081d9516f_to_t_string_memory_ptr_fromStack( tail)

            }

            function require_helper_t_stringliteral_0557e210f7a69a685100a7e4e3d0a7024c546085cee28910fd17d0b081d9516f(condition ) {
                if iszero(condition) {
                    let memPtr := allocate_unbounded()
                    mstore(memPtr, 0x08c379a000000000000000000000000000000000000000000000000000000000)
                    let end := abi_encode_tuple_t_stringliteral_0557e210f7a69a685100a7e4e3d0a7024c546085cee28910fd17d0b081d9516f__to_t_string_memory_ptr__fromStack(add(memPtr, 4) )
                    revert(memPtr, sub(end, memPtr))
                }
            }

            function store_literal_in_memory_4107e8a8b9e94bf8ff83080ddec1c0bffe897ebc2241b89d44f66b3d274088b6(memPtr) {

                mstore(add(memPtr, 0), "ERC20: transfer amount exceeds b")

                mstore(add(memPtr, 32), "alance")

            }

            function abi_encode_t_stringliteral_4107e8a8b9e94bf8ff83080ddec1c0bffe897ebc2241b89d44f66b3d274088b6_to_t_string_memory_ptr_fromStack(pos) -> end {
                pos := array_storeLengthForEncoding_t_string_memory_ptr_fromStack(pos, 38)
                store_literal_in_memory_4107e8a8b9e94bf8ff83080ddec1c0bffe897ebc2241b89d44f66b3d274088b6(pos)
                end := add(pos, 64)
            }

            function abi_encode_tuple_t_stringliteral_4107e8a8b9e94bf8ff83080ddec1c0bffe897ebc2241b89d44f66b3d274088b6__to_t_string_memory_ptr__fromStack(headStart ) -> tail {
                tail := add(headStart, 32)

                mstore(add(headStart, 0), sub(tail, headStart))
                tail := abi_encode_t_stringliteral_4107e8a8b9e94bf8ff83080ddec1c0bffe897ebc2241b89d44f66b3d274088b6_to_t_string_memory_ptr_fromStack( tail)

            }

            function require_helper_t_stringliteral_4107e8a8b9e94bf8ff83080ddec1c0bffe897ebc2241b89d44f66b3d274088b6(condition ) {
                if iszero(condition) {
                    let memPtr := allocate_unbounded()
                    mstore(memPtr, 0x08c379a000000000000000000000000000000000000000000000000000000000)
                    let end := abi_encode_tuple_t_stringliteral_4107e8a8b9e94bf8ff83080ddec1c0bffe897ebc2241b89d44f66b3d274088b6__to_t_string_memory_ptr__fromStack(add(memPtr, 4) )
                    revert(memPtr, sub(end, memPtr))
                }
            }

            function shift_left_0(value) -> newValue {
                newValue :=

                shl(0, value)

            }

            function update_byte_slice_32_shift_0(value, toInsert) -> result {
                let mask := 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
                toInsert := shift_left_0(toInsert)
                value := and(value, not(mask))
                result := or(value, and(toInsert, mask))
            }

            function convert_t_uint256_to_t_uint256(value) -> converted {
                converted := cleanup_t_uint256(identity(cleanup_t_uint256(value)))
            }

            function prepare_store_t_uint256(value) -> ret {
                ret := value
            }

            function update_storage_value_offset_0t_uint256_to_t_uint256(slot, value_0) {
                let convertedValue_0 := convert_t_uint256_to_t_uint256(value_0)
                sstore(slot, update_byte_slice_32_shift_0(sload(slot), prepare_store_t_uint256(convertedValue_0)))
            }

            /// @ast-id 348
            /// @src 1:7478:8129  "function _transfer(..."
            function fun__transfer_348(var_from_274, var_to_276, var_amount_278) {

                /// @src 1:7604:7608  "from"
                let _61 := var_from_274
                let expr_282 := _61
                /// @src 1:7620:7621  "0"
                let expr_285 := 0x00
                /// @src 1:7612:7622  "address(0)"
                let expr_286 := convert_t_rational_0_by_1_to_t_address(expr_285)
                /// @src 1:7604:7622  "from != address(0)"
                let expr_287 := iszero(eq(cleanup_t_address(expr_282), cleanup_t_address(expr_286)))
                /// @src 1:7596:7664  "require(from != address(0), \"ERC20: transfer from the zero address\")"
                require_helper_t_stringliteral_baecc556b46f4ed0f2b4cb599d60785ac8563dd2dc0a5bf12edea1c39e5e1fea(expr_287)
                /// @src 1:7682:7684  "to"
                let _62 := var_to_276
                let expr_292 := _62
                /// @src 1:7696:7697  "0"
                let expr_295 := 0x00
                /// @src 1:7688:7698  "address(0)"
                let expr_296 := convert_t_rational_0_by_1_to_t_address(expr_295)
                /// @src 1:7682:7698  "to != address(0)"
                let expr_297 := iszero(eq(cleanup_t_address(expr_292), cleanup_t_address(expr_296)))
                /// @src 1:7674:7738  "require(to != address(0), \"ERC20: transfer to the zero address\")"
                require_helper_t_stringliteral_0557e210f7a69a685100a7e4e3d0a7024c546085cee28910fd17d0b081d9516f(expr_297)
                /// @src 1:7770:7774  "from"
                let _63 := var_from_274
                let expr_302 := _63
                /// @src 1:7776:7778  "to"
                let _64 := var_to_276
                let expr_303 := _64
                /// @src 1:7780:7786  "amount"
                let _65 := var_amount_278
                let expr_304 := _65
                fun__beforeTokenTransfer_575(expr_302, expr_303, expr_304)
                /// @src 1:7820:7829  "_balances"
                let _66 := 0x00
                let expr_309 := _66
                /// @src 1:7830:7834  "from"
                let _67 := var_from_274
                let expr_310 := _67
                /// @src 1:7820:7835  "_balances[from]"
                let _68 := mapping_index_access_t_mapping$_t_address_$_t_uint256_$_of_t_address(expr_309,expr_310)
                let _69 := read_from_storage_split_offset_0_t_uint256(_68)
                let expr_311 := _69
                /// @src 1:7798:7835  "uint256 fromBalance = _balances[from]"
                let var_fromBalance_308 := expr_311
                /// @src 1:7853:7864  "fromBalance"
                let _70 := var_fromBalance_308
                let expr_314 := _70
                /// @src 1:7868:7874  "amount"
                let _71 := var_amount_278
                let expr_315 := _71
                /// @src 1:7853:7874  "fromBalance >= amount"
                let expr_316 := iszero(lt(cleanup_t_uint256(expr_314), cleanup_t_uint256(expr_315)))
                /// @src 1:7845:7917  "require(fromBalance >= amount, \"ERC20: transfer amount exceeds balance\")"
                require_helper_t_stringliteral_4107e8a8b9e94bf8ff83080ddec1c0bffe897ebc2241b89d44f66b3d274088b6(expr_316)
                /// @src 1:7969:7980  "fromBalance"
                let _72 := var_fromBalance_308
                let expr_323 := _72
                /// @src 1:7983:7989  "amount"
                let _73 := var_amount_278
                let expr_324 := _73
                /// @src 1:7969:7989  "fromBalance - amount"
                let expr_325 := wrapping_sub_t_uint256(expr_323, expr_324)

                /// @src 1:7951:7960  "_balances"
                let _74 := 0x00
                let expr_320 := _74
                /// @src 1:7961:7965  "from"
                let _75 := var_from_274
                let expr_321 := _75
                /// @src 1:7951:7966  "_balances[from]"
                let _76 := mapping_index_access_t_mapping$_t_address_$_t_uint256_$_of_t_address(expr_320,expr_321)
                /// @src 1:7951:7989  "_balances[from] = fromBalance - amount"
                update_storage_value_offset_0t_uint256_to_t_uint256(_76, expr_325)
                let expr_326 := expr_325
                /// @src 1:8026:8032  "amount"
                let _77 := var_amount_278
                let expr_332 := _77
                /// @src 1:8009:8018  "_balances"
                let _78 := 0x00
                let expr_329 := _78
                /// @src 1:8019:8021  "to"
                let _79 := var_to_276
                let expr_330 := _79
                /// @src 1:8009:8022  "_balances[to]"
                let _80 := mapping_index_access_t_mapping$_t_address_$_t_uint256_$_of_t_address(expr_329,expr_330)
                /// @src 1:8009:8032  "_balances[to] += amount"
                let _81 := read_from_storage_split_offset_0_t_uint256(_80)
                let expr_333 := checked_add_t_uint256(_81, expr_332)

                update_storage_value_offset_0t_uint256_to_t_uint256(_80, expr_333)
                /// @src 1:8057:8061  "from"
                let _82 := var_from_274
                let expr_336 := _82
                /// @src 1:8063:8065  "to"
                let _83 := var_to_276
                let expr_337 := _83
                /// @src 1:8067:8073  "amount"
                let _84 := var_amount_278
                let expr_338 := _84
                /// @src 1:8048:8074  "Transfer(from, to, amount)"
                let _85 := 0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef
                let _86 := convert_t_address_to_t_address(expr_336)
                let _87 := convert_t_address_to_t_address(expr_337)
                {
                    let _88 := allocate_unbounded()
                    let _89 := abi_encode_tuple_t_uint256__to_t_uint256__fromStack(_88 , expr_338)
                    log3(_88, sub(_89, _88) , _85, _86, _87)
                }/// @src 1:8105:8109  "from"
                let _90 := var_from_274
                let expr_342 := _90
                /// @src 1:8111:8113  "to"
                let _91 := var_to_276
                let expr_343 := _91
                /// @src 1:8115:8121  "amount"
                let _92 := var_amount_278
                let expr_344 := _92
                fun__afterTokenTransfer_586(expr_342, expr_343, expr_344)

            }
            /// @src 1:1393:12620  "contract ERC20 is Context, IERC20, IERC20Metadata {..."

            function store_literal_in_memory_c953f4879035ed60e766b34720f656aab5c697b141d924c283124ecedb91c208(memPtr) {

                mstore(add(memPtr, 0), "ERC20: approve from the zero add")

                mstore(add(memPtr, 32), "ress")

            }

            function abi_encode_t_stringliteral_c953f4879035ed60e766b34720f656aab5c697b141d924c283124ecedb91c208_to_t_string_memory_ptr_fromStack(pos) -> end {
                pos := array_storeLengthForEncoding_t_string_memory_ptr_fromStack(pos, 36)
                store_literal_in_memory_c953f4879035ed60e766b34720f656aab5c697b141d924c283124ecedb91c208(pos)
                end := add(pos, 64)
            }

            function abi_encode_tuple_t_stringliteral_c953f4879035ed60e766b34720f656aab5c697b141d924c283124ecedb91c208__to_t_string_memory_ptr__fromStack(headStart ) -> tail {
                tail := add(headStart, 32)

                mstore(add(headStart, 0), sub(tail, headStart))
                tail := abi_encode_t_stringliteral_c953f4879035ed60e766b34720f656aab5c697b141d924c283124ecedb91c208_to_t_string_memory_ptr_fromStack( tail)

            }

            function require_helper_t_stringliteral_c953f4879035ed60e766b34720f656aab5c697b141d924c283124ecedb91c208(condition ) {
                if iszero(condition) {
                    let memPtr := allocate_unbounded()
                    mstore(memPtr, 0x08c379a000000000000000000000000000000000000000000000000000000000)
                    let end := abi_encode_tuple_t_stringliteral_c953f4879035ed60e766b34720f656aab5c697b141d924c283124ecedb91c208__to_t_string_memory_ptr__fromStack(add(memPtr, 4) )
                    revert(memPtr, sub(end, memPtr))
                }
            }

            function store_literal_in_memory_24883cc5fe64ace9d0df1893501ecb93c77180f0ff69cca79affb3c316dc8029(memPtr) {

                mstore(add(memPtr, 0), "ERC20: approve to the zero addre")

                mstore(add(memPtr, 32), "ss")

            }

            function abi_encode_t_stringliteral_24883cc5fe64ace9d0df1893501ecb93c77180f0ff69cca79affb3c316dc8029_to_t_string_memory_ptr_fromStack(pos) -> end {
                pos := array_storeLengthForEncoding_t_string_memory_ptr_fromStack(pos, 34)
                store_literal_in_memory_24883cc5fe64ace9d0df1893501ecb93c77180f0ff69cca79affb3c316dc8029(pos)
                end := add(pos, 64)
            }

            function abi_encode_tuple_t_stringliteral_24883cc5fe64ace9d0df1893501ecb93c77180f0ff69cca79affb3c316dc8029__to_t_string_memory_ptr__fromStack(headStart ) -> tail {
                tail := add(headStart, 32)

                mstore(add(headStart, 0), sub(tail, headStart))
                tail := abi_encode_t_stringliteral_24883cc5fe64ace9d0df1893501ecb93c77180f0ff69cca79affb3c316dc8029_to_t_string_memory_ptr_fromStack( tail)

            }

            function require_helper_t_stringliteral_24883cc5fe64ace9d0df1893501ecb93c77180f0ff69cca79affb3c316dc8029(condition ) {
                if iszero(condition) {
                    let memPtr := allocate_unbounded()
                    mstore(memPtr, 0x08c379a000000000000000000000000000000000000000000000000000000000)
                    let end := abi_encode_tuple_t_stringliteral_24883cc5fe64ace9d0df1893501ecb93c77180f0ff69cca79affb3c316dc8029__to_t_string_memory_ptr__fromStack(add(memPtr, 4) )
                    revert(memPtr, sub(end, memPtr))
                }
            }

            /// @ast-id 521
            /// @src 1:10113:10483  "function _approve(..."
            function fun__approve_521(var_owner_479, var_spender_481, var_amount_483) {

                /// @src 1:10244:10249  "owner"
                let _93 := var_owner_479
                let expr_487 := _93
                /// @src 1:10261:10262  "0"
                let expr_490 := 0x00
                /// @src 1:10253:10263  "address(0)"
                let expr_491 := convert_t_rational_0_by_1_to_t_address(expr_490)
                /// @src 1:10244:10263  "owner != address(0)"
                let expr_492 := iszero(eq(cleanup_t_address(expr_487), cleanup_t_address(expr_491)))
                /// @src 1:10236:10304  "require(owner != address(0), \"ERC20: approve from the zero address\")"
                require_helper_t_stringliteral_c953f4879035ed60e766b34720f656aab5c697b141d924c283124ecedb91c208(expr_492)
                /// @src 1:10322:10329  "spender"
                let _94 := var_spender_481
                let expr_497 := _94
                /// @src 1:10341:10342  "0"
                let expr_500 := 0x00
                /// @src 1:10333:10343  "address(0)"
                let expr_501 := convert_t_rational_0_by_1_to_t_address(expr_500)
                /// @src 1:10322:10343  "spender != address(0)"
                let expr_502 := iszero(eq(cleanup_t_address(expr_497), cleanup_t_address(expr_501)))
                /// @src 1:10314:10382  "require(spender != address(0), \"ERC20: approve to the zero address\")"
                require_helper_t_stringliteral_24883cc5fe64ace9d0df1893501ecb93c77180f0ff69cca79affb3c316dc8029(expr_502)
                /// @src 1:10423:10429  "amount"
                let _95 := var_amount_483
                let expr_511 := _95
                /// @src 1:10393:10404  "_allowances"
                let _96 := 0x01
                let expr_506 := _96
                /// @src 1:10405:10410  "owner"
                let _97 := var_owner_479
                let expr_507 := _97
                /// @src 1:10393:10411  "_allowances[owner]"
                let _98 := mapping_index_access_t_mapping$_t_address_$_t_mapping$_t_address_$_t_uint256_$_$_of_t_address(expr_506,expr_507)
                let _99 := _98
                let expr_509 := _99
                /// @src 1:10412:10419  "spender"
                let _100 := var_spender_481
                let expr_508 := _100
                /// @src 1:10393:10420  "_allowances[owner][spender]"
                let _101 := mapping_index_access_t_mapping$_t_address_$_t_uint256_$_of_t_address(expr_509,expr_508)
                /// @src 1:10393:10429  "_allowances[owner][spender] = amount"
                update_storage_value_offset_0t_uint256_to_t_uint256(_101, expr_511)
                let expr_512 := expr_511
                /// @src 1:10453:10458  "owner"
                let _102 := var_owner_479
                let expr_515 := _102
                /// @src 1:10460:10467  "spender"
                let _103 := var_spender_481
                let expr_516 := _103
                /// @src 1:10469:10475  "amount"
                let _104 := var_amount_483
                let expr_517 := _104
                /// @src 1:10444:10476  "Approval(owner, spender, amount)"
                let _105 := 0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925
                let _106 := convert_t_address_to_t_address(expr_515)
                let _107 := convert_t_address_to_t_address(expr_516)
                {
                    let _108 := allocate_unbounded()
                    let _109 := abi_encode_tuple_t_uint256__to_t_uint256__fromStack(_108 , expr_517)
                    log3(_108, sub(_109, _108) , _105, _106, _107)
                }
            }
            /// @src 1:1393:12620  "contract ERC20 is Context, IERC20, IERC20Metadata {..."

            function store_literal_in_memory_3b6607e091cba9325f958656d2b5e0622ab7dc0eac71a26ac788cb25bc19f4fe(memPtr) {

                mstore(add(memPtr, 0), "ERC20: insufficient allowance")

            }

            function abi_encode_t_stringliteral_3b6607e091cba9325f958656d2b5e0622ab7dc0eac71a26ac788cb25bc19f4fe_to_t_string_memory_ptr_fromStack(pos) -> end {
                pos := array_storeLengthForEncoding_t_string_memory_ptr_fromStack(pos, 29)
                store_literal_in_memory_3b6607e091cba9325f958656d2b5e0622ab7dc0eac71a26ac788cb25bc19f4fe(pos)
                end := add(pos, 32)
            }

            function abi_encode_tuple_t_stringliteral_3b6607e091cba9325f958656d2b5e0622ab7dc0eac71a26ac788cb25bc19f4fe__to_t_string_memory_ptr__fromStack(headStart ) -> tail {
                tail := add(headStart, 32)

                mstore(add(headStart, 0), sub(tail, headStart))
                tail := abi_encode_t_stringliteral_3b6607e091cba9325f958656d2b5e0622ab7dc0eac71a26ac788cb25bc19f4fe_to_t_string_memory_ptr_fromStack( tail)

            }

            function require_helper_t_stringliteral_3b6607e091cba9325f958656d2b5e0622ab7dc0eac71a26ac788cb25bc19f4fe(condition ) {
                if iszero(condition) {
                    let memPtr := allocate_unbounded()
                    mstore(memPtr, 0x08c379a000000000000000000000000000000000000000000000000000000000)
                    let end := abi_encode_tuple_t_stringliteral_3b6607e091cba9325f958656d2b5e0622ab7dc0eac71a26ac788cb25bc19f4fe__to_t_string_memory_ptr__fromStack(add(memPtr, 4) )
                    revert(memPtr, sub(end, memPtr))
                }
            }

            /// @ast-id 564
            /// @src 1:10764:11205  "function _spendAllowance(..."
            function fun__spendAllowance_564(var_owner_524, var_spender_526, var_amount_528) {

                /// @src 1:10931:10936  "owner"
                let _110 := var_owner_524
                let expr_534 := _110
                /// @src 1:10938:10945  "spender"
                let _111 := var_spender_526
                let expr_535 := _111
                /// @src 1:10921:10946  "allowance(owner, spender)"
                let expr_536 := fun_allowance_141(expr_534, expr_535)
                /// @src 1:10894:10946  "uint256 currentAllowance = allowance(owner, spender)"
                let var_currentAllowance_532 := expr_536
                /// @src 1:10960:10976  "currentAllowance"
                let _112 := var_currentAllowance_532
                let expr_538 := _112
                /// @src 1:10980:10997  "type(uint256).max"
                let expr_543 := 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
                /// @src 1:10960:10997  "currentAllowance != type(uint256).max"
                let expr_544 := iszero(eq(cleanup_t_uint256(expr_538), cleanup_t_uint256(expr_543)))
                /// @src 1:10956:11199  "if (currentAllowance != type(uint256).max) {..."
                if expr_544 {
                    /// @src 1:11021:11037  "currentAllowance"
                    let _113 := var_currentAllowance_532
                    let expr_546 := _113
                    /// @src 1:11041:11047  "amount"
                    let _114 := var_amount_528
                    let expr_547 := _114
                    /// @src 1:11021:11047  "currentAllowance >= amount"
                    let expr_548 := iszero(lt(cleanup_t_uint256(expr_546), cleanup_t_uint256(expr_547)))
                    /// @src 1:11013:11081  "require(currentAllowance >= amount, \"ERC20: insufficient allowance\")"
                    require_helper_t_stringliteral_3b6607e091cba9325f958656d2b5e0622ab7dc0eac71a26ac788cb25bc19f4fe(expr_548)
                    /// @src 1:11132:11137  "owner"
                    let _115 := var_owner_524
                    let expr_553 := _115
                    /// @src 1:11139:11146  "spender"
                    let _116 := var_spender_526
                    let expr_554 := _116
                    /// @src 1:11148:11164  "currentAllowance"
                    let _117 := var_currentAllowance_532
                    let expr_555 := _117
                    /// @src 1:11167:11173  "amount"
                    let _118 := var_amount_528
                    let expr_556 := _118
                    /// @src 1:11148:11173  "currentAllowance - amount"
                    let expr_557 := wrapping_sub_t_uint256(expr_555, expr_556)

                    fun__approve_521(expr_553, expr_554, expr_557)
                    /// @src 1:10956:11199  "if (currentAllowance != type(uint256).max) {..."
                }

            }
            /// @src 1:1393:12620  "contract ERC20 is Context, IERC20, IERC20Metadata {..."

            /// @ast-id 575
            /// @src 1:11789:11910  "function _beforeTokenTransfer(..."
            function fun__beforeTokenTransfer_575(var_from_567, var_to_569, var_amount_571) {

            }
            /// @src 1:1393:12620  "contract ERC20 is Context, IERC20, IERC20Metadata {..."

            /// @ast-id 586
            /// @src 1:12498:12618  "function _afterTokenTransfer(..."
            function fun__afterTokenTransfer_586(var_from_578, var_to_580, var_amount_582) {

            }
            /// @src 1:1393:12620  "contract ERC20 is Context, IERC20, IERC20Metadata {..."

            function zero_value_for_split_t_address() -> ret {
                ret := 0
            }

            /// @ast-id 599
            /// @src 0:640:736  "function _msgSender() internal view virtual returns (address) {..."
            function fun__msgSender_599() -> var__593 {
                /// @src 0:693:700  "address"
                let zero_t_address_119 := zero_value_for_split_t_address()
                var__593 := zero_t_address_119

                /// @src 0:719:729  "msg.sender"
                let expr_596 := caller()
                /// @src 0:712:729  "return msg.sender"
                var__593 := expr_596
                leave

            }
            /// @src 1:1393:12620  "contract ERC20 is Context, IERC20, IERC20Metadata {..."

        }

        data ".metadata" hex"a3646970667358221220430c597f6f1345201b97d2e55b23aeca5dea2c07e4e779d17a3007aebac2d92c6c6578706572696d656e74616cf564736f6c637828302e382e31322d646576656c6f702e323032322e322e31322b636f6d6d69742e31323130633365360067"
    }

}

    "###
        )));
    }
}
