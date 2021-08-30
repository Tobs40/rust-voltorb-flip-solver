


// convert states to u64 for hashmap
pub fn array_to_u64(state: &[[usize;5];5]) -> u64
{
    let mut r = 0;
    for row in 0..5
    {
        for col in 0..5
        {
            r <<= 2;
            r |= state[row][col] as u64;
        }
    }

    return r;
}


// convert u64 back to it's state
pub fn u64_to_array(r: u64) -> [[usize;5];5]
{
    let mut rc = r;
    let mut state = [[0;5];5];
    for row in (0..5).rev()
    {
        for col in (0..5).rev()
        {
            state[row][col] = rc as usize & 0x3;
            rc >>= 2;
        }
    }

    state
}


pub fn get_from_packed_state(
    packed_state: u64,
    row: usize,
    col: usize,
) -> usize
{
    ((packed_state >> ((4-row)*10+(4-col)*2)) & 0x3) as usize
}


pub fn set_in_packed_state(
    packed_state: u64,
    row: usize,
    col: usize,
    symbol: usize,
) -> u64
{
    let by = ((4-row)*10+(4-col)*2) as u64;
    let tmp = packed_state & !(0x3 << by);
    tmp | (symbol << by) as u64
}

// takes a board (0=bomb,1,2,3) and a state (0=unassigned,1,2,3)
// and determines whether playing from this state could result in that board e.g.
// for every square: if the state is unassigned we don't care, but if it's assigned
// it has to match the square of the board
pub fn is_possible_board_of_state(
    packed_board: u64,
    packed_state: u64
) -> bool
{
    // for every square
    // 00 --> 00
    // 01 --> 11
    // 10 --> 11
    // 11 --> 11
    // so that every assigned square is covered with set bits

    let mask_upper = 0xAAAA_AAAA_AAAA_AAAA_u64;
    let upper_bits = mask_upper & packed_state;

    let mask_lower = 0x5555_5555_5555_5555_u64;
    let lower_bits = mask_lower & packed_state;
    let mask_assigned = packed_state | (lower_bits << 1) | (upper_bits >> 1);

    let possible = (packed_board & mask_assigned) == packed_state;

    if false
    {
        println!("{:#066b} board", packed_board);
        println!("{:#066b} state", packed_state);

        println!("{:#066b} mask_upper", mask_upper);
        println!("{:#066b} upper", upper_bits);

        println!("{:#066b} mask_lower", mask_lower);
        println!("{:#066b} lower", lower_bits);

        println!("{:#066b} mask_assigned", mask_assigned);

        println!();
    }

    possible
}

// determines whether there is a 2 or 3 on a square in the given board
// which is unassigned in the given state, assuming that the board is a possible board
// of that state
pub fn board_has_possible_2_3_for_state(
    packed_board: u64,
    packed_state: u64
) -> bool
{
    // upper bits are set for a square if it's a two or three
    let mask_upper = 0xAAAA_AAAA_AAAA_AAAA_u64;

    let has = (packed_board & mask_upper) != (packed_state & mask_upper);

    if false
    {
        println!("{:#066b} board", packed_board);
        println!("{:#066b} state", packed_state);

        println!("{:#066b} mask_upper", mask_upper);

        println!();
    }

    has
}


pub fn coins_of_state(state: u64) -> usize
{
    let mut coins = 0;

    for r in 0..5
    {
        for c in 0..5
        {
            let s = get_from_packed_state(state, r, c);

            if s > 0
            {
                if coins == 0
                {
                    coins = s;
                }
                else {
                    coins *= s;
                }
            }
        }
    }

    coins
}