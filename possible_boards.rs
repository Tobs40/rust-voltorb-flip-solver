use crate::packed::{array_to_u64, is_possible_board_of_state, get_from_packed_state, u64_to_array};
use crate::csp_constraints::find_possible_boards;


// There are very few boards (low 2 digit number) most of the time,
// like a single digit number of boards, rarely more than 50
// so optimize for small sizes

pub fn accumulate_symbol_weights(
    packed_state: u64,
    possible_boards: &Vec<u64>,
    indices: &Vec<usize>,
    index_start: usize,
    weights: &Vec<f64>,
) -> [[[f64;4];5];5]
{
    // array for storing the probs of a square (row, col) turning out to be one of the four symbols
    let mut acc = [[[0.0;4];5];5];

    // for each weight group
    for index_weight in 0..weights.len()
    {
        // for each board of that weight
        for i in indices[index_start+index_weight]..indices[index_start+index_weight+1]
        {
            let packed_board = possible_boards[i];
            for r in 0..5
            {
                for c in 0..5
                {
                    // figure out the symbol and add the weight of the state
                    let symbol = get_from_packed_state(packed_board, r, c);
                    acc[r][c][symbol] += weights[index_weight];
                }
            }
        }
    }

    if true
    {
        let total = acc[0][0][0] + acc[0][0][1] + acc[0][0][2] + acc[0][0][3];

        for r in 0..5
        {
            for c in 0..5
            {
                for s in 0..4
                {
                    acc[r][c][s] /= total;
                }
            }
        }
    }

    acc
}

// removes all states without state[row][col] == symbol
pub fn filter_possible_boards_of_next_depth(
    packed_state: u64,
    possible_boards: &mut Vec<u64>,
    indices: &mut Vec<usize>,
    index_start: usize,
    weights: &Vec<f64>,
) -> ()
{
    // truncate because we're adding now!
    possible_boards.truncate(indices[index_start+weights.len()]);

    // for each weight group
    for index_weight in 0..weights.len()
    {
        // for each board of that depth and weight
        for i in indices[index_start+index_weight]..indices[index_start+index_weight+1]
        {
            let packed_board = possible_boards[i];
            if is_possible_board_of_state(packed_board, packed_state) // faster than comparing symbols actually
            {
                possible_boards.push(packed_board);
            }
        }
        // the index where we would've inserted next is the start of the next group.
        // Or the end of the boards of that depth, anyway set the index correctly:
        indices[index_start+weights.len()+index_weight+1] = possible_boards.len();
    }
}